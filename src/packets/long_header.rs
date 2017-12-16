use super::{LongHeaderPacketType, PacketNumber};
use protocol::{ConnectionId, Version};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LongHeader {
    pub packet_type: LongHeaderPacketType,
    pub connection_id: ConnectionId,
    pub version: Version,
    pub packet_number: PacketNumber,
}