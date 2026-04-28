use crate::app::TuiAppState;
use application::events::AppEvent;

pub fn handle(app: &mut TuiAppState, event: AppEvent) {
    match event {
        AppEvent::PeerAdded(peer) => {
            app.add_peer(peer);
        }
        AppEvent::PeerConnected(peer) => {
            app.connected_peers.insert(peer.clone());
            app.add_peer(peer);
        }
        AppEvent::PeerDisconnected(peer) => {
            app.connected_peers.remove(&peer);
        }
        AppEvent::MessageSent {
            peer,
            content,
            delivered,
        } => {
            app.add_message(&peer, &content, true, delivered);
        }
        AppEvent::MessageReceived { peer, content } => {
            app.add_message(&peer, &content, false, true);
            app.add_peer(peer);
        }
        AppEvent::MessagesDelivered { peer } => {
            app.mark_all_delivered(&peer);
        }
    }
}
