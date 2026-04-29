use crate::error::ApplicationError;
use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use crate::ports::repository::chat::ChatRepository;
use crate::ports::repository::error::RepositoryError;
use crate::ports::sender_service::{ConnectionServiceError, MessageSenderService};
use domain::chat::Chat;
use domain::message::{ChatMessage, MessageContent, SentBy};
use domain::peer::PeerId;
use std::sync::Arc;

pub struct SendMessage<C: ChatRepository, S: MessageSenderService, E: EventSender> {
    chat_repository: Arc<C>,
    sender_service: Arc<S>,
    event_sender: Arc<E>,
}

impl<C, S, E> SendMessage<C, S, E>
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

    pub async fn execute(&self, peer: PeerId, text: String) -> Result<(), ApplicationError> {
        let content = MessageContent::Text(text);
        let mut message = ChatMessage::create(SentBy::Me, content.clone());

        let send_result = self
            .sender_service
            .send(peer.clone(), message.clone())
            .await;

        if send_result.is_ok() {
            message.mark_sent();
        }

        self.persist_message_to_chat(&peer, message).await?;

        self.emit_event(peer, content, send_result).await;

        Ok(())
    }

    async fn emit_event(
        &self,
        peer: PeerId,
        content: MessageContent,
        send_result: Result<(), ConnectionServiceError>,
    ) {
        let message_sent_event = AppEvent::MessageSent {
            peer,
            content: content.clone(),
            delivered: send_result.is_ok(),
        };
        self.event_sender.send(message_sent_event).await;
    }

    async fn persist_message_to_chat(
        &self,
        peer: &PeerId,
        message: ChatMessage,
    ) -> Result<(), RepositoryError> {
        let mut chat = self
            .chat_repository
            .get(peer)
            .await?
            .unwrap_or_else(|| Chat::create(peer.clone()));
        chat.add_message(message);
        self.chat_repository.save(chat).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::test_helper::{
        MockChatRepository, MockEventSender, MockMessageSenderService,
    };
    use domain::message::DeliveryStatus;
    use domain::peer::PeerAddress;

    #[tokio::test]
    async fn send_message_success() {
        let chat_repo = Arc::new(MockChatRepository::new());
        let sender = Arc::new(MockMessageSenderService::new());
        let events = Arc::new(MockEventSender::new());
        let uc = SendMessage::new(chat_repo.clone(), sender.clone(), events.clone());
        let peer = PeerAddress::new("1.2.3.4".into());

        uc.execute(peer.clone(), "hello".into()).await.unwrap();

        let chat = chat_repo.get_chat(&peer).await.unwrap();
        assert_eq!(chat.messages.len(), 1);
        assert_eq!(chat.messages[0].delivery_status(), &DeliveryStatus::Sent);
        assert_eq!(sender.sent.read().await.len(), 1);

        let evts = events.get_events().await;
        assert_eq!(
            evts[0],
            AppEvent::MessageSent {
                peer,
                content: MessageContent::Text("hello".into()),
                delivered: true,
            }
        );
    }

    #[tokio::test]
    async fn send_message_network_failure() {
        let chat_repo = Arc::new(MockChatRepository::new());
        let sender = Arc::new(MockMessageSenderService::failing());
        let events = Arc::new(MockEventSender::new());
        let uc = SendMessage::new(chat_repo.clone(), sender, events.clone());
        let peer = PeerAddress::new("1.2.3.4".into());

        uc.execute(peer.clone(), "hello".into()).await.unwrap();

        let chat = chat_repo.get_chat(&peer).await.unwrap();
        assert_eq!(chat.messages[0].delivery_status(), &DeliveryStatus::NotSent);

        let received_events = events.get_events().await;
        assert_eq!(
            received_events[0],
            AppEvent::MessageSent {
                peer,
                content: MessageContent::Text("hello".into()),
                delivered: false,
            }
        );
    }
}
