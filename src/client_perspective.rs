use errors::*;
use futures::{Future, IntoFuture};
use protocol::ServerId;
use rustls::ClientSession;
use std::sync::Arc;
use tokio_core::net::UdpSocket;
use tokio_rustls::{ClientConfigExt, TlsStream};
use webpki::DNSNameRef;
use {ClientConfiguration, DataStream, Perspective, StreamMap};

#[derive(Debug)]
pub struct ClientPerspective {
    server_id: Arc<ServerId>,
    client_configuration: Arc<ClientConfiguration>,
}

impl ClientPerspective {
    pub(crate) fn new(
        udp_socket: UdpSocket,
        client_configuration: ClientConfiguration,
        server_id: ServerId,
    ) -> Self {
        // TODO LH Do something with the UdpSocket
        Self {
            server_id: Arc::new(server_id),
            client_configuration: Arc::new(client_configuration),
        }
    }
}

impl Perspective for ClientPerspective {
    type TlsSession = ClientSession;
    type HandshakeFuture =
        Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>;

    fn handshake(&self, crypto_stream: DataStream<Self>) -> Self::HandshakeFuture {
        let connection_id = crypto_stream.connection().connection_id();
        trace!(
            "connection {:?}: performing TLS handshake from client to server {:?}",
            connection_id,
            self.server_id
        );

        let host = self.server_id.host();
        let tls_config = self.client_configuration.tls_config.clone();

        let when_connected = DNSNameRef::try_from_ascii_str(host)
            .map_err(|_| Error::from_kind(ErrorKind::HostIsNotAValidDomainName(host.to_owned())))
            .map(|dns_name| {
                let server_id_for_error = self.server_id.clone();
                let server_id_for_success = self.server_id.clone();
                tls_config
                    .connect_async(dns_name, crypto_stream)
                    .chain_err(move || {
                        ErrorKind::FailedToPerformTlsHandshakeWithServer(
                            server_id_for_error.host().to_owned(),
                        )
                    })
                    .map(move |x| {
                        info!(
                            "connection {:?}: performed TLS handshake from client to server {:?}",
                            connection_id, server_id_for_success
                        );
                        x
                    })
            })
            .into_future()
            .flatten();

        Box::new(when_connected)
    }

    fn tls_exporter_send_label() -> &'static str {
        "EXPORTER-QUIC client 1rtt"
    }

    fn tls_exporter_receive_label() -> &'static str {
        "EXPORTER-QUIC server 1rtt"
    }

    fn create_stream_map() -> StreamMap {
        StreamMap::new_client_stream_map()
    }
}
