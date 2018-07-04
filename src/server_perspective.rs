use errors::*;
use futures::{Future, Poll};
use packets::IncomingPacket;
use protocol::{ConnectionId, Role};
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
        let connection_description = crypto_stream.connection().description();
        trace!(
            "connection {}: performing TLS handshake from server to client {:?}",
            connection_description,
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
                    "connection {}: performed TLS handshake from server to client {:?}",
                    connection_description, client_address_for_success
                );
                x
            });

        Box::new(when_connected)
    }

    fn client_connection_id(
        _local_connection_id: ConnectionId,
        remote_connection_id: ConnectionId,
    ) -> ConnectionId {
        remote_connection_id
    }

    fn handshake_send_label() -> &'static str {
        "server hs"
    }

    fn handshake_receive_label() -> &'static str {
        "client hs"
    }

    fn update_secret_send_label() -> &'static str {
        "server 1rtt"
    }

    fn update_secret_receive_label() -> &'static str {
        "client 1rtt"
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

    fn poll_incoming_packet(&self, connection_id: ConnectionId) -> Poll<IncomingPacket, Error> {
        unimplemented!()
    }

    fn role() -> Role {
        Role::Server
    }

    fn initial_max_incoming_data_per_stream(&self) -> u32 {
        self.server_configuration
            .initial_max_incoming_data_per_stream
    }

    fn initial_max_incoming_data(&self) -> u32 {
        self.server_configuration.initial_max_incoming_data
    }
}
