use crate::ports::repository::error::RepositoryError;
use async_trait::async_trait;
use domain::peer::{Peer, PeerId};

#[async_trait]
pub trait PeerRepository: Send + Sync {
    async fn add(&self, peer: Peer) -> Result<(), RepositoryError>;

    async fn get(&self, id: &PeerId) -> Result<Option<Peer>, RepositoryError>;

    async fn list(&self) -> Result<Vec<Peer>, RepositoryError>;

    async fn remove(&self, id: &PeerId) -> Result<(), RepositoryError>;
}
