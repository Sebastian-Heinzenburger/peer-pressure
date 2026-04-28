use domain::peer::PeerId;

#[derive(Debug, Clone)]
pub enum UserCommand {
    SendMessage { peer: PeerId, text: String },
    AddPeer { address: String },
    ConnectToPeer { peer: PeerId },
    Quit,
}
