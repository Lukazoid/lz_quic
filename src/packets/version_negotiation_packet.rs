use protocol::{ConnectionId, Version};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VersionNegotiationPacket {
    pub destination_connection_id: Option<ConnectionId>,
    pub source_connection_id: Option<ConnectionId>,
    pub supported_versions: Vec<Version>,
}
