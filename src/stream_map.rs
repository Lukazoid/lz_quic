use errors::*;
use StreamState;
use std::collections::HashMap;
use protocol::StreamId;
use std::sync::Arc;
use std::sync::Mutex;
use std::mem;

#[derive(Debug, Clone)]
pub enum StreamMapEntry {
    Live(Arc<Mutex<StreamState>>),
    Dead,
}

#[derive(Debug)]
pub struct StreamMap {
    streams: HashMap<StreamId, StreamMapEntry>,
    next_outgoing_stream_id: StreamId,
    next_incoming_stream_id: StreamId,
}

fn new_stream(
    streams: &mut HashMap<StreamId, StreamMapEntry>,
    next_stream_id: &mut StreamId,
) -> (StreamId, Arc<Mutex<StreamState>>) {
    let id = *next_stream_id;
    *next_stream_id = next_stream_id.next();

    let new_stream_state = Arc::new(Mutex::new(StreamState::new(id)));
    streams.insert(id, StreamMapEntry::Live(new_stream_state.clone()));

    (id, new_stream_state)
}

impl StreamMap {
    pub fn new_client_stream_map() -> Self {
        StreamMap {
            streams: HashMap::new(),
            next_outgoing_stream_id: StreamId::first_client_stream_id(),
            next_incoming_stream_id: StreamId::first_server_stream_id(),
        }
    }

    pub fn new_server_stream_map() -> Self {
        StreamMap {
            streams: HashMap::new(),
            next_outgoing_stream_id: StreamId::first_server_stream_id(),
            next_incoming_stream_id: StreamId::first_client_stream_id(),
        }
    }

    pub fn next_outgoing_stream(&mut self) -> (StreamId, Arc<Mutex<StreamState>>) {
        new_stream(&mut self.streams, &mut self.next_outgoing_stream_id)
    }

    pub fn next_incoming_stream(&mut self) -> (StreamId, Arc<Mutex<StreamState>>) {
        new_stream(&mut self.streams, &mut self.next_incoming_stream_id)
    }

    pub fn crypto_stream(&mut self) -> (StreamId, StreamMapEntry) {
        let id = StreamId::crypto_stream_id();

        let stream_map_entry = self.streams.entry(id).or_insert_with(|| {
            let state = Arc::new(Mutex::new(StreamState::new(id)));
            StreamMapEntry::Live(state)
        });

        (id, stream_map_entry.clone())
    }

    pub fn get_stream(&self, stream_id: StreamId) -> Result<StreamMapEntry> {
        let stream_map_entry = self.streams
            .get(&stream_id)
            .ok_or_else(|| ErrorKind::UnknownStreamId(stream_id))?;

        Ok(stream_map_entry.clone())
    }

    pub fn forget_stream(&mut self, stream_id: StreamId) -> Result<StreamMapEntry> {
        let stream_map_entry = self.streams
            .get_mut(&stream_id)
            .ok_or_else(|| ErrorKind::UnknownStreamId(stream_id))?;

        let old_stream_map_entry = mem::replace(stream_map_entry, StreamMapEntry::Dead);

        Ok(old_stream_map_entry)
    }
}
