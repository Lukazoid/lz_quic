use errors::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use protocol::{Perspective, StreamId};
use std::sync::Arc;

#[derive(Debug)]
pub struct StreamMap {
    opened_streams: HashMap<StreamId, Arc<()>>,
    next_outgoing_stream_id: StreamId,
    next_incoming_stream_id: StreamId,
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

    pub fn next_outgoing_stream_id(&mut self) -> StreamId {
        let id = self.next_outgoing_stream_id;
        self.next_outgoing_stream_id = self.next_outgoing_stream_id.next();

        id
    }

    pub fn next_incoming_stream_id(&mut self) -> StreamId {
        let id = self.next_incoming_stream_id;
        self.next_incoming_stream_id = self.next_incoming_stream_id.next();

        id
    }

    pub fn stream_state(&mut self, stream_id: StreamId) -> Result<&Arc<()>> {
        
        if let Some(data_stream_state) = self.opened_streams.get(&stream_id) {
            return Ok(data_stream_state);
        } 

        unimplemented!();
    }

    pub fn accepted_stream(&mut self, stream_id: StreamId, data_stream: Arc<()>) -> Result<()> {
        unimplemented!();
    }
}
