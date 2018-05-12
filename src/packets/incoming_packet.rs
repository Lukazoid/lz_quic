use bytes::Bytes;
use chrono::{DateTime, UTC};
use packets::PacketHeader;
use std::net::SocketAddr;

/// An incoming packet before any decryption has taken place.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IncomingPacket {
    pub source_address: SocketAddr,
    pub packet_header: PacketHeader,
    pub data: Bytes,
    pub received_at: DateTime<UTC>,
}
