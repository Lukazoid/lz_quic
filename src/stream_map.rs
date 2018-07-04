use errors::*;
use protocol::{StreamId, StreamType};
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;
use std::sync::Mutex;
use StreamState;

#[derive(Debug, Clone)]
pub enum StreamMapEntry {
    Live(Arc<Mutex<StreamState>>),
    Dead,
}

#[derive(Debug)]
pub struct StreamMap {
    streams: HashMap<StreamId, StreamMapEntry>,
    next_outgoing_unidirectional_stream_id: StreamId,
    next_outgoing_bidirectional_stream_id: StreamId,
}

fn new_stream(
    streams: &mut HashMap<StreamId, StreamMapEntry>,
    next_stream_id: &mut StreamId,
    initial_max_incoming_data_per_stream: u64,
    initial_max_outgoing_data_per_stream: u64,
) -> (StreamId, Arc<Mutex<StreamState>>) {
    let id = *next_stream_id;
    *next_stream_id = next_stream_id.next();

    let new_stream_state = Arc::new(Mutex::new(StreamState::new(
        id,
        Some(initial_max_incoming_data_per_stream),
        Some(initial_max_outgoing_data_per_stream),
    )));
    streams.insert(id, StreamMapEntry::Live(new_stream_state.clone()));

    (id, new_stream_state)
}

impl StreamMap {
    pub fn new_client_stream_map() -> Self {
        StreamMap {
            streams: HashMap::new(),
            next_outgoing_unidirectional_stream_id: StreamId::first_unidirectional_client_stream_id(
            ),
            next_outgoing_bidirectional_stream_id: StreamId::crypto_stream_id().next(),
        }
    }

    pub fn new_server_stream_map() -> Self {
        StreamMap {
            streams: HashMap::new(),
            next_outgoing_unidirectional_stream_id: StreamId::first_unidirectional_server_stream_id(
            ),
            next_outgoing_bidirectional_stream_id: StreamId::first_bidirectional_server_stream_id(),
        }
    }

    pub fn next_outgoing_stream(
        &mut self,
        stream_type: StreamType,
        initial_max_incoming_data: u64,
        initial_max_outgoing_data: u64,
    ) -> (StreamId, Arc<Mutex<StreamState>>) {
        let next_outgoing_stream_id = match stream_type {
            StreamType::Unidirectional => &mut self.next_outgoing_unidirectional_stream_id,
            StreamType::Bidirectional => &mut self.next_outgoing_bidirectional_stream_id,
        };

        new_stream(
            &mut self.streams,
            next_outgoing_stream_id,
            initial_max_incoming_data,
            initial_max_outgoing_data,
        )
    }

    pub fn crypto_stream(&mut self) -> (StreamId, StreamMapEntry) {
        let id = StreamId::crypto_stream_id();

        (id, self.get_or_ensure_stream(id))
    }

    pub fn get_stream(&self, stream_id: StreamId) -> Result<StreamMapEntry> {
        let stream_map_entry = self.streams
            .get(&stream_id)
            .ok_or_else(|| ErrorKind::UnknownStreamId(stream_id))?;

        Ok(stream_map_entry.clone())
    }

    pub fn get_or_ensure_stream(&mut self, stream_id: StreamId) -> StreamMapEntry {
        let stream_map_entry = self.streams.entry(stream_id).or_insert_with(|| {
            let state = Arc::new(Mutex::new(StreamState::new(stream_id, None, None)));
            StreamMapEntry::Live(state)
        });

        stream_map_entry.clone()
    }

    pub fn forget_stream(&mut self, stream_id: StreamId) -> Result<StreamMapEntry> {
        let stream_map_entry = self.streams
            .get_mut(&stream_id)
            .ok_or_else(|| ErrorKind::UnknownStreamId(stream_id))?;

        let old_stream_map_entry = mem::replace(stream_map_entry, StreamMapEntry::Dead);

        Ok(old_stream_map_entry)
    }
}
