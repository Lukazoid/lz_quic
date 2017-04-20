use connection_id::ConnectionId;

#[derive(Debug, Clone)]
pub struct PublicResetPacket {
    pub connection_id: ConnectionId,
}