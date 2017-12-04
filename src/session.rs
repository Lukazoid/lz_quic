use errors::*;
use futures::{Future, Stream};
use {DataStream, NewDataStream, NewDataStreams, Perspective};
use handshake::HandshakeCodec;
use protocol::ConnectionId;
use tokio_core::net::UdpFramed;
use tokio_io::codec::Framed;
use packets::{PacketCodec, PacketDispatcher};
use std::sync::Arc;


/// The session exists so a single client-server session may span multiple physical connections.
#[derive(Debug)]
pub struct Session<P> {
    connection_id: ConnectionId,
    perspective: P,
}

impl<P> Drop for Session<P> {
    fn drop(&mut self) {
        // TODO LH Inform the packet dispatcher that this connection has closed
    }
}

impl<P: Perspective> Session<P> {
    pub fn new(connection_id: ConnectionId, perspective: P) -> Self {
        debug!("created new session with connection id {:?}", connection_id);

        Self {
            connection_id: connection_id,
            perspective: perspective,
        }
    }

    fn crypto_stream(&self) -> Framed<DataStream<P>, HandshakeCodec> {
        unimplemented!()
    }

    pub fn handshake(&self) -> Box<Future<Item = (), Error = Error> + Send + Sync> {
        unimplemented!()
    }
}
