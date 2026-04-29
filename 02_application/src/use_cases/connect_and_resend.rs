use crate::error::ApplicationError;
use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use crate::ports::repository::chat::ChatRepository;
use crate::ports::sender_service::{ConnectionServiceError, MessageSenderService};
use domain::chat::Chat;
use domain::message::{ChatMessage, DeliveryStatus, SentBy};
use domain::peer::PeerId;
use std::sync::Arc;

pub struct ConnectAndResend<C: ChatRepository, S: MessageSenderService, E: EventSender> {
    chat_repository: Arc<C>,
    sender_service: Arc<S>,
    event_sender: Arc<E>,
}

impl<C, S, E> ConnectAndResend<C, S, E>
where
    C: ChatRepository,
    S: MessageSenderService,
    E: EventSender,
{
    pub fn new(chat_repository: Arc<C>, sender_service: Arc<S>, event_sender: Arc<E>) -> Self {
        Self {
            chat_repository,
            sender_service,
            event_sender,
        }
    }

    pub async fn execute(&self, peer: PeerId) -> Result<(), ApplicationError> {
        self.connect_to_peer(&peer).await?;

        let mut chat = self
            .chat_repository
            .get(&peer)
            .await?
            .unwrap_or_else(|| Chat::create(peer.clone()));

        if self.resend_unsent_messages(&peer, &mut chat).await {
            self.chat_repository.save(chat).await?;
            self.event_sender
                .send(AppEvent::MessagesDelivered { peer })
                .await;
        }

        Ok(())
    }

    async fn resend_unsent_messages(&self, peer: &PeerId, chat: &mut Chat) -> bool {
        let unsent_messages = chat.messages.iter_mut().filter(|m| {
            m.sent_by() == &SentBy::Me && m.delivery_status() == &DeliveryStatus::NotSent
        });

        let mut any_sent = false;
        for msg in unsent_messages {
            if self.resend_message(peer, msg).await.is_ok() {
                msg.mark_sent();
                any_sent = true;
            }
        }
        any_sent
    }

    async fn resend_message(
        &self,
        peer: &PeerId,
        msg: &mut ChatMessage,
    ) -> Result<(), ConnectionServiceError> {
        self.sender_service.send(peer.clone(), msg.clone()).await
    }

    async fn connect_to_peer(&self, peer: &PeerId) -> Result<(), ConnectionServiceError> {
        self.sender_service.connect(peer.clone()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::test_helper::{MockChatRepository, MockEventSender, MockMessageSenderService};
    use domain::message::MessageContent;
    use domain::peer::PeerAddress;

    fn make_unsent_msg(text: &str) -> ChatMessage {
        ChatMessage::create(SentBy::Me, MessageContent::Text(text.into()))
    }

    fn make_sent_msg(text: &str) -> ChatMessage {
        let mut m = ChatMessage::create(SentBy::Me, MessageContent::Text(text.into()));
        m.mark_sent();
        m
    }

    #[tokio::test]
    async fn resends_unsent_messages() {
        let peer = PeerAddress::new("1.2.3.4".into());
        let chat_repo = Arc::new(MockChatRepository::new());
        let mut chat = Chat::create(peer.clone());
        chat.add_message(make_unsent_msg("a"));
        chat.add_message(make_unsent_msg("b"));
        chat_repo.set_chat(chat).await;

        let sender = Arc::new(MockMessageSenderService::new());
        let events = Arc::new(MockEventSender::new());
        let uc = ConnectAndResend::new(chat_repo.clone(), sender.clone(), events.clone());

        uc.execute(peer.clone()).await.unwrap();

        let chat = chat_repo.get_chat(&peer).await.unwrap();
        assert!(chat.messages.iter().all(|m| m.delivery_status() == &DeliveryStatus::Sent));
        assert_eq!(sender.sent.read().await.len(), 2);
        assert_eq!(events.get().await[0], AppEvent::MessagesDelivered { peer });
    }

    #[tokio::test]
    async fn skips_already_sent() {
        let peer = PeerAddress::new("1.2.3.4".into());
        let chat_repo = Arc::new(MockChatRepository::new());
        let mut chat = Chat::create(peer.clone());
        chat.add_message(make_sent_msg("old"));
        chat.add_message(make_unsent_msg("new"));
        chat_repo.set_chat(chat).await;

        let sender = Arc::new(MockMessageSenderService::new());
        let events = Arc::new(MockEventSender::new());
        let uc = ConnectAndResend::new(chat_repo.clone(), sender.clone(), events.clone());

        uc.execute(peer.clone()).await.unwrap();

        assert_eq!(sender.sent.read().await.len(), 1); // only "new" resent
    }

    #[tokio::test]
    async fn connect_failure() {
        let peer = PeerAddress::new("1.2.3.4".into());
        let chat_repo = Arc::new(MockChatRepository::new());
        let sender = Arc::new(MockMessageSenderService::failing());
        let events = Arc::new(MockEventSender::new());
        let uc = ConnectAndResend::new(chat_repo, sender, events.clone());

        let result = uc.execute(peer).await;
        assert!(result.is_err());
        assert!(events.get().await.is_empty());
    }
}
