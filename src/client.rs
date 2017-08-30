use errors::*;
use ClientConfiguration;
use Session;
use session;
use rand::OsRng;
use futures::{self, Future, IntoFuture};
use futures::future::BoxFuture;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio_core::reactor::Handle;
use tokio_core::net::UdpSocket;
use protocol::{ConnectionId, ServerId};

pub struct Client {
    session: Session,
}

fn bind_udp_socket(handle: &Handle) -> Result<UdpSocket> {
    let any_port = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0).into();
    UdpSocket::bind(&any_port, handle).chain_err(|| ErrorKind::FailedToBindUdpSocket)
}

fn generate_connection_id() -> Result<ConnectionId> {
    let mut rng = OsRng::new()
        .chain_err(|| {
            ErrorKind::FailedToCreateCryptographicRandomNumberGenerator
        })?;

    Ok(ConnectionId::generate(&mut rng))
}

impl Client {
    pub fn connect(
        server_address: SocketAddr,
        server_id: ServerId,
        client_configuration: ClientConfiguration,
        handle: &Handle,
    ) -> BoxFuture<Client, Error> {
        bind_udp_socket(handle)
            .and_then(|udp_socket| {
                let connection_id = generate_connection_id()?;

                let session = session::new_client_session(connection_id, udp_socket);

                Ok(Client { session: session })
            })
            .into_future()
            .boxed()
    }
}
