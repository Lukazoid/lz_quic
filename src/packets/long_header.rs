use super::{LongHeaderPacketType, PartialPacketNumber};
use protocol::{ConnectionId, Version};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LongHeader {
    pub packet_type: LongHeaderPacketType,
    pub version: Version,
    pub destination_connection_id: Option<ConnectionId>,
    pub source_connection_id: Option<ConnectionId>,
    pub payload_length: u64,
    pub partial_packet_number: PartialPacketNumber,
}
