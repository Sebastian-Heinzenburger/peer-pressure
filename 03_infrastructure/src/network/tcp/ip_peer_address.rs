use domain::peer::PeerAddress;
use std::net::{AddrParseError, IpAddr};
use std::str::FromStr;

pub struct IpPeerAddress(IpAddr);

impl IpPeerAddress {
    pub fn ip(&self) -> IpAddr {
        self.0
    }
}

impl TryFrom<&PeerAddress> for IpPeerAddress {
    type Error = AddrParseError;

    fn try_from(value: &PeerAddress) -> Result<Self, Self::Error> {
        Ok(IpPeerAddress(IpAddr::from_str(&value.to_string())?))
    }
}
