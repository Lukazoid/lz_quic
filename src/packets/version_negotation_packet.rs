use protocol::{ConnectionId, Version};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct VersionNegotationPacket {
    pub connection_id: ConnectionId,
    pub supported_versions: Vec<Version>,
}
