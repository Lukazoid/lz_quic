use debugit::DebugIt;
use rustls::ClientConfig as TlsConfig;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use ConnectionTerminationMode;

lazy_static! {
    static ref DEFAULT_TLS_CONFIG: Arc<TlsConfig> = Arc::new(TlsConfig::new());
}

pub struct ClientConfiguration {
    pub connection_termination_mode: ConnectionTerminationMode,
    pub tls_config: Arc<TlsConfig>,
    pub max_incoming_data_per_stream: u32,
    pub max_incoming_data: u32,
}

impl Debug for ClientConfiguration {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("ClientConfiguration")
            .field(
                "connection_termination_mode",
                &self.connection_termination_mode,
            )
            .field("tls_config", &DebugIt(&self.tls_config))
            .field(
                "max_incoming_data_per_stream",
                &self.max_incoming_data_per_stream,
            )
            .field("max_incoming_data", &self.max_incoming_data)
            .finish()
    }
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
            tls_config: DEFAULT_TLS_CONFIG.clone(),
            max_incoming_data_per_stream: 8192,
            max_incoming_data: 65536,
        }
    }
}
