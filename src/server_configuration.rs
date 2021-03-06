use debugit::DebugIt;
use rustls::{NoClientAuth, ServerConfig as TlsConfig};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;
use ConnectionTerminationMode;

lazy_static! {
    static ref DEFAULT_TLS_CONFIG: Arc<TlsConfig> = Arc::new(TlsConfig::new(NoClientAuth::new()));
}

pub struct ServerConfiguration {
    pub connection_termination_mode: ConnectionTerminationMode,
    pub tls_config: Arc<TlsConfig>,
    pub max_incoming_data_per_stream: u32,
    pub max_incoming_data_per_connection: u32,
}

impl Debug for ServerConfiguration {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("ServerConfiguration")
            .field(
                "connection_termination_mode",
                &self.connection_termination_mode,
            )
            .field("tls_config", &DebugIt(&self.tls_config))
            .field(
                "max_incoming_data_per_stream",
                &self.max_incoming_data_per_stream,
            )
            .field(
                "max_incoming_data_per_connection",
                &self.max_incoming_data_per_connection,
            )
            .finish()
    }
}

impl ServerConfiguration {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for ServerConfiguration {
    fn default() -> Self {
        ServerConfiguration {
            connection_termination_mode: ConnectionTerminationMode::Explicit,
            tls_config: DEFAULT_TLS_CONFIG.clone(),
            max_incoming_data_per_stream: 8192,
            max_incoming_data_per_connection: 65536,
        }
    }
}
