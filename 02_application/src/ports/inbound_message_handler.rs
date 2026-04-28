use async_trait::async_trait;
use domain::peer::PeerAddress;

#[async_trait]
pub trait InboundMessageHandler: Send + Sync {
    async fn handle_message(&self, from: PeerAddress, content: String);
}
