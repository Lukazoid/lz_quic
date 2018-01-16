use errors::*;
use {ClientConfiguration, ClientPerspective, NewClient, NewDataStream, NewDataStreams, Connection};
use rand::OsRng;
use futures::{Future, IntoFuture};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio_core::reactor::Handle;
use tokio_core::net::UdpSocket;
use protocol::{ConnectionId, ServerId};
use std::sync::Arc;

#[derive(Debug)]
pub struct Client {
    connection: Arc<Connection<ClientPerspective>>,
}

fn bind_udp_socket(handle: &Handle) -> Result<UdpSocket> {
    let any_port = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into();
    trace!("binding udp socket to {:?}", any_port);

    let udp_socket = UdpSocket::bind(&any_port, handle).chain_err(|| ErrorKind::FailedToBindUdpSocket)?;

    debug!("bound udp socket to {:?}", any_port);

    Ok(udp_socket)
}

fn generate_connection_id() -> Result<ConnectionId> {
    let mut rng = OsRng::new().chain_err(|| {
        ErrorKind::FailedToCreateCryptographicRandomNumberGenerator
    })?;

    Ok(ConnectionId::generate(&mut rng))
}

fn new_session(udp_socket: UdpSocket) -> Result<Connection<ClientPerspective>> {
    let connection_id = generate_connection_id()?;

    let client_perspective = ClientPerspective::new(udp_socket);

    let connection = Connection::new(connection_id, client_perspective);

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
            .and_then(new_session)
            .into_future()
            .and_then(|connection| {
                connection.handshake().map(|_| {
                    Client {
                        connection: Arc::new(connection),
                    }
                })
            });

        NewClient::new(Box::new(future))
    }

    pub fn open_stream(&self) -> NewDataStream<ClientPerspective> {
        NewDataStream::new(self.connection.clone())
    }

    pub fn incoming_streams(&self) -> NewDataStreams<ClientPerspective> {
        NewDataStreams::new(self.connection.clone())
    }
}
