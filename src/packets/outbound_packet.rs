use std::net::SocketAddr;
use packets::PublicHeader;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutboundPacket {
    pub destination_address: SocketAddr,
    pub public_header: PublicHeader,
    pub data: Vec<u8>,
}

