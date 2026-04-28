use crate::network::tcp::ip_peer_address::IpPeerAddress;
use crate::network::tcp::TcpPort;
use crate::network::wire_protocol::WireMessage;
use application::events::AppEvent;
use application::ports::event_sender::EventSender;
use application::ports::sender_service::{ConnectionServiceError, MessageSenderService};
use async_trait::async_trait;
use domain::message::ChatMessage;
use domain::peer::PeerAddress;
use futures::SinkExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio_util::bytes::Bytes;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct TcpOutboundConnectionService {
    connections: RwLock<HashMap<PeerAddress, Framed<TcpStream, LengthDelimitedCodec>>>,
    port: TcpPort,
    event_sender: Arc<dyn EventSender>,
}

impl TcpOutboundConnectionService {
    pub fn new(port: TcpPort, event_sender: Arc<dyn EventSender>) -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            port,
            event_sender,
        }
    }
}

#[async_trait]
impl MessageSenderService for TcpOutboundConnectionService {
    async fn connect(&self, peer_address: PeerAddress) -> Result<(), ConnectionServiceError> {
        let ip_addr = IpPeerAddress::try_from(&peer_address)
            .map_err(|_| ConnectionServiceError::InvalidAddress(peer_address.clone()))?
            .ip();

        let tcp_stream = TcpStream::connect((ip_addr, self.port))
            .await
            .map_err(|e| ConnectionServiceError::ConnectionError(e.to_string()))?;

        let framed = Framed::new(tcp_stream, LengthDelimitedCodec::new());

        self.connections
            .write()
            .await
            .insert(peer_address.clone(), framed);

        self.event_sender
            .send(AppEvent::PeerConnected(peer_address))
            .await;

        Ok(())
    }

    async fn disconnect(&self, peer: PeerAddress) -> Result<(), ConnectionServiceError> {
        let mut connections = self.connections.write().await;
        connections.remove(&peer);

        self.event_sender
            .send(AppEvent::PeerDisconnected(peer))
            .await;

        Ok(())
    }

    async fn send(
        &self,
        peer: PeerAddress,
        message: ChatMessage,
    ) -> Result<(), ConnectionServiceError> {
        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(&peer)
            .ok_or(ConnectionServiceError::NotConnected)?;

        // Probe connection health: a non-blocking read on a send-only socket
        // returns Ok(0) (EOF) or Err if the peer has disconnected.
        let mut probe_buf = [0u8; 1];
        match connection.get_ref().try_read(&mut probe_buf) {
            Ok(0) => {
                // EOF — peer closed the connection
                connections.remove(&peer);
                drop(connections);
                self.event_sender
                    .send(AppEvent::PeerDisconnected(peer))
                    .await;
                return Err(ConnectionServiceError::SendError(
                    "Connection closed by peer".to_string(),
                ));
            }
            Err(e) if e.kind() != std::io::ErrorKind::WouldBlock => {
                // Real error — connection is dead
                connections.remove(&peer);
                drop(connections);
                self.event_sender
                    .send(AppEvent::PeerDisconnected(peer))
                    .await;
                return Err(ConnectionServiceError::SendError(e.to_string()));
            }
            _ => {
                // WouldBlock means no data available — connection is still alive
            }
        }

        let wire_message = WireMessage::from(&message);
        let serialized = serde_json::to_string(&wire_message)
            .expect("Serialization of WireMessage should never fail");

        let result = connection.send(Bytes::from(serialized)).await;

        if let Err(e) = result {
            connections.remove(&peer);
            drop(connections);
            self.event_sender
                .send(AppEvent::PeerDisconnected(peer))
                .await;
            return Err(ConnectionServiceError::SendError(e.to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use application::events::AppEvent;
    use domain::message::{MessageContent, SentBy};
    use futures::StreamExt;
    use rand::random_range;
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use tokio::net::TcpListener;
    use tokio::task;
    use uuid::Uuid;

    #[derive(Default)]
    pub struct MockEventSender {
        events: Arc<RwLock<Vec<AppEvent>>>,
    }

    #[async_trait]
    impl EventSender for MockEventSender {
        async fn send(&self, event: AppEvent) {
            self.events.write().await.push(event);
        }
    }

    #[tokio::test]
    async fn it_works() {
        let localhost_peer_address = PeerAddress::new("127.0.0.1".into());
        let port = random_range(10000..60000);
        let message_content = "This is a test message";

        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .await
            .unwrap();
        let handle = task::spawn(async move {
            let (connection, _) = listener.accept().await.unwrap();
            let mut framed = Framed::new(connection, LengthDelimitedCodec::new());
            let message_bytes = framed.next().await.unwrap().unwrap();
            let message_str = String::from_utf8(message_bytes.to_vec()).unwrap();
            assert!(message_str.contains(message_content));
        });

        let events = Arc::new(MockEventSender::default());
        let sender_service = TcpOutboundConnectionService::new(port, events.clone());

        sender_service
            .connect(localhost_peer_address.clone())
            .await
            .unwrap();

        let msg = ChatMessage::new(
            Uuid::from_str("123e4567-e89b-12d3-a456-426614174000").unwrap(),
            SentBy::Me,
            MessageContent::Text(message_content.to_string()),
            domain::message::DeliveryStatus::NotSent,
        );
        sender_service
            .send(localhost_peer_address.clone(), msg)
            .await
            .unwrap();

        handle.await.unwrap();
        assert!(
            events
                .events
                .read()
                .await
                .contains(&AppEvent::PeerConnected(localhost_peer_address))
        );
    }
}
