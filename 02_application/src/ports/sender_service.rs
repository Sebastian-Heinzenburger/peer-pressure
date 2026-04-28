use async_trait::async_trait;
use domain::message::ChatMessage;
use domain::peer::PeerAddress;

#[derive(Debug, thiserror::Error)]
pub enum ConnectionServiceError {
    #[error("Invalid Address: {0}")]
    InvalidAddress(PeerAddress),
    #[error("Error while connecting: {0}")]
    ConnectionError(String),
    #[error("Not Connected")]
    NotConnected,
    #[error("Error while sending message: {0}")]
    SendError(String),
}

#[async_trait]
pub trait MessageSenderService {
    async fn connect(&self, peer: PeerAddress) -> Result<(), ConnectionServiceError>;
    async fn disconnect(&self, peer: PeerAddress) -> Result<(), ConnectionServiceError>;
    async fn send(
        &self,
        peer: PeerAddress,
        message: ChatMessage,
    ) -> Result<(), ConnectionServiceError>;
}
