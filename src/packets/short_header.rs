use super::PartialPacketNumber;
use protocol::ConnectionId;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ShortHeader {
    pub key_phase: bool,
    pub destination_connection_id: Option<ConnectionId>,
    pub partial_packet_number: PartialPacketNumber,
}
