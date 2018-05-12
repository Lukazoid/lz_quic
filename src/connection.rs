use errors::*;
use futures::{Future, Poll, Stream};
use {DataStream, NewDataStreams, Perspective, StreamMap, StreamMapEntry, StreamState};
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
        debug!(
            "created new connection with connection id {:?}",
            connection_id
        );

        Self {
            connection_id: connection_id,
            perspective: perspective,
            stream_map: Mutex::new(P::create_stream_map()),
        }
    }

    pub fn handshake(
        &self,
        crypto_stream: DataStream<P>,
    ) -> Box<Future<Item = (), Error = Error> + Send>
    where
        P::TlsSession: 'static,
    {
        Box::new(self.perspective.handshake(crypto_stream).map(|_| ()))
    }
}

impl<P: Perspective> Connection<P> {
    pub fn crypto_stream(&self) -> (StreamId, StreamMapEntry) {
        let mut stream_map = self.stream_map
            .lock()
            .expect("failed to obtain stream_map lock");

        stream_map.crypto_stream()
    }

    pub fn new_stream(&self) -> (StreamId, Arc<Mutex<StreamState>>) {
        let mut stream_map = self.stream_map
            .lock()
            .expect("failed to obtain stream_map lock");

        stream_map.next_outgoing_stream()
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub fn process_incoming_packets(&self) -> Result<()> {
        unimplemented!();
    }

    /// This also guarantees that the remote end acknowledged all of the stream
    /// data.
    pub fn flush_stream(&self, stream_id: StreamId) -> Poll<(), Error> {
        let stream_map_entry = {
            let stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            stream_map.get_stream(stream_id)?
        };

        match stream_map_entry {
            StreamMapEntry::Dead => {}
            StreamMapEntry::Live(stream_state) => {
                let mut stream_state = stream_state
                    .lock()
                    .expect("failed to obtain stream_state lock");

                self.queue_pending_writes(&mut stream_state);
            }
        }

        Ok(().into())
    }

    fn queue_pending_writes(&self, stream_state: &mut StreamState) {
        while let Some(pending_write) = stream_state.pop_pending_write() {
            // TODO LH actually push the bytes somewhere and send the frames
        }
    }

    pub fn forget_stream(&self, stream_id: StreamId) -> Result<()> {
        let stream_map_entry = {
            let mut stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            stream_map.forget_stream(stream_id)?
        };

        if let StreamMapEntry::Live(stream_state) = stream_map_entry {
            let mut stream_state = stream_state
                .lock()
                .expect("failed to obtain stream_state lock");

            self.queue_pending_writes(&mut stream_state);
        }

        Ok(())
    }
}
