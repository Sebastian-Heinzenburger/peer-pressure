use crate::ports::repository::error::RepositoryError;
use crate::ports::sender_service::ConnectionServiceError;
use domain::error::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error("Business rule violation: {0}")]
    DomainError(#[from] DomainError),
    #[error(transparent)]
    RepositoryError(#[from] RepositoryError),
    #[error("Peer already exists")]
    PeerAlreadyExists,
    #[error("Network error: {0}")]
    NetworkError(#[from] ConnectionServiceError),
}
