use domain::message::MessageContent;
use domain::peer::PeerId;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct DisplayMessage {
    pub content: String,
    pub sent_by_me: bool,
    pub delivered: bool,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[derive(Default)]
pub struct TuiAppState {
    pub peers: Vec<PeerId>,
    pub connected_peers: HashSet<PeerId>,
    pub chats: HashMap<String, Vec<DisplayMessage>>,
    pub selected_peer: usize,
    pub input: String,
    pub input_mode: InputMode,
    pub should_quit: bool,
}

impl TuiAppState {
    pub fn new() -> Self {
        Self {
            peers: Vec::new(),
            connected_peers: HashSet::new(),
            chats: HashMap::new(),
            selected_peer: 0,
            input: String::new(),
            input_mode: InputMode::Normal,
            should_quit: false,
        }
    }

    pub fn selected_peer_id(&self) -> Option<&PeerId> {
        self.peers.get(self.selected_peer)
    }

    pub fn current_messages(&self) -> &[DisplayMessage] {
        self.selected_peer_id()
            .and_then(|p| self.chats.get(&p.to_string()))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn add_message(
        &mut self,
        peer: &PeerId,
        content: &MessageContent,
        sent_by_me: bool,
        delivered: bool,
    ) {
        let text = match content {
            MessageContent::Text(t) => t.clone(),
        };
        let msg = DisplayMessage {
            content: text,
            sent_by_me,
            delivered,
        };
        self.chats.entry(peer.to_string()).or_default().push(msg);
    }

    pub fn mark_all_delivered(&mut self, peer: &PeerId) {
        if let Some(messages) = self.chats.get_mut(&peer.to_string()) {
            for msg in messages.iter_mut() {
                if msg.sent_by_me {
                    msg.delivered = true;
                }
            }
        }
    }

    pub fn add_peer(&mut self, peer: PeerId) {
        if !self.peers.contains(&peer) {
            self.peers.push(peer);
        }
    }
}
