#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct QuicServerId {
    host: String,
    port: u16,
}

impl QuicServerId {
    fn new(host: String, port: u16) -> QuicServerId {
        QuicServerId {
            host: host,
            port: port,
        }
    }
}