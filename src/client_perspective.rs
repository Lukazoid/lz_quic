use debugit::DebugIt;
use errors::*;
use futures::{Async, Future, IntoFuture, Poll, Stream};
use lz_shared_udp::{SharedUdpFramed, SharedUdpSocket};
use packets::{IncomingPacket, PacketCodec};
use protocol::{ConnectionId, Role, ServerId};
use rustls::quic::ClientQuicExt;
use rustls::ClientSession;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio_core::net::UdpSocket;
use tokio_rustls::{self, TlsStream};
use webpki::DNSNameRef;
use {AddressConnectionIds, ClientConfiguration, ConnectionMap, DataStream, Perspective, StreamMap};

#[derive(Debug)]
pub struct ClientPerspective {
    packets: DebugIt<SharedUdpFramed<Arc<UdpSocket>, PacketCodec>>,
    server_id: Arc<ServerId>,
    client_configuration: Arc<ClientConfiguration>,
    connection_map: RwLock<ConnectionMap>,
}

impl ClientPerspective {
    pub(crate) fn new(
        udp_socket: UdpSocket,
        client_configuration: ClientConfiguration,
        server_id: ServerId,
    ) -> Self {
        Self {
            packets: DebugIt(Arc::new(udp_socket).framed(PacketCodec::default())),
            server_id: Arc::new(server_id),
            client_configuration: Arc::new(client_configuration),
            connection_map: RwLock::new(ConnectionMap::with_capacity(1)),
        }
    }

    fn local_address(&self) -> SocketAddr {
        // TODO LH Add methods to SharedUdpFramed to get the Arc<UdpSocket>
        unimplemented!()
    }

    fn get_connection_id_for_incoming_packet(
        &self,
        incoming_packet: &IncomingPacket,
    ) -> Option<AddressConnectionIds> {
        if let Some(packet_connection_id) =
            incoming_packet.packet_header.destination_connection_id()
        {
            Some(AddressConnectionIds::Single(packet_connection_id))
        } else {
            let connection_map = self.connection_map
                .read()
                .expect("failed to lock connection_map");

            let local_address = self.local_address();

            connection_map.get_connection_id(local_address, incoming_packet.source_address)
        }
    }

    fn should_accept_incoming_packet(
        &self,
        connection_id: ConnectionId,
        incoming_packet: &IncomingPacket,
    ) -> bool {
        match self.get_connection_id_for_incoming_packet(incoming_packet) {
            Some(AddressConnectionIds::Single(matched_connection_id)) => {
                matched_connection_id == connection_id
            }
            Some(AddressConnectionIds::Multiple(matched_connection_ids)) => {
                matched_connection_ids.contains(&connection_id)
            }
            None => false,
        }
    }
}

impl Perspective for ClientPerspective {
    type TlsSession = ClientSession;
    type HandshakeFuture =
        Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>;

    fn handshake(&self, crypto_stream: DataStream<Self>) -> Self::HandshakeFuture {
        let connection_description = crypto_stream.connection().description();
        trace!(
            "connection {}: performing TLS handshake from client to server {:?}",
            connection_description,
            self.server_id
        );

        let host = self.server_id.host();
        let tls_config = self.client_configuration.tls_config.clone();

        let when_connected = DNSNameRef::try_from_ascii_str(host)
            .map_err(|_| Error::from_kind(ErrorKind::HostIsNotAValidDomainName(host.to_owned())))
            .map(|dns_name| {
                let server_id_for_error = self.server_id.clone();
                let server_id_for_success = self.server_id.clone();

                let quic_transport_parameters =
                    unimplemented!("populate the quic transport parameters");

                let client_session =
                    ClientSession::new_quic(&tls_config, dns_name, quic_transport_parameters);

                tokio_rustls::connect_async_with_session(crypto_stream, client_session)
                    .chain_err(move || {
                        ErrorKind::FailedToPerformTlsHandshakeWithServer(
                            server_id_for_error.host().to_owned(),
                        )
                    })
                    .map(move |x| {
                        info!(
                            "connection {}: performed TLS handshake from client to server {:?}",
                            connection_description, server_id_for_success
                        );
                        x
                    })
            })
            .into_future()
            .flatten();

        Box::new(when_connected)
    }

    fn client_connection_id(
        local_connection_id: ConnectionId,
        _remote_connection_id: ConnectionId,
    ) -> ConnectionId {
        local_connection_id
    }

    fn handshake_send_label() -> &'static str {
        "client hs"
    }

    fn handshake_receive_label() -> &'static str {
        "server hs"
    }

    fn update_secret_send_label() -> &'static str {
        "client 1rtt"
    }

    fn update_secret_receive_label() -> &'static str {
        "server 1rtt"
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

    fn poll_incoming_packet(&self, connection_id: ConnectionId) -> Poll<IncomingPacket, Error> {
        let mut packets_stream = self.packets
            .0
            .clone()
            .chain_err(move || ErrorKind::FailedToReadIncomingPacket(connection_id));

        loop {
            match packets_stream.poll()? {
                Async::NotReady => return Ok(Async::NotReady),
                Async::Ready(Some(incoming_packet)) => {
                    if self.should_accept_incoming_packet(connection_id, &incoming_packet) {
                        return Ok(Async::Ready(incoming_packet));
                    } else {
                        warn!("discarded packet");
                    }
                }
                Async::Ready(None) => unreachable!("the packets stream should never end"),
            }
        }
    }

    fn role() -> Role {
        Role::Client
    }

    fn max_incoming_data_per_stream(&self) -> u32 {
        self.client_configuration.max_incoming_data_per_stream
    }

    fn max_incoming_data(&self) -> u32 {
        self.client_configuration.max_incoming_data
    }
}
