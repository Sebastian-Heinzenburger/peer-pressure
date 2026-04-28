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
            .get(&peer)
            .await?
            .unwrap_or_else(|| Chat::create(peer.clone()));
        chat.add_message(message);
        self.chat_repository.save(chat).await
    }
}
