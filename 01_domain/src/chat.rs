use crate::message::ChatMessage;
use crate::peer::PeerId;

#[derive(Clone)]
pub struct Chat {
    pub peer: PeerId,
    pub messages: Vec<ChatMessage>,
}

impl Chat {
    pub fn new(peer: PeerId, messages: Vec<ChatMessage>) -> Self {
        Self { peer, messages }
    }

    pub fn create(peer: PeerId) -> Self {
        Self::new(peer, Vec::new())
    }

    pub fn add_message(&mut self, msg: ChatMessage) {
        self.messages.push(msg);
    }
}
