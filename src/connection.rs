use errors::*;
use futures::{Future, Stream, Poll};
use {DataStream, NewDataStreams, Perspective, StreamState, StreamMap};
use protocol::{ConnectionId, StreamId};
use tokio_core::net::UdpFramed;
use tokio_io::codec::Framed;
use packets::{PacketCodec, PacketDispatcher};
use std::sync::{Arc, Mutex};
use tokio_rustls::TlsStream;
use bytes::Bytes;

/// The connection exists so a single client-server connection may span multiple physical connections.
#[derive(Debug)]
pub struct Connection<P: Perspective> {
    connection_id: ConnectionId,
    perspective: P,
    stream_map: Mutex<StreamMap>,
}

impl<P: Perspective + 'static> Connection<P> {
    pub fn new(connection_id: ConnectionId, perspective: P) -> Self {
        debug!("created new connection with connection id {:?}", connection_id);

        Self {
            connection_id: connection_id,
            perspective: perspective,
            stream_map: Mutex::new(P::create_stream_map())
        }
    }

    pub fn handshake(&self, crypto_stream: DataStream<P>) -> Box<Future<Item = (), Error = Error> + Send> where P::TlsSession: 'static {
        Box::new(self.perspective.handshake(crypto_stream)
            .map(|_|()))
    }
}

impl<P: Perspective> Connection<P> {
    pub fn crypto_stream(&self) -> (StreamId, Arc<Mutex<StreamState>>) {
        let mut stream_map = self.stream_map.lock().expect("failed to obtain stream_map lock");

        stream_map.crypto_stream()
    }

    pub fn new_stream(&self) -> (StreamId, Arc<Mutex<StreamState>>) {
        let mut stream_map = self.stream_map.lock().expect("failed to obtain stream_map lock");

        stream_map.next_outgoing_stream()
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub fn process_incoming_packets(&self) -> Result<()> {
        unimplemented!();
    }

    pub fn flush_stream(&self, stream_id: StreamId) -> Poll<(), Error> {
        unimplemented!()
    }

    pub fn forget_stream(&self, stream_id: StreamId) -> Result<()> {
        // TODO LH Actually forget about the stream
        Ok(())
    }
}