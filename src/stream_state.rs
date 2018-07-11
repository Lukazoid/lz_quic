use bytes::Bytes;
use errors::*;
use futures::{Async, Poll};
use protocol::{FlowControl, StreamId};
use std::collections::VecDeque;
use std::mem;
use utils::DataQueue;

#[derive(Debug)]
pub struct StreamState {
    stream_id: StreamId,
    incoming_data: DataQueue,
    pending_outgoing_data: VecDeque<Bytes>,
    incoming_flow_control: Option<FlowControl>,
    outgoing_flow_control: Option<FlowControl>,
}

impl StreamState {
    pub fn new(
        stream_id: StreamId,
        initial_max_incoming_data: Option<u64>,
        initial_max_outgoing_data: Option<u64>,
    ) -> Self {
        // TODO LH Initialize the flow control
        Self {
            stream_id,
            incoming_data: DataQueue::new(),
            pending_outgoing_data: VecDeque::new(),
            incoming_flow_control: initial_max_incoming_data.map(FlowControl::with_initial_max),
            outgoing_flow_control: initial_max_outgoing_data.map(FlowControl::with_initial_max),
        }
    }

    pub fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    pub fn enqueue_write<B: Into<Bytes>>(&mut self, buf: B) {
        self.pending_outgoing_data.push_back(buf.into())
    }

    pub fn dequeue_write(&mut self) -> Option<Bytes> {
        self.pending_outgoing_data.pop_front()
    }

    pub fn dequeue_writes(&mut self) -> impl Iterator<Item = Bytes> {
        mem::replace(&mut self.pending_outgoing_data, VecDeque::new()).into_iter()
    }

    pub fn poll_read(&mut self, buf: &mut [u8]) -> Poll<usize, Error> {
        if buf.is_empty() {
            return Ok(0.into());
        }

        let read_bytes = self.incoming_data.read(buf);

        if read_bytes == 0 {
            return Ok(Async::NotReady);
        }

        return Ok(read_bytes.into());
    }
}
