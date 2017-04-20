use connection_id::ConnectionId;
use version::Version;


#[derive(Debug, Clone)]
pub struct VersionNegotiationPacket {
    pub connection_id: ConnectionId,
    pub supported_versions: Vec<Version>,
}
