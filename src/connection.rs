use bytes::Bytes;
use crypto;
use crypto::CryptoState;
use errors::*;
use futures::{Async, Future, Poll, Stream};
use packets::{PacketCodec, PacketDispatcher};
use protocol::{ConnectionId, FlowControl, StreamId, StreamType};
use rustls::Session;
use std::sync::{Arc, Mutex};
use tokio_core::net::UdpFramed;
use tokio_io::codec::Framed;
use tokio_rustls::TlsStream;
use {DataStream, NewDataStreams, Perspective, StreamMap, StreamMapEntry, StreamState};

#[derive(Debug)]
struct AeadPair {
    write: CryptoState,
    read: CryptoState,
}

#[derive(Debug)]
enum State {
    Initializing,
    Established { aead_protected: AeadPair },
}

/// The connection exists so a single client-server connection may span multiple physical connections.
#[derive(Debug)]
pub struct Connection<P: Perspective> {
    local_connection_id: ConnectionId,
    remote_connection_id: ConnectionId,
    perspective: P,
    stream_map: Mutex<StreamMap>,
    aead_clear: AeadPair,
    state: Arc<Mutex<State>>,
    local_flow_control: Mutex<FlowControl>,
    remote_flow_control: Mutex<FlowControl>,
}

impl<P: Perspective + 'static> Connection<P> {
    pub fn new(
        local_connection_id: ConnectionId,
        remote_connection_id: ConnectionId,
        perspective: P,
    ) -> Result<Self> {
        let client_connection_id =
            P::client_connection_id(local_connection_id, remote_connection_id);

        let write_clear =
            CryptoState::for_handshake(client_connection_id, P::handshake_send_label())?;

        let read_clear =
            CryptoState::for_handshake(client_connection_id, P::handshake_receive_label())?;

        let aead_clear = AeadPair {
            write: write_clear,
            read: read_clear,
        };

        let local_flow_control =
            FlowControl::with_initial_max(perspective.max_incoming_data().into());

        let connection = Self {
            local_connection_id,
            remote_connection_id,
            perspective,
            stream_map: Mutex::new(P::create_stream_map()),
            aead_clear,
            state: Arc::new(Mutex::new(State::Initializing)),
            local_flow_control: Mutex::new(local_flow_control),
            remote_flow_control: Mutex::default(),
        };

        debug!("created new connection {}", connection.description());

        Ok(connection)
    }

    pub fn description(&self) -> String {
        format!(
            "[{:?}] connection {:?}->{:?}",
            P::role(),
            self.local_connection_id,
            self.remote_connection_id
        )
    }

    pub fn handshake(
        &self,
        crypto_stream: DataStream<P>,
    ) -> impl Future<Item = (), Error = Error> + Send
    where
        P::TlsSession: 'static,
    {
        let state = self.state.clone();

        self.perspective
            .handshake(crypto_stream)
            .and_then(move |tls_stream| {
                let (_, session) = tls_stream.get_ref();

                let crypto_write = CryptoState::from_tls(session, P::tls_exporter_send_label())?;
                let crypto_read = CryptoState::from_tls(session, P::tls_exporter_receive_label())?;

                let mut state = state.lock().expect("failed to lock state");

                *state = State::Established {
                    aead_protected: AeadPair {
                        write: crypto_write,
                        read: crypto_read,
                    },
                };

                Ok(())
            })
    }
}

impl<P: Perspective> Connection<P> {
    pub fn crypto_stream(&self) -> (StreamId, StreamMapEntry) {
        let mut stream_map = self.stream_map
            .lock()
            .expect("failed to obtain stream_map lock");

        stream_map.crypto_stream()
    }

    pub fn new_stream(&self, stream_type: StreamType) -> (StreamId, Arc<Mutex<StreamState>>) {
        let mut stream_map = self.stream_map
            .lock()
            .expect("failed to obtain stream_map lock");

        // TODO LH Use the transport parameters to determine how much can be sent on each newly created stream
        let max_outgoing_data_per_stream: u32 = unimplemented!();

        stream_map.next_outgoing_stream(
            stream_type,
            self.perspective.max_incoming_data_per_stream().into(),
            max_outgoing_data_per_stream.into(),
        )
    }

    pub fn local_connection_id(&self) -> ConnectionId {
        self.local_connection_id
    }

    pub fn remote_connection_id(&self) -> ConnectionId {
        self.remote_connection_id
    }

    pub fn poll_process_incoming_packets(&self) -> Poll<(), Error> {
        // TODO LH Eventually handle remote connection termination

        while let Async::Ready(incoming_packet) = self.perspective
            .poll_incoming_packet(self.local_connection_id())?
        {
            unimplemented!()
        }

        Ok(Async::NotReady)
    }

    pub fn poll_flush_stream(&self, stream_id: StreamId) -> Poll<(), Error> {
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

                self.enqueue_pending_writes(&mut stream_state);
            }
        }

        Ok(().into())
    }

    /// This also guarantees that the remote end acknowledged all of the stream
    /// data.
    pub fn poll_flush_stream_and_wait_for_ack(&self, stream_id: StreamId) -> Poll<(), Error> {
        self.poll_flush_stream(stream_id)?;

        // TODO LH Wait for acknowledgement
        Ok(().into())
    }

    fn enqueue_pending_writes(&self, stream_state: &mut StreamState) {
        while let Some(pending_write) = stream_state.dequeue_write() {
            debug!("popped pending write");
            // self.perspective
            //     .queue_write(stream_state.stream_id(), pending_write);
        }
    }

    pub fn poll_forget_stream(&self, stream_id: StreamId) -> Poll<(), Error> {
        let stream_map_entry = {
            let mut stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            stream_map.get_stream(stream_id)?
        };

        if let StreamMapEntry::Live(stream_state) = stream_map_entry {
            let mut stream_state = stream_state
                .lock()
                .expect("failed to obtain stream_state lock");

            self.enqueue_pending_writes(&mut stream_state);
        }

        {
            let mut stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            stream_map.forget_stream(stream_id)?;
        }

        Ok(().into())
    }
}
