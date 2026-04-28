use domain::message::MessageContent;
use domain::peer::PeerId;

#[derive(Debug, Clone, PartialEq)]
pub enum AppEvent {
    PeerAdded(PeerId),
    PeerConnected(PeerId),
    PeerDisconnected(PeerId),
    MessageSent {
        peer: PeerId,
        content: MessageContent,
        delivered: bool,
    },
    MessageReceived {
        peer: PeerId,
        content: MessageContent,
    },
    MessagesDelivered {
        peer: PeerId,
    },
}
