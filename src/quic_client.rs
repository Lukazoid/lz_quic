use futures::{Future, IntoFuture};
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket as NetUdpSocket};
use std::io::{Result as IoResult, Error as IoError};
use tokio_core::net::UdpSocket;
use tokio_core::reactor::Handle;
use quic_codec::QuicCodec;

struct QuicClientHandshake {
    
}


pub struct QuicClient {
    
}

impl QuicClient {
    pub fn connect<B: ToSocketAddrs, C: ToSocketAddrs>(bind_addr: B,
                                                       addr: C,
                                                       handle: &Handle)
                                                       -> impl Future<Item=QuicClient, Error=IoError> {
        let bind_result = NetUdpSocket::bind(bind_addr)
            .and_then(|s| UdpSocket::from_socket(s, handle))
            .map(|s| s.framed(QuicCodec::default()))
            .into_future();

        // TODO LH Perform the crypto handshake

        bind_result.map(|s| {
            unimplemented!()
        })
    }
}