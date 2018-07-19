use bytes::Bytes;
use conv::{ConvUtil, ValueInto};
use errors::*;
use futures::{Async, Poll};
use protocol::{FlowControl, StreamId, StreamOffset};
use std::collections::VecDeque;
use std::mem;
use utils::DataQueue;

#[derive(Debug)]
pub enum DequeueWriteResult {
    DequeuedWrite {
        offset: StreamOffset,
        data: Bytes,
        finished: bool,
    },
    NotReady,
}

#[derive(Debug)]
pub struct StreamState {
    stream_id: StreamId,
    incoming_data: DataQueue,
    pending_outgoing_data: VecDeque<Bytes>,
    outgoing_offset: StreamOffset,
    has_all_outgoing_data: bool,
    incoming_flow_control: Option<FlowControl>,
    outgoing_flow_control: Option<FlowControl>,
}

impl StreamState {
    pub fn new(
        stream_id: StreamId,
        initial_max_incoming_data: Option<u64>,
        initial_max_outgoing_data: Option<u64>,
    ) -> Self {
        Self {
            stream_id,
            incoming_data: DataQueue::new(),
            pending_outgoing_data: VecDeque::new(),
            outgoing_offset: 0u32.into(),
            has_all_outgoing_data: false,
            incoming_flow_control: initial_max_incoming_data.map(FlowControl::with_initial_max),
            outgoing_flow_control: initial_max_outgoing_data.map(FlowControl::with_initial_max),
        }
    }

    pub fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    pub fn enqueue_write(
        &mut self,
        buf: Bytes,
        connection_outgoing_flow_control: &mut FlowControl,
    ) -> usize {
        let bytes_to_enqueue = if let Some(outgoing_flow_control) = &mut self.outgoing_flow_control
        {
            FlowControl::take(
                outgoing_flow_control,
                connection_outgoing_flow_control,
                buf.len(),
            )
        } else {
            buf.len()
        };

        let buf = buf.slice_to(bytes_to_enqueue);
        self.pending_outgoing_data.push_back(buf);

        bytes_to_enqueue
    }

    pub fn dequeue_write(&mut self) -> DequeueWriteResult {
        if self.pending_outgoing_data.is_empty() && self.has_all_outgoing_data {
            return DequeueWriteResult::DequeuedWrite {
                offset: self.outgoing_offset,
                data: Bytes::new(),
                finished: true,
            };
        }

        if let Some(data) = self.pending_outgoing_data.pop_front() {
            let offset = self.outgoing_offset;
            self.outgoing_offset += data.len().value_as::<u64>().unwrap();

            DequeueWriteResult::DequeuedWrite {
                offset,
                data,
                finished: self.pending_outgoing_data.is_empty() && self.has_all_outgoing_data,
            }
        } else {
            DequeueWriteResult::NotReady
        }
    }

    pub fn poll_read(&mut self, buf: &mut [u8]) -> Async<usize> {
        if buf.is_empty() || self.incoming_data.is_finished() {
            return 0.into();
        }

        let read_bytes = self.incoming_data.read(buf);

        if read_bytes == 0 {
            return Async::NotReady;
        }

        return read_bytes.into();
    }
}
