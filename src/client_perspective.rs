use errors::*;
use {ClientConfiguration, DataStream, Perspective};
use protocol::ServerId;
use tokio_core::net::UdpSocket;
use webpki::DNSNameRef;
use rustls::{ClientConfig as TlsConfig, ClientSession};
use tokio_rustls::{ClientConfigExt, TlsStream};
use futures::{Future, IntoFuture};
use std::sync::Arc;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use debugit::DebugIt;

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
    fn handshake(
        &self,
        crypto_stream: DataStream<Self>,
    ) -> Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>
    {
        let connection_id = crypto_stream.connection().connection_id();
        trace!(
            "performing TLS handshake from client {:?} to server {:?}",
            connection_id,
            self.server_id
        );

        let host = self.server_id.host();
        let tls_config = self.client_configuration.tls_config.clone();

        let when_connected = DNSNameRef::try_from_ascii_str(host)
            .map_err(|_| {
                Error::from_kind(ErrorKind::HostIsNotAValidDomainName(host.to_owned()))
            })
            .map(|dns_name| {
                let server_id_for_error = self.server_id.clone();
                let server_id_for_logging = self.server_id.clone();
                tls_config
                    .connect_async(dns_name, crypto_stream)
                    .chain_err(move || {
                        ErrorKind::FailedToPerformTlsHandshake(
                            server_id_for_error.host().to_owned(),
                        )
                    })
                    .map(move |x| {
                        info!(
                            "performed TLS handshake from client {:?} to server {:?}",
                            connection_id,
                            server_id_for_logging
                        );
                        x
                    })
            })
            .into_future()
            .flatten();

        Box::new(when_connected)
    }
}
