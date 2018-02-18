use packets::PacketHeader;
use std::net::SocketAddr;
use chrono::{DateTime, UTC};
use bytes::Bytes;

/// An inbound packet before any decryption has taken place.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InboundPacket {
    pub source_address: SocketAddr,
    pub packet_header: PacketHeader,
    pub data: Bytes,
    pub received_at: DateTime<UTC>,
}
