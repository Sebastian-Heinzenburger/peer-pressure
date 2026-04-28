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
