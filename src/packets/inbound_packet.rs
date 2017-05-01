use packets::public_header::PublicHeader;
use std::net::SocketAddr;
use chrono::{DateTime, UTC};

#[derive(Debug)]
pub struct InboundPacket {
    pub source_address: SocketAddr,
    pub public_header: PublicHeader,
    pub data: Vec<u8>,
    pub received_at: DateTime<UTC>,
}

