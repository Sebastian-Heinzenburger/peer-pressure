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
