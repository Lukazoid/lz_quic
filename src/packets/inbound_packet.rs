use packets::PublicHeader;
use std::net::SocketAddr;
use chrono::{DateTime, UTC};

/// An inbound packet before any decryption has taken place.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InboundPacket {
    pub source_address: SocketAddr,
    pub public_header: PublicHeader,
    pub data: Vec<u8>,
    pub received_at: DateTime<UTC>,
}

