use crate::network::tcp::TcpPort;
use crate::network::wire_protocol::WireMessage;
use application::ports::event_sender::EventSender;
use application::ports::inbound_message_handler::InboundMessageReceiver;
use application::use_cases::AddPeer;
use domain::message::MessageContent;
use domain::peer::PeerAddress;
use futures::StreamExt;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug, thiserror::Error)]
pub enum TcpInboundListenerError {
    #[error("Could not bind to {ip}:{port}: {reason}")]
    BindError {
        ip: IpAddr,
        port: TcpPort,
        reason: String,
    },
}

pub struct TcpInboundListener {
    ip: IpAddr,
    port: TcpPort,
    inbound_message_handler: Arc<dyn InboundMessageReceiver>,
    add_peer_use_case: Arc<AddPeer>,
}

impl TcpInboundListener {
    pub fn new(
        ip: IpAddr,
        port: TcpPort,
        inbound_message_handler: Arc<dyn InboundMessageReceiver>,
        _event_sender: Arc<dyn EventSender>,
        add_peer_use_case: Arc<AddPeer>,
    ) -> Self {
        Self {
            ip,
            port,
            inbound_message_handler,
            add_peer_use_case,
        }
    }

    pub async fn listen(self) -> Result<(), TcpInboundListenerError> {
        let listener = self.bind().await?;

        while let Ok((stream, addr)) = listener.accept().await {
            let handler = self.inbound_message_handler.clone();
            let add_peer_use_case = self.add_peer_use_case.clone();
            task::spawn(async move {
                Self::handle_connection(stream, addr, handler, add_peer_use_case).await;
            });
        }

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        handler: Arc<dyn InboundMessageReceiver>,
        add_peer_use_case: Arc<AddPeer>,
    ) {
        let peer_address = PeerAddress::new(addr.ip().to_string().into());
        let _ = add_peer_use_case.execute(peer_address.clone()).await;

        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
        while let Some(Ok(bytes)) = framed.next().await {
            let handler = handler.clone();
            let _ignored = Self::handle_message(handler, &peer_address, bytes).await;
        }
    }

    async fn handle_message(
        handler: Arc<dyn InboundMessageReceiver>,
        peer_address: &PeerAddress,
        bytes: BytesMut,
    ) -> Result<(), ()> {
        let text = String::from_utf8(bytes.to_vec()).map_err(|_| ())?;
        let WireMessage::ChatMessage(wm) =
            serde_json::from_str::<WireMessage>(&text).map_err(|_| ())?;
        let content = MessageContent::from(wm.content);
        handler.receive_message(peer_address.clone(), content).await;
        Ok(())
    }

    async fn bind(&self) -> Result<TcpListener, TcpInboundListenerError> {
        TcpListener::bind((self.ip, self.port)).await.map_err(|e| {
            TcpInboundListenerError::BindError {
                ip: self.ip,
                port: self.port,
                reason: e.to_string(),
            }
        })
    }
}
