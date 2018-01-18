use errors::*;
use futures::{Future, Stream};
use {DataStream, NewDataStreams, Perspective};
use protocol::{ConnectionId, StreamId};
use tokio_core::net::UdpFramed;
use tokio_io::codec::Framed;
use packets::{PacketCodec, PacketDispatcher};
use std::sync::Arc;
use tokio_rustls::TlsStream;

/// The connection exists so a single client-server connection may span multiple physical connections.
#[derive(Debug)]
pub struct Connection<P: Perspective> {
    connection_id: ConnectionId,
    perspective: P
}

impl<P: Perspective + 'static> Connection<P> {
    pub fn new(connection_id: ConnectionId, perspective: P) -> Self {
        debug!("created new connection with connection id {:?}", connection_id);

        Self {
            connection_id: connection_id,
            perspective: perspective,
        }
    }

    pub fn handshake(&self, crypto_stream: DataStream<P>) -> Box<Future<Item = (), Error = Error> + Send> where P::TlsSession: 'static {
        Box::new(self.perspective.handshake(crypto_stream)
            .map(|_|()))
    }

    pub fn new_stream_id(&self) -> StreamId {
        unimplemented!()
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}
