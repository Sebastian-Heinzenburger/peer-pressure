use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use domain::peer::{PeerAddress, PeerId};
use std::sync::Arc;

pub struct PeerDisconnected {
    event_sender: Arc<dyn EventSender>,
}

impl PeerDisconnected {
    pub fn new(event_sender: Arc<dyn EventSender>) -> Self {
        PeerDisconnected { event_sender }
    }

    pub async fn execute(&self, addr: &PeerAddress) {
        let peer_id = PeerId::from(addr.clone());
        self.event_sender
            .send(AppEvent::PeerDisconnected(peer_id))
            .await;
    }
}
