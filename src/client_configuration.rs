use ConnectionTerminationMode;

#[derive(Debug)]
pub struct ClientConfiguration{
    pub connection_termination_mode: ConnectionTerminationMode,
}

impl ClientConfiguration {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ClientConfiguration {
    fn default() -> Self {
        ClientConfiguration {
            connection_termination_mode: ConnectionTerminationMode::Explicit,
        }
    }
}
