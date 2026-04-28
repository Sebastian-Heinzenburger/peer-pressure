use uuid::Uuid;

pub type MessageId = Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum DeliveryStatus {
    Sent,
    NotSent,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    id: MessageId,
    sent_by: SentBy,
    pub content: MessageContent,
    delivery_status: DeliveryStatus,
}

impl ChatMessage {
    pub fn new(
        id: MessageId,
        sent_by: SentBy,
        content: MessageContent,
        delivery_status: DeliveryStatus,
    ) -> Self {
        Self {
            id,
            sent_by,
            content,
            delivery_status,
        }
    }

    pub fn create(sent_by: SentBy, content: MessageContent) -> Self {
        let id = Uuid::new_v4();
        let delivery_status = match &sent_by {
            SentBy::Me => DeliveryStatus::NotSent,
            SentBy::Peer => DeliveryStatus::Sent,
        };
        Self::new(id, sent_by, content, delivery_status)
    }

    pub fn sent_by(&self) -> &SentBy {
        &self.sent_by
    }

    pub fn id(&self) -> &MessageId {
        &self.id
    }

    pub fn delivery_status(&self) -> &DeliveryStatus {
        &self.delivery_status
    }

    pub fn mark_sent(&mut self) {
        self.delivery_status = DeliveryStatus::Sent;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageContent {
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SentBy {
    Me,
    Peer,
}
