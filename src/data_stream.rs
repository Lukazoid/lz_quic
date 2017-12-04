use Session;
use tokio_io::{AsyncRead, AsyncWrite};
use futures::Poll;
use std::io::{Error as IoError, Read, Result as IoResult, Write};
use protocol::StreamId;
use std::sync::Arc;

/// A stream of data between the server and client.
#[derive(Debug)]
pub struct DataStream<P> {
    stream_id: StreamId,
    session: Arc<Session<P>>,
}

impl<P> DataStream<P> {
    pub(crate) fn stream_id(&self) -> StreamId {
        self.stream_id
    }
}

impl<P> Read for DataStream<P> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        unimplemented!()
    }
}

impl<P> AsyncRead for DataStream<P> {}

impl<P> Write for DataStream<P> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        unimplemented!();
    }

    fn flush(&mut self) -> IoResult<()> {
        unimplemented!()
    }
}

impl<P> AsyncWrite for DataStream<P> {
    fn shutdown(&mut self) -> Poll<(), IoError> {
        unimplemented!()
    }
}
