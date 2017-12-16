use protocol::ConnectionId;
use super::PartialPacketNumber;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ShortHeader {
    pub connection_id: Option<ConnectionId>,
    pub partial_packet_number: PartialPacketNumber,
    pub key_phase: bool,
}
