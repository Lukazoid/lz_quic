use errors::*;
use bytes::Bytes;
use std::collections::VecDeque;
use protocol::StreamId;
use futures::{Poll, Stream};
use futures::stream::Then;
use futures::sync::mpsc::{self, Receiver, Sender};
use std::result::Result as StdResult;
use std::io::{Error as IoError, Read, Result as IoResult};
use lz_stream_io::StreamRead;

#[derive(Debug)]
pub struct StreamState {
    stream_id: StreamId,
    pending_writes: VecDeque<Bytes>,
    async_read: StreamRead<
        Then<
            Receiver<Result<Bytes>>,
            fn(StdResult<Result<Bytes>, ()>) -> IoResult<Bytes>,
            IoResult<Bytes>,
        >,
    >,
    incoming: Sender<Result<Bytes>>,
}

fn unwrap_read_stream_error(result: StdResult<Result<Bytes>, ()>) -> IoResult<Bytes> {
    let result = match result {
        Ok(inner) => inner,
        Err(()) => Err(ErrorKind::DataStreamClosed.into()),
    };

    let bytes = result?;

    Ok(bytes)
}

impl StreamState {
    pub fn new(stream_id: StreamId) -> Self {
        // TODO LH Decide on a sensible maximum unprocessed number of incoming frames
        let (sender, receiver) = mpsc::channel(100);

        Self {
            stream_id: stream_id,
            pending_writes: Default::default(),
            async_read: StreamRead::new(receiver.then(unwrap_read_stream_error)),
            incoming: sender,
        }
    }

    pub fn push_pending_write<B: Into<Bytes>>(&mut self, buf: B) {
        self.pending_writes.push_back(buf.into())
    }

    pub fn pop_pending_write(&mut self) -> Option<Bytes> {
        self.pending_writes.pop_front()
    }

    fn poll_read_io(&mut self, buf: &mut [u8]) -> Poll<usize, IoError> {
        let byte_count = try_nb!(self.async_read.read(buf));

        Ok(byte_count.into())
    }

    pub fn poll_read(&mut self, buf: &mut [u8]) -> Poll<usize, Error> {
        self.poll_read_io(buf)
            .chain_err(|| ErrorKind::FailedToReadStreamData(self.stream_id))
    }
}
