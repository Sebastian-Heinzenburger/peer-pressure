use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use crate::ports::inbound_message_handler::InboundMessageReceiver;
use crate::ports::repository::chat::ChatRepository;
use async_trait::async_trait;
use domain::chat::Chat;
use domain::message::{ChatMessage, MessageContent, SentBy};
use domain::peer::PeerAddress;
use std::sync::Arc;

pub struct ReceiveMessage<C: ChatRepository, E: EventSender> {
    chat_repository: Arc<C>,
    event_sender: Arc<E>,
}

#[async_trait]
impl<C, E> InboundMessageReceiver for ReceiveMessage<C, E>
where
    C: ChatRepository + Send + Sync,
    E: EventSender + Send + Sync,
{
    async fn receive_message(&self, from: PeerAddress, content: MessageContent) {
        let message = ChatMessage::create(SentBy::Peer, content.clone());
        self.persist_message_to_chat(&from, message).await;
        self.emit_message_received_event(from, content).await;
    }
}

impl<C, E> ReceiveMessage<C, E>
where
    C: ChatRepository + Send + Sync,
    E: EventSender + Send + Sync,
{
    pub fn new(chat_repository: Arc<C>, event_sender: Arc<E>) -> Self {
        Self {
            chat_repository,
            event_sender,
        }
    }

    async fn persist_message_to_chat(&self, from: &PeerAddress, message: ChatMessage) {
        let mut chat = self
            .chat_repository
            .get(&from)
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| Chat::create(from.clone()));

        chat.add_message(message);

        let _ = self.chat_repository.save(chat).await;
    }

    async fn emit_message_received_event(&self, from: PeerAddress, content: MessageContent) {
        self.event_sender
            .send(AppEvent::MessageReceived {
                peer: from,
                content,
            })
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::test_helper::{MockChatRepository, MockEventSender};
    use domain::message::DeliveryStatus;

    #[tokio::test]
    async fn receive_message_persists_and_emits() {
        let chat_repo = Arc::new(MockChatRepository::new());
        let events = Arc::new(MockEventSender::new());
        let uc = ReceiveMessage::new(chat_repo.clone(), events.clone());
        let peer = PeerAddress::new("5.6.7.8".into());
        let content = MessageContent::Text("hey".into());

        uc.receive_message(peer.clone(), content.clone()).await;

        let chat = chat_repo.get_chat(&peer).await.unwrap();
        assert_eq!(chat.messages.len(), 1);
        assert_eq!(chat.messages[0].sent_by(), &SentBy::Peer);
        assert_eq!(chat.messages[0].delivery_status(), &DeliveryStatus::Sent);

        let evts = events.get().await;
        assert_eq!(evts[0], AppEvent::MessageReceived { peer, content });
    }
}
