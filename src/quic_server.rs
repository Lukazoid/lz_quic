use std::io::Result as IoResult;
use std::net::SocketAddr;
use tokio_core::reactor::Handle;
use tokio_core::net::UdpSocket;
use futures::Stream;
use quic_connection::QuicConnection;

pub struct QuicServer {

}

impl QuicServer {
    /// Create a new `QuicServer` bound to the specified address.
    pub fn bind(addr: &SocketAddr, handle: &Handle) -> IoResult<QuicServer> {
        let udp_socket = UdpSocket::bind(addr, handle)?;


        unimplemented!()
    }

    pub fn incoming(self) {
        unimplemented!()
    }
}