#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerId {
    host: String,
    port: u16,
}

impl ServerId {
    fn new(host: String, port: u16) -> ServerId {
        ServerId {
            host: host,
            port: port,
        }
    }
}
