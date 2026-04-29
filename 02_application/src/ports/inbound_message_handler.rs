use async_trait::async_trait;
use domain::message::MessageContent;
use domain::peer::PeerAddress;

#[async_trait]
pub trait InboundMessageReceiver: Send + Sync {
    async fn receive_message(&self, from: PeerAddress, content: MessageContent);
}
