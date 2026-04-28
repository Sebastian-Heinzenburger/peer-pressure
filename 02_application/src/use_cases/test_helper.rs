use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use crate::ports::repository::error::RepositoryError;
use crate::ports::repository::peer::PeerRepository;
use async_trait::async_trait;
use domain::peer::{Peer, PeerAddress, PeerId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};

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
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(vec![])),
        }
    }
    pub(crate) async fn get(&self) -> RwLockReadGuard<'_, Vec<AppEvent>> {
        self.events.read().await
    }
}

pub struct MockPeerRepository {
    peers: RwLock<HashMap<PeerAddress, Peer>>,
}

impl MockPeerRepository {
    pub fn new() -> Self {
        MockPeerRepository {
            peers: RwLock::new(HashMap::new()),
        }
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
