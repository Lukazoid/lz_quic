use ConnectionTerminationMode;
use rustls::ClientConfig as TlsConfig;
use std::sync::Arc;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use debugit::DebugIt;

lazy_static!{
    static ref DEFAULT_TLS_CONFIG: Arc<TlsConfig> = Arc::new(TlsConfig::new());
}

pub struct ClientConfiguration {
    pub connection_termination_mode: ConnectionTerminationMode,
    pub tls_config: Arc<TlsConfig>,
}

impl Debug for ClientConfiguration {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.debug_struct("ClientConfiguration")
            .field(
                "connection_termination_mode",
                &self.connection_termination_mode,
            )
            .field("tls_config", &DebugIt(&self.tls_config))
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
        }
    }
}
