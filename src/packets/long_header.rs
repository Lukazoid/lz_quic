use super::{LongHeaderPacketType, PartialPacketNumber};
use protocol::{ConnectionId, VarInt, Version};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LongHeader {
    pub packet_type: LongHeaderPacketType,
    pub version: Version,
    pub destination_connection_id: Option<ConnectionId>,
    pub source_connection_id: Option<ConnectionId>,
    pub payload_length: VarInt,
    pub partial_packet_number: PartialPacketNumber,
}
