use quic_connection_id::QuicConnectionId;

#[derive(Debug, Clone)]
pub struct PublicResetPacket {
    pub connection_id: QuicConnectionId,
}