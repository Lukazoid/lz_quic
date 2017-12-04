#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerId {
    host: String,
    port: u16,
}

impl ServerId {
    pub fn new(host: String, port: u16) -> ServerId {
        ServerId {
            host: host,
            port: port,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}
