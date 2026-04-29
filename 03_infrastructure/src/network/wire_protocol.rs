use domain::message::{ChatMessage, MessageContent};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum WireMessage {
    ChatMessage(WireChatMessage),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WireChatMessage {
    pub id: String,
    pub content: WireChatMessageContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WireChatMessageContent {
    Text(String),
}

impl From<&MessageContent> for WireChatMessageContent {
    fn from(message_content: &MessageContent) -> Self {
        match message_content {
            MessageContent::Text(text) => Self::Text(text.to_string()),
        }
    }
}

impl From<WireChatMessageContent> for MessageContent {
    fn from(content: WireChatMessageContent) -> Self {
        match content {
            WireChatMessageContent::Text(t) => Self::Text(t),
        }
    }
}

impl From<&ChatMessage> for WireMessage {
    fn from(value: &ChatMessage) -> Self {
        Self::ChatMessage(WireChatMessage {
            id: value.id().to_string(),
            content: WireChatMessageContent::from(&value.content),
        })
    }
}
