use {Connection, Perspective};
use tokio_io::{AsyncRead, AsyncWrite};
use futures::Poll;
use std::io::{Error as IoError, Read, Result as IoResult, Write};
use protocol::StreamId;
use std::sync::Arc;

/// A stream of data between the server and client.
#[derive(Debug)]
pub struct DataStream<P: Perspective> {
    stream_id: StreamId,
    connection: Arc<Connection<P>>,
}

impl<P: Perspective> DataStream<P> {
    pub(crate) fn new(stream_id: StreamId, connection: Arc<Connection<P>>) -> Self {
        Self {
            stream_id: stream_id,
            connection: connection,
        }
    }

    pub(crate) fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    pub(crate) fn connection(&self) -> &Arc<Connection<P>> {
        &self.connection
    }
}

impl<P: Perspective> Read for DataStream<P> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        unimplemented!()
    }
}

impl<P: Perspective> AsyncRead for DataStream<P> {}

impl<P: Perspective> Write for DataStream<P> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        unimplemented!();
    }

    fn flush(&mut self) -> IoResult<()> {
        unimplemented!()
    }
}

impl<P: Perspective> AsyncWrite for DataStream<P> {
    fn shutdown(&mut self) -> Poll<(), IoError> {
        unimplemented!()
    }
}
