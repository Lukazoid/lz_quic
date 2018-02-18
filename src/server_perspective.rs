use errors::*;
use {DataStream, Perspective, StreamMap};
use tokio_core::net::UdpSocket;
use rustls::ServerSession;
use tokio_rustls::TlsStream;
use futures::Future;

#[derive(Debug)]
pub struct ServerPerspective {
}

impl ServerPerspective {
    pub(crate) fn new(udp_socket: UdpSocket) -> Self {
        Self {}
    }
}

impl Perspective for ServerPerspective {
    type TlsSession = ServerSession;

    fn handshake(
        &self,
        crypto_stream: DataStream<Self>,
    ) -> Box<Future<Item = TlsStream<DataStream<Self>, Self::TlsSession>, Error = Error> + Send>
    {
        unimplemented!()
    }

    fn create_stream_map() -> StreamMap {
        StreamMap::new_server_stream_map()
    }
}
