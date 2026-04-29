use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerAddress(Arc<str>);

impl Display for PeerAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PeerAddress {
    pub fn new(address: Arc<str>) -> Self {
        PeerAddress(address)
    }
}

pub type PeerId = PeerAddress;

#[derive(Debug, Clone)]
pub struct Peer {
    address: PeerAddress,
    // in the future also public_key: PublicKey
}

// pub trait PeerAddress:
//     Clone + Copy + Display + FromStr + Debug + Eq + PartialEq + Hash + Send + Sync
// {
// }

impl Peer {
    pub fn new(address: PeerAddress) -> Self {
        Self { address }
    }

    pub fn address(&self) -> PeerAddress {
        self.address.clone()
    }

    pub fn id(&self) -> PeerId {
        self.address.clone()
    }
}
