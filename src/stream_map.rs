use errors::*;
use StreamState;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use protocol::{Perspective, StreamId};
use std::sync::Arc;
use futures::sync::mpsc::{self, Receiver, Sender};
use bytes::Bytes;
use std::sync::Mutex;

#[derive(Debug)]
pub struct StreamMap {
    opened_streams: HashMap<StreamId, Arc<Mutex<StreamState>>>,
    next_outgoing_stream_id: StreamId,
    next_incoming_stream_id: StreamId,
}

fn new_stream(
    opened_streams: &mut HashMap<StreamId, Arc<Mutex<StreamState>>>,
    next_stream_id: &mut StreamId,
) -> (StreamId, Arc<Mutex<StreamState>>) {
    let id = *next_stream_id;
    *next_stream_id = next_stream_id.next();

    let new_stream_state = Arc::new(Mutex::new(StreamState::new(id)));
    opened_streams.insert(id, new_stream_state.clone());

    (id, new_stream_state)
}

impl StreamMap {
    pub fn new_client_stream_map() -> Self {
        StreamMap {
            opened_streams: HashMap::new(),
            next_outgoing_stream_id: StreamId::first_client_stream_id(),
            next_incoming_stream_id: StreamId::first_server_stream_id(),
        }
    }

    pub fn new_server_stream_map() -> Self {
        StreamMap {
            opened_streams: HashMap::new(),
            next_outgoing_stream_id: StreamId::first_server_stream_id(),
            next_incoming_stream_id: StreamId::first_client_stream_id(),
        }
    }

    pub fn next_outgoing_stream(&mut self) -> (StreamId, Arc<Mutex<StreamState>>) {
        new_stream(&mut self.opened_streams, &mut self.next_outgoing_stream_id)
    }

    pub fn next_incoming_stream(&mut self) -> (StreamId, Arc<Mutex<StreamState>>) {
        new_stream(&mut self.opened_streams, &mut self.next_incoming_stream_id)
    }

    pub fn crypto_stream(&mut self) -> (StreamId, Arc<Mutex<StreamState>>) {
        let id = StreamId::crypto_stream_id();

        let stream_state = self.opened_streams
            .entry(id)
            .or_insert_with(|| Arc::new(Mutex::new(StreamState::new(id))))
            .clone();

        (id, stream_state)
    }

    pub fn stream_state(&mut self, stream_id: StreamId) -> Result<&Arc<Mutex<StreamState>>> {
        self.opened_streams
            .get(&stream_id)
            .ok_or_else(|| ErrorKind::UnknownStreamId(stream_id).into())
    }
}
