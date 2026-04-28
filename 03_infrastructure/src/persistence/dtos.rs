use domain::chat::Chat;
use domain::message::{ChatMessage, DeliveryStatus, MessageContent, MessageId, SentBy};
use domain::peer::{Peer, PeerAddress};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct PeerDto {
    pub address: PeerAddressDto,
}

#[derive(Serialize, Deserialize)]
pub struct PeerAddressDto(pub String);

#[derive(Serialize, Deserialize)]
pub struct MessageVecDto {
    pub messages: Vec<ChatMessageDto>,
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessageDto {
    pub id: String,
    pub content: MessageContentDto,
    pub sent_by: SentByDto,
    pub delivery_status: DeliveryStatusDto,
}

#[derive(Serialize, Deserialize)]
pub enum MessageContentDto {
    Text(String),
}

#[derive(Serialize, Deserialize)]
pub enum SentByDto {
    Me,
    Peer,
}

#[derive(Serialize, Deserialize)]
pub enum DeliveryStatusDto {
    Sent,
    NotSent,
}

impl From<&SentBy> for SentByDto {
    fn from(sent_by: &SentBy) -> Self {
        match sent_by {
            SentBy::Me => SentByDto::Me,
            SentBy::Peer => SentByDto::Peer,
        }
    }
}

impl From<&SentByDto> for SentBy {
    fn from(sent_by_dto: &SentByDto) -> Self {
        match sent_by_dto {
            SentByDto::Me => SentBy::Me,
            SentByDto::Peer => SentBy::Peer,
        }
    }
}

impl From<&DeliveryStatus> for DeliveryStatusDto {
    fn from(status: &DeliveryStatus) -> Self {
        match status {
            DeliveryStatus::Sent => DeliveryStatusDto::Sent,
            DeliveryStatus::NotSent => DeliveryStatusDto::NotSent,
        }
    }
}

impl From<&DeliveryStatusDto> for DeliveryStatus {
    fn from(dto: &DeliveryStatusDto) -> Self {
        match dto {
            DeliveryStatusDto::Sent => DeliveryStatus::Sent,
            DeliveryStatusDto::NotSent => DeliveryStatus::NotSent,
        }
    }
}

impl From<MessageContent> for MessageContentDto {
    fn from(content: MessageContent) -> Self {
        match content {
            MessageContent::Text(t) => MessageContentDto::Text(t),
        }
    }
}

impl From<MessageContentDto> for MessageContent {
    fn from(content_dto: MessageContentDto) -> Self {
        match content_dto {
            MessageContentDto::Text(t) => MessageContent::Text(t),
        }
    }
}

impl From<&ChatMessage> for ChatMessageDto {
    fn from(chat_message: &ChatMessage) -> Self {
        let id = chat_message.id().to_string();
        let sent_by = SentByDto::from(chat_message.sent_by());
        let content = MessageContentDto::from(chat_message.content.clone());
        let delivery_status = DeliveryStatusDto::from(chat_message.delivery_status());
        Self {
            id,
            content,
            sent_by,
            delivery_status,
        }
    }
}

impl TryFrom<ChatMessageDto> for ChatMessage {
    type Error = ();

    fn try_from(chat_message: ChatMessageDto) -> Result<Self, Self::Error> {
        let id = MessageId::parse_str(&chat_message.id).map_err(|_| ())?;
        let sent_by = SentBy::from(&chat_message.sent_by);
        let content = MessageContent::from(chat_message.content);
        let delivery_status = DeliveryStatus::from(&chat_message.delivery_status);
        Ok(ChatMessage::new(id, sent_by, content, delivery_status))
    }
}

impl From<Chat> for MessageVecDto {
    fn from(value: Chat) -> Self {
        Self {
            messages: value.messages.iter().map(ChatMessageDto::from).collect(),
        }
    }
}

impl From<&[ChatMessage]> for MessageVecDto {
    fn from(messages: &[ChatMessage]) -> Self {
        Self {
            messages: messages.iter().map(ChatMessageDto::from).collect(),
        }
    }
}

impl From<PeerAddress> for PeerAddressDto {
    fn from(value: PeerAddress) -> Self {
        PeerAddressDto(value.to_string())
    }
}

impl From<PeerAddressDto> for PeerAddress {
    fn from(value: PeerAddressDto) -> Self {
        PeerAddress::new(Arc::from(value.0))
    }
}

impl From<Peer> for PeerDto {
    fn from(peer: Peer) -> Self {
        Self {
            address: PeerAddressDto::from(peer.address()),
        }
    }
}

impl TryFrom<PeerDto> for Peer {
    type Error = ();

    fn try_from(value: PeerDto) -> Result<Self, Self::Error> {
        let peer_address = PeerAddress::from(value.address);
        Ok(Peer::new(peer_address))
    }
}
