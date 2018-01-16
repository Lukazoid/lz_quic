use errors::*;
use futures::{Future, Stream};
use {DataStream, NewDataStream, NewDataStreams, Perspective};
use protocol::ConnectionId;
use tokio_core::net::UdpFramed;
use tokio_io::codec::Framed;
use packets::{PacketCodec, PacketDispatcher};
use std::sync::Arc;


/// The connection exists so a single client-server connection may span multiple physical connections.
#[derive(Debug)]
pub struct Connection<P> {
    connection_id: ConnectionId,
    perspective: P,
}

impl<P> Drop for Connection<P> {
    fn drop(&mut self) {
        // TODO LH Inform the packet dispatcher that this connection has closed
    }
}

impl<P: Perspective> Connection<P> {
    pub fn new(connection_id: ConnectionId, perspective: P) -> Self {
        debug!("created new connection with connection id {:?}", connection_id);

        Self {
            connection_id: connection_id,
            perspective: perspective,
        }
    }

    pub fn handshake(&self) -> Box<Future<Item = (), Error = Error> + Send + Sync> {
        unimplemented!()
    }
}
