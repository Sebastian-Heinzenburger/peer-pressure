#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Peer with address Not Found: {0}")]
    PeerNotFound(String),
}
