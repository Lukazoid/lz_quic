use errors::*;
use futures::{Future, Stream};
use futures::stream::{self, BoxStream};
use connection_id::ConnectionId;
use data_stream::DataStream;
use tokio_core::net::UdpSocket;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
enum Perspective {
    Server {
        // TODO LH Eventually it won't be the UdpSocket we are sharing but something which dispatches to the correct sessions
        udp_socket: Arc<RwLock<UdpSocket>>,
    },
    Client {
        // TODO LH Eventually it won't be the UdpSocket we are sharing but something which dispatches packets to the client session
        udp_socket: UdpSocket,
    },
}

/// The session exists so a single client-server session may span multiple physical connections.
#[derive(Debug)]
pub struct Session {
    connection_id: ConnectionId,
    perspective: Perspective,
}

impl Session {
    pub fn new_client(connection_id: ConnectionId, udp_socket: UdpSocket) -> Self {
        Self::new(connection_id,
            Perspective::Client { udp_socket: udp_socket })
    }

    pub fn new_server(connection_id: ConnectionId, udp_socket: Arc<RwLock<UdpSocket>>) -> Self {
        Self::new(connection_id,
            Perspective::Server { udp_socket: udp_socket })
    }

    fn new(connection_id: ConnectionId, perspective: Perspective) -> Self {
        Self {
            connection_id: connection_id,
            perspective: perspective,
        }
    }

    pub fn open_stream(&self) -> DataStream {
        unimplemented!()
    }

    pub fn incoming_streams(&self) -> BoxStream<DataStream, Error> {
        stream::empty::<DataStream, Error>().boxed()
    }
}

