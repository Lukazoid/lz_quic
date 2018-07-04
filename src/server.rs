use errors::*;
use futures::future::Empty;
use futures::Stream;
use packets::PacketDispatcher;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_core::net::{UdpFramed, UdpSocket};
use tokio_core::reactor::Handle;
use {NewRemoteClients, ServerConfiguration};

#[derive(Debug)]
pub struct Server {
    packet_dispatcher: Arc<PacketDispatcher>,
    server_configuration: ServerConfiguration,
}

impl Server {
    pub fn bind(
        addr: SocketAddr,
        server_configuration: ServerConfiguration,
        handle: &Handle,
    ) -> Result<Self> {
        trace!("binding udp socket to {:?}", addr);

        let udp_socket =
            UdpSocket::bind(&addr, handle).chain_err(|| ErrorKind::FailedToBindToUdpSocket(addr))?;

        debug!("bound udp socket to {:?}", addr);

        Ok(Self {
            packet_dispatcher: Arc::new(PacketDispatcher::new(udp_socket)),
            server_configuration,
        })
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.packet_dispatcher.local_addr()
    }

    pub fn incoming(self) -> NewRemoteClients {
        unimplemented!()
        // stream::empty::<(), Error>()
        //     .map(move |_|{
        //         let connection_id = ConnectionId::generate(&mut ::rand::thread_rng());
        //         Connection::new_server(connection_id)
        //     })
        //     .boxed()
    }
}
