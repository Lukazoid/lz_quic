use errors::*;
use {DataStream, Perspective, ServerConfiguration, StreamMap};
use rustls::ServerSession;
use tokio_rustls::{ServerConfigExt, TlsStream};
use futures::Future;
use std::net::SocketAddr;
use std::sync::Arc;

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

    fn handshake(
        &self,
        crypto_stream: DataStream<Self>,
    ) -> Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>
    {
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

    fn create_stream_map() -> StreamMap {
        StreamMap::new_server_stream_map()
    }
}
