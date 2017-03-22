use quic_connection_id::QuicConnectionId;
use quic_version::QuicVersion;


#[derive(Debug, Clone)]
pub struct VersionNegotiationPacket {
    pub connection_id: QuicConnectionId,
    pub supported_versions: Vec<QuicVersion>,
}
