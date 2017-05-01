use std::net::SocketAddr;
use packets::public_header::PublicHeader;

#[derive(Debug)]
pub struct OutboundPacket {
    pub destination_address: SocketAddr,
    pub public_header: PublicHeader,
    pub data: Vec<u8>,
}

