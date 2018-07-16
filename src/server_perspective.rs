use errors::*;
use futures::{Future, IntoFuture, Poll};
use packets::{IncomingPacket, OutgoingPacket};
use protocol::{ClientHelloMessageParameters, ConnectionId, EncryptedExtensionsMessageParameters,
               Role, ServerSpecificTransportParameters, TransportParameters, Version, Writable};
use rustls::quic::{QuicExt, ServerQuicExt};
use rustls::ServerSession;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_rustls::{self, TlsStream};
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

    fn build_transport_parameters(
        &self,
    ) -> TransportParameters<EncryptedExtensionsMessageParameters, ServerSpecificTransportParameters>
    {
        TransportParameters {
            message_parameters: EncryptedExtensionsMessageParameters {
                negotiated_version: Version::DRAFT_IETF_08,
                supported_versions: hashset![Version::DRAFT_IETF_08],
            },
            initial_max_stream_data: self.server_configuration.max_incoming_data_per_stream,
            initial_max_data: self.server_configuration.max_incoming_data_per_connection,
            idle_timeout: 10,
            initial_max_bidi_streams: None,
            initial_max_uni_streams: None,
            max_packet_size: Some(65527),
            ack_delay_exponent: None,
            disable_migration: false,
            role_specific_transport_parameters: ServerSpecificTransportParameters {
                stateless_reset_token: None,
                preferred_address: None,
            },
        }
    }
}

impl Perspective for ServerPerspective {
    type TlsSession = ServerSession;
    type HandshakeFuture =
        Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>;
    type IncomingTransportMessageParameters = ClientHelloMessageParameters;
    type RoleSpecificTransportParameters = ServerSpecificTransportParameters;

    fn handshake(&self, crypto_stream: DataStream<Self>) -> Self::HandshakeFuture {
        let connection_description = crypto_stream.connection().description();
        trace!(
            "connection {}: performing TLS handshake from server to client {:?}",
            connection_description,
            self.client_address
        );

        let client_address_for_error = self.client_address;
        let client_address_for_success = self.client_address;

        let quic_transport_parameters = self.build_transport_parameters();

        let when_connected = quic_transport_parameters
            .bytes_vec()
            .map(|quic_transport_parameters| {
                ServerSession::new_quic(
                    &self.server_configuration.tls_config,
                    quic_transport_parameters,
                )
            })
            .map(|server_session| {
                tokio_rustls::accept_async_with_session(crypto_stream, server_session).chain_err(
                    move || {
                        ErrorKind::FailedToPerformTlsHandshakeWithClient(client_address_for_error)
                    },
                )
            })
            .into_future()
            .flatten()
            .and_then(move |tls_stream| {
                info!(
                    "connection {}: performed TLS handshake from server to client {:?}",
                    connection_description, client_address_for_success
                );

                {
                    let (stream, session) = tls_stream.get_ref();
                    stream.connection().handle_negotiated_session(session)?;
                }

                Ok(tls_stream)
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

    fn poll_send_packet(&self, packet: OutgoingPacket) -> Poll<(), Error> {
        unimplemented!()
    }

    fn role() -> Role {
        Role::Server
    }

    fn max_incoming_data_per_stream(&self) -> u32 {
        self.server_configuration.max_incoming_data_per_stream
    }

    fn max_incoming_data(&self) -> u32 {
        self.server_configuration.max_incoming_data_per_connection
    }
}
