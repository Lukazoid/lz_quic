use std::net::SocketAddr;
use packets::PacketHeader;
use protocol::EncryptionLevel;

/// An outbound packet after any encryption has taken place.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct OutboundPacket {
    pub destination_address: SocketAddr,
    pub packet_header: PacketHeader,
    pub data: Vec<u8>,
    pub encryption_level: EncryptionLevel,
}

