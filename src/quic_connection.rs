use quic_connection_id::QuicConnectionId;
use quic_version::QuicVersion;
use std::net::SocketAddr;
use std::collections::HashSet;

#[derive(Debug)]
pub struct QuicConnection {
    connection_id: QuicConnectionId,
    socket_address: SocketAddr,
    supported_versions: HashSet<QuicVersion>,
}