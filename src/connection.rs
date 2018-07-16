use bytes::Bytes;
use crypto::CryptoState;
use errors::*;
use frames::StreamFrame;
use futures::{Async, Future, Poll};
use packets::{LongHeader, LongHeaderPacketType, OutgoingPacket, PacketHeader, PacketNumber,
              PartialPacketNumber};
use protocol::{ConnectionId, EncryptionLevel, FlowControl, Readable, StreamId, StreamType,
               TransportParameters, Version};
use rustls::Session;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use {DataStream, DequeueWriteResult, Perspective, StreamMap, StreamMapEntry, StreamState};

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
    incoming_flow_control: Mutex<FlowControl>,
    outgoing_flow_control: Mutex<FlowControl>,
    pending_stream_frames: Mutex<VecDeque<StreamFrame>>,
    remote_address: SocketAddr,
}

impl<P: Perspective + 'static> Connection<P> {
    pub fn new(
        local_connection_id: ConnectionId,
        remote_connection_id: ConnectionId,
        perspective: P,
        remote_address: SocketAddr,
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

        let incoming_flow_control =
            FlowControl::with_initial_max(perspective.max_incoming_data().into());

        let connection = Self {
            local_connection_id,
            remote_connection_id,
            perspective,
            stream_map: Mutex::new(P::create_stream_map()),
            aead_clear,
            state: Arc::new(Mutex::new(State::Initializing)),
            incoming_flow_control: Mutex::new(incoming_flow_control),
            outgoing_flow_control: Mutex::default(),
            pending_stream_frames: Mutex::default(),
            remote_address,
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

    fn should_transmit(&self, stream_frames: &VecDeque<StreamFrame>) -> bool {
        // TODO LH Write the actual logic over whether we should transmit
        !stream_frames.is_empty()
    }

    pub fn poll_try_transmit(&self) -> Poll<(), Error> {
        trace!("determining whether to transmit a new frame");

        let mut stream_frames = self.pending_stream_frames
            .lock()
            .expect("failed to lock pending_stream_frames");

        if self.should_transmit(&*stream_frames) {
            self.poll_transmit_stream_frames(&mut *stream_frames)
        } else {
            Ok(Async::NotReady)
        }
    }

    fn poll_transmit(&self) -> Poll<(), Error> {
        let mut stream_frames = self.pending_stream_frames
            .lock()
            .expect("failed to lock pending_stream_frames");

        self.poll_transmit_stream_frames(&mut *stream_frames)
    }

    fn poll_transmit_stream_frames(
        &self,
        stream_frames: &mut VecDeque<StreamFrame>,
    ) -> Poll<(), Error> {
        // TODO LH Actually write the stream frames
        stream_frames.clear();
        loop {
            let outgoing_packet = OutgoingPacket {
                destination_address: self.remote_address,
                packet_header: PacketHeader::Long(LongHeader {
                    packet_type: LongHeaderPacketType::Initial,
                    version: Version::DRAFT_IETF_08,
                    destination_connection_id: Some(self.remote_connection_id),
                    source_connection_id: Some(self.local_connection_id),
                    payload_length: 0,
                    partial_packet_number: PartialPacketNumber::from_packet_number(
                        PacketNumber::from(0u32),
                        PacketNumber::from(0u32),
                    )?,
                }),
                data: Bytes::new(),
                encryption_level: EncryptionLevel::Unencrypted,
            };

            if self.perspective
                .poll_send_packet(outgoing_packet)?
                .is_not_ready()
            {
                return Ok(Async::NotReady);
            }
        }
    }

    pub fn poll_process_incoming_packet(&self) -> Poll<(), Error> {
        trace!("checking for a new incoming packet");

        if let Async::Ready(incoming_packet) = self.perspective
            .poll_incoming_packet(self.local_connection_id())?
        {
            trace!("found new incoming packet");
            // TODO LH Do something with the packet
            return Ok(().into());
        }

        trace!("no more incoming packets");

        Ok(Async::NotReady)
    }

    pub fn poll_process_incoming_packets(&self) -> Poll<(), Error> {
        while self.poll_process_incoming_packet()?.is_ready() {}

        Ok(Async::NotReady)
    }

    pub fn poll_flush_stream(&self, stream_id: StreamId) -> Poll<(), Error> {
        let stream_map_entry = {
            let stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            stream_map.get_stream(stream_id)?
        };

        self.enqueue_stream_frames_from_stream_map_entry(&stream_map_entry);

        while self.poll_transmit()?.is_ready() {
            // TODO LH Stop when all frames for this stream have been sent
        }

        Ok(Async::NotReady)
    }

    /// This also guarantees that the remote end acknowledged all of the stream
    /// data.
    pub fn poll_flush_stream_and_wait_for_ack(&self, stream_id: StreamId) -> Poll<(), Error> {
        self.poll_flush_stream(stream_id)?;

        // TODO LH Wait for acknowledgement
        unimplemented!();

        Ok(().into())
    }

    fn enqueue_stream_frames(&self) {
        let stream_map_entries: Vec<_> = {
            let stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            let stream_map_entries = stream_map.get_streams();
            stream_map_entries.collect()
        };

        for stream_map_entry in stream_map_entries {
            self.enqueue_stream_frames_from_stream_map_entry(&stream_map_entry);
        }
    }

    fn enqueue_stream_frames_from_stream_map_entry(&self, stream_map_entry: &StreamMapEntry) {
        match stream_map_entry {
            StreamMapEntry::Dead => {}
            StreamMapEntry::Live(stream_state) => {
                let mut stream_state = stream_state
                    .lock()
                    .expect("failed to obtain stream_state lock");

                self.enqueue_stream_frames_from_stream_state(&mut stream_state);
            }
        }
    }

    fn enqueue_stream_frames_from_stream_state(&self, stream_state: &mut StreamState) {
        loop {
            let stream_id = stream_state.stream_id();

            trace!("stream {:?}: popping pending writes", stream_id);

            let stream_frame = match stream_state.dequeue_write() {
                DequeueWriteResult::DequeuedWrite {
                    offset,
                    data,
                    finished,
                } => {
                    debug!(
                        "stream {:?}: popped pending write at offset {} length {}",
                        stream_id,
                        offset,
                        data.len()
                    );

                    StreamFrame {
                        finished,
                        offset: offset.into(),
                        stream_id,
                        data,
                    }
                }
                DequeueWriteResult::NotReady => {
                    trace!("stream {:?}: no pending writes", stream_id);
                    break;
                }
            };

            let mut pending_stream_frames = self.pending_stream_frames
                .lock()
                .expect("failed to lock pending_stream_frames");

            pending_stream_frames.push_back(stream_frame);
        }
    }

    pub fn poll_forget_stream(&self, stream_id: StreamId) -> Poll<(), Error> {
        let stream_map_entry = {
            let mut stream_map = self.stream_map
                .lock()
                .expect("failed to obtain stream_map lock");
            stream_map.forget_stream(stream_id)?
        };

        self.enqueue_stream_frames_from_stream_map_entry(&stream_map_entry);

        Ok(().into())
    }

    pub fn handle_negotiated_session<S: Session>(&self, tls_session: &S) -> Result<()>
    where
        <<P as Perspective>::IncomingTransportMessageParameters as Readable>::Context: Default,
    {
        let transport_parameter_bytes = tls_session
            .get_quic_transport_parameters()
            .ok_or_else(|| ErrorKind::TransportParametersAreRequired)?;

        let transport_parameters: TransportParameters<
            P::IncomingTransportMessageParameters,
            P::RoleSpecificTransportParameters,
        > = TransportParameters::from_bytes(transport_parameter_bytes)?;

        unimplemented!()
    }

    pub fn incoming_flow_control(&self) -> &Mutex<FlowControl> {
        &self.incoming_flow_control
    }

    pub fn outgoing_flow_control(&self) -> &Mutex<FlowControl> {
        &self.outgoing_flow_control
    }
}
