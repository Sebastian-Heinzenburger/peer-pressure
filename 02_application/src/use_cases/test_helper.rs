use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use crate::ports::repository::chat::ChatRepository;
use crate::ports::repository::error::RepositoryError;
use crate::ports::repository::peer::PeerRepository;
use crate::ports::sender_service::{ConnectionServiceError, MessageSenderService};
use async_trait::async_trait;
use domain::chat::Chat;
use domain::message::ChatMessage;
use domain::peer::{Peer, PeerAddress, PeerId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct MockEventSender {
    events: Arc<RwLock<Vec<AppEvent>>>,
}

#[async_trait]
impl EventSender for MockEventSender {
    async fn send(&self, event: AppEvent) {
        self.events.write().await.push(event)
    }
}

impl MockEventSender {
    #[cfg(test)]
    pub async fn get_events(&self) -> Vec<AppEvent> {
        self.events.read().await.clone()
    }
}

impl MockEventSender {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(vec![])),
        }
    }
}

#[derive(Default)]
pub struct MockPeerRepository {
    peers: RwLock<HashMap<PeerAddress, Peer>>,
}

impl MockPeerRepository {
    pub fn new() -> Self {
        Default::default()
    }
}

#[async_trait]
impl PeerRepository for MockPeerRepository {
    async fn add(&self, peer: Peer) -> Result<(), RepositoryError> {
        let mut map = self.peers.write().await;
        map.insert(peer.address().clone(), peer);
        Ok(())
    }

    async fn get(&self, address: &PeerAddress) -> Result<Option<Peer>, RepositoryError> {
        let map = self.peers.read().await;
        Ok(map.get(address).cloned())
    }

    async fn list(&self) -> Result<Vec<Peer>, RepositoryError> {
        let map = self.peers.read().await;
        Ok(map.values().cloned().collect())
    }

    async fn remove(&self, id: &PeerId) -> Result<(), RepositoryError> {
        let mut map = self.peers.write().await;
        map.remove(id);
        Ok(())
    }
}

#[derive(Default)]
pub struct MockChatRepository {
    chats: RwLock<HashMap<String, Chat>>,
}

impl MockChatRepository {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn get_chat(&self, peer: &PeerId) -> Option<Chat> {
        self.chats.read().await.get(&peer.to_string()).cloned()
    }

    pub async fn set_chat(&self, chat: Chat) {
        self.chats.write().await.insert(chat.peer.to_string(), chat);
    }
}

#[async_trait]
impl ChatRepository for MockChatRepository {
    async fn get(&self, peer: &PeerId) -> Result<Option<Chat>, RepositoryError> {
        Ok(self.chats.read().await.get(&peer.to_string()).cloned())
    }

    async fn save(&self, chat: Chat) -> Result<(), RepositoryError> {
        self.chats.write().await.insert(chat.peer.to_string(), chat);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Chat>, RepositoryError> {
        Ok(self.chats.read().await.values().cloned().collect())
    }
}

#[derive(Default)]
pub struct MockMessageSenderService {
    pub should_fail: RwLock<bool>,
    pub sent: RwLock<Vec<(PeerAddress, ChatMessage)>>,
    pub connected: RwLock<Vec<PeerAddress>>,
}

impl MockMessageSenderService {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn failing() -> Self {
        Self {
            should_fail: RwLock::new(true),
            sent: RwLock::new(vec![]),
            connected: RwLock::new(vec![]),
        }
    }
}

#[async_trait]
impl MessageSenderService for MockMessageSenderService {
    async fn connect(&self, peer: PeerAddress) -> Result<(), ConnectionServiceError> {
        if *self.should_fail.read().await {
            return Err(ConnectionServiceError::ConnectionError(
                "mock failure".into(),
            ));
        }
        self.connected.write().await.push(peer);
        Ok(())
    }

    async fn disconnect(&self, _peer: PeerAddress) -> Result<(), ConnectionServiceError> {
        Ok(())
    }

    async fn send(
        &self,
        peer: PeerAddress,
        message: ChatMessage,
    ) -> Result<(), ConnectionServiceError> {
        if *self.should_fail.read().await {
            return Err(ConnectionServiceError::SendError("mock failure".into()));
        }
        self.sent.write().await.push((peer, message));
        Ok(())
    }
}
