#[derive(Debug)]
pub struct ClientConfiguration;

impl ClientConfiguration {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ClientConfiguration {
    fn default() -> Self {
        ClientConfiguration {}
    }
}
