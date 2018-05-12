use bytes::Bytes;
use packets::PacketHeader;
use protocol::EncryptionLevel;
use std::net::SocketAddr;

/// An outgoing packet after any encryption has taken place.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutgoingPacket {
    pub destination_address: SocketAddr,
    pub packet_header: PacketHeader,
    pub data: Bytes,
    pub encryption_level: EncryptionLevel,
}
