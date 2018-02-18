use std::net::SocketAddr;
use packets::PacketHeader;
use protocol::EncryptionLevel;
use bytes::Bytes;

/// An outgoing packet after any encryption has taken place.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutgoingPacket {
    pub destination_address: SocketAddr,
    pub packet_header: PacketHeader,
    pub data: Bytes,
    pub encryption_level: EncryptionLevel,
}

