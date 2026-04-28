use crate::network::tcp::TcpPort;
use crate::network::wire_protocol::{WireChatMessageContent, WireMessage};
use application::events::AppEvent;
use application::ports::event_sender::EventSender;
use application::ports::inbound_message_handler::InboundMessageHandler;
use domain::peer::PeerAddress;
use futures::StreamExt;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
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
    handler: Arc<dyn InboundMessageHandler>,
    event_sender: Arc<dyn EventSender>,
}

impl TcpInboundListener {
    pub fn new(
        ip: IpAddr,
        port: TcpPort,
        handler: Arc<dyn InboundMessageHandler>,
        event_sender: Arc<dyn EventSender>,
    ) -> Self {
        Self {
            ip,
            port,
            handler,
            event_sender,
        }
    }

    pub async fn listen(&self) -> Result<(), TcpInboundListenerError> {
        let listener = self.bind().await?;

        while let Ok((stream, addr)) = listener.accept().await {
            let handler = self.handler.clone();
            let event_sender = self.event_sender.clone();
            task::spawn(async move {
                Self::handle_connection(stream, addr, handler, event_sender).await;
            });
        }

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: SocketAddr,
        handler: Arc<dyn InboundMessageHandler>,
        event_sender: Arc<dyn EventSender>,
    ) {
        let peer_address = PeerAddress::new(addr.ip().to_string().into());
        event_sender
            .send(AppEvent::PeerConnected(peer_address.clone()))
            .await;

        let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
        while let Some(Ok(bytes)) = framed.next().await {
            if let Ok(text) = String::from_utf8(bytes.to_vec()) {
                if let Ok(wire_msg) = serde_json::from_str::<WireMessage>(&text) {
                    match wire_msg {
                        WireMessage::ChatMessage(wm) => {
                            let content_text = match wm.content {
                                WireChatMessageContent::Text(t) => t,
                            };
                            handler
                                .handle_message(peer_address.clone(), content_text)
                                .await;
                        }
                    }
                }
            }
        }

        event_sender
            .send(AppEvent::PeerDisconnected(peer_address))
            .await;
    }

    async fn bind(&self) -> Result<TcpListener, TcpInboundListenerError> {
        TcpListener::bind((self.ip, self.port))
            .await
            .map_err(|e| TcpInboundListenerError::BindError {
                ip: self.ip,
                port: self.port,
                reason: e.to_string(),
            })
    }
}
