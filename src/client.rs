use errors::*;
use futures::{Future, IntoFuture};
use protocol::{ConnectionId, ServerId, StreamType};
use rand::OsRng;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Handle;
use {ClientConfiguration, ClientPerspective, Connection, DataStream, NewClient, NewDataStreams,
     SharedConnection};

#[derive(Debug)]
pub struct Client {
    connection: Arc<Connection<ClientPerspective>>,
}

fn bind_udp_socket(handle: &Handle) -> Result<UdpSocket> {
    let any_port = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into();
    trace!("binding udp socket to {:?}", any_port);

    let udp_socket =
        UdpSocket::bind(&any_port, handle).chain_err(|| ErrorKind::FailedToBindUdpSocket)?;

    debug!("bound udp socket to {:?}", any_port);

    Ok(udp_socket)
}

fn generate_connection_id() -> Result<ConnectionId> {
    let mut rng =
        OsRng::new().chain_err(|| ErrorKind::FailedToCreateCryptographicRandomNumberGenerator)?;

    Ok(ConnectionId::generate(&mut rng))
}

fn new_connection(
    server_id: ServerId,
    udp_socket: UdpSocket,
    client_configuration: ClientConfiguration,
) -> Result<Connection<ClientPerspective>> {
    let local_connection_id = generate_connection_id()?;
    let remote_connection_id = generate_connection_id()?;

    let client_perspective = ClientPerspective::new(udp_socket, client_configuration, server_id);

    let connection = Connection::new(
        local_connection_id,
        remote_connection_id,
        client_perspective,
    )?;

    Ok(connection)
}

impl Client {
    pub fn connect(
        server_address: SocketAddr,
        server_id: ServerId,
        client_configuration: ClientConfiguration,
        handle: &Handle,
    ) -> NewClient {
        let future = bind_udp_socket(handle)
            .and_then(|udp_socket| new_connection(server_id, udp_socket, client_configuration))
            .into_future()
            .and_then(|connection| {
                let connection = Arc::new(connection);
                let connection_copy = connection.clone();
                connection.handshake().map(|_| Client {
                    connection: connection_copy,
                })
            });

        NewClient::new(Box::new(future))
    }

    fn open_stream(&self, stream_type: StreamType) -> DataStream<ClientPerspective> {
        let (stream_id, stream_state) = self.connection.new_stream(stream_type);

        DataStream::new(stream_id, self.connection.clone(), stream_state)
    }

    pub fn open_bidirectional_stream(&self) -> DataStream<ClientPerspective> {
        self.open_stream(StreamType::Bidirectional)
    }

    pub fn open_unidirectional_stream(&self) -> DataStream<ClientPerspective> {
        self.open_stream(StreamType::Unidirectional)
    }

    pub fn incoming_streams(&self) -> NewDataStreams<ClientPerspective> {
        NewDataStreams::new(self.connection.clone())
    }
}
