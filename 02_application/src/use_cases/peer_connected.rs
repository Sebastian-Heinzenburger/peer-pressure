use crate::events::AppEvent;
use crate::ports::event_sender::EventSender;
use domain::peer::{PeerAddress, PeerId};
use std::sync::Arc;

pub struct PeerConnected {
    event_sender: Arc<dyn EventSender>,
}

impl PeerConnected {
    pub fn new(event_sender: Arc<dyn EventSender>) -> Self {
        PeerConnected { event_sender }
    }

    pub async fn execute(&self, address: PeerAddress) {
        let peer_id = PeerId::from(address.clone());
        self.event_sender
            .send(AppEvent::PeerConnected(peer_id))
            .await;
    }
}
