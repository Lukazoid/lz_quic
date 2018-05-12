use {Connection, Perspective, StreamState};
use tokio_io::{AsyncRead, AsyncWrite};
use futures::Poll;
use std::io::{Error as IoError, Read, Result as IoResult, Write};
use protocol::StreamId;
use std::sync::{Arc, Mutex};

/// A stream of data between the server and client.
#[derive(Debug)]
pub struct DataStream<P: Perspective> {
    stream_id: StreamId,
    connection: Arc<Connection<P>>,
    stream_state: Arc<Mutex<StreamState>>,
}

impl<P: Perspective> DataStream<P> {
    pub(crate) fn new(
        stream_id: StreamId,
        connection: Arc<Connection<P>>,
        stream_state: Arc<Mutex<StreamState>>,
    ) -> Self {
        Self {
            stream_id: stream_id,
            connection: connection,
            stream_state: stream_state,
        }
    }

    pub(crate) fn stream_id(&self) -> StreamId {
        self.stream_id
    }

    pub(crate) fn connection(&self) -> &Arc<Connection<P>> {
        &self.connection
    }

    fn push_pending_write(&self, buf: &[u8]) {
        let mut stream_state = self.stream_state
            .lock()
            .expect("failed to obtain stream_state lock");

        stream_state.push_pending_write(buf);
    }
}

impl<P: Perspective> Read for DataStream<P> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.connection.process_incoming_packets()?;

        let mut stream_state = self.stream_state
            .lock()
            .expect("failed to obtain stream_state lock");

        let byte_count = async_io!(stream_state.poll_read(buf)?);

        Ok(byte_count)
    }
}

impl<P: Perspective> AsyncRead for DataStream<P> {}

impl<P: Perspective> Write for DataStream<P> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.push_pending_write(buf);

        self.flush()?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        async_io!(self.connection.flush_stream(self.stream_id())?);

        Ok(())
    }
}

impl<P: Perspective> AsyncWrite for DataStream<P> {
    fn shutdown(&mut self) -> Poll<(), IoError> {
        try_nb!(self.flush());

        self.connection.forget_stream(self.stream_id())?;

        Ok(().into())
    }
}

impl<P: Perspective> Drop for DataStream<P> {
    fn drop(&mut self) {
        let _ = self.connection().forget_stream(self.stream_id());
    }
}
