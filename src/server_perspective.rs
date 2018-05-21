use errors::*;
use futures::Future;
use rustls::ServerSession;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_rustls::{ServerConfigExt, TlsStream};
use {DataStream, Perspective, ServerConfiguration, StreamMap};

#[derive(Debug)]
pub struct ServerPerspective {
    client_address: SocketAddr,
    server_configuration: Arc<ServerConfiguration>,
}

impl ServerPerspective {
    pub(crate) fn new(
        client_address: SocketAddr,
        server_configuration: Arc<ServerConfiguration>,
    ) -> Self {
        Self {
            client_address,
            server_configuration,
        }
    }
}

impl Perspective for ServerPerspective {
    type TlsSession = ServerSession;
    type HandshakeFuture =
        Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>;
        
    fn handshake(&self, crypto_stream: DataStream<Self>) -> Self::HandshakeFuture {
        let connection_id = crypto_stream.connection().connection_id();
        trace!(
            "connection {:?}: performing TLS handshake from server to client {:?}",
            connection_id,
            self.client_address
        );

        let client_address_for_error = self.client_address;
        let client_address_for_success = self.client_address;
        let when_connected = self.server_configuration
            .tls_config
            .accept_async(crypto_stream)
            .chain_err(move || {
                ErrorKind::FailedToPerformTlsHandshakeWithClient(client_address_for_error)
            })
            .map(move |x| {
                info!(
                    "connection {:?}: performed TLS handshake from server to client {:?}",
                    connection_id, client_address_for_success
                );
                x
            });

        Box::new(when_connected)
    }

    fn tls_exporter_send_label() -> &'static str {
        "EXPORTER-QUIC server 1rtt"
    }

    fn tls_exporter_receive_label() -> &'static str {
        "EXPORTER-QUIC client 1rtt"
    }

    fn create_stream_map() -> StreamMap {
        StreamMap::new_server_stream_map()
    }
}
