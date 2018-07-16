use bytes::Bytes;
use errors::Error;
use futures::{Async, Poll};
use protocol::StreamId;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write};
use std::sync::{Arc, Mutex};
use tokio_io::{AsyncRead, AsyncWrite};
use {Connection, Perspective, StreamState};

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

    fn enqueue_write(&self, buf: Bytes) -> usize {
        let mut stream_state = self.stream_state
            .lock()
            .expect("failed to obtain stream_state lock");

        let mut connection_outgoing_flow_control = self.connection
            .outgoing_flow_control()
            .lock()
            .expect("failed to obtain connection outgoing_flow_control lock");

        stream_state.enqueue_write(buf, &mut *connection_outgoing_flow_control)
    }

    fn poll_read(&mut self, buf: &mut [u8]) -> Poll<usize, Error> {
        loop {
            let read_result = {
                let mut stream_state = self.stream_state
                    .lock()
                    .expect("failed to obtain stream_state lock");
                stream_state.poll_read(buf)
            };

            if let Async::Ready(byte_count) = read_result {
                // if some bytes were read then we will return the read bytes immediately
                return Ok(byte_count.into());
            }

            if self.connection
                .poll_process_incoming_packets()?
                .is_not_ready()
            {
                return Ok(Async::NotReady);
            }
        }
    }
}

impl<P: Perspective> Read for DataStream<P> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        trace!("stream {:?}: reading", self.stream_id());

        let byte_count = async_io!(self.poll_read(buf)?);

        debug!("stream {:?}: read {} bytes", self.stream_id(), byte_count);

        Ok(byte_count)
    }
}

impl<P: Perspective> AsyncRead for DataStream<P> {}

impl<P: Perspective> Write for DataStream<P> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        trace!("stream {:?}: writing", self.stream_id());

        let bytes: Bytes = buf.into();

        loop {
            // try to buffer as many bytes as flow control allows
            let byte_count = self.enqueue_write(bytes.clone());

            // let the connection transmit any packets if it decides
            let transmitted_async = self.connection.poll_try_transmit()?;

            if byte_count > 0 {
                debug!(
                    "stream {:?}: written {} bytes",
                    self.stream_id(),
                    byte_count
                );
                return Ok(byte_count);
            }

            // if no bytes could be buffered then process incoming packets
            if self.connection.poll_process_incoming_packet()?.is_ready() {
                // if any new incoming packets then re-attempt buffering
                // in case we were granted more credit
                continue;
            }

            async_io!(transmitted_async);
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        trace!("stream {:?}: flushing", self.stream_id());

        async_io!(self.connection.poll_flush_stream(self.stream_id())?);

        debug!("stream {:?}: flushed", self.stream_id());

        Ok(())
    }
}

impl<P: Perspective> AsyncWrite for DataStream<P> {
    fn shutdown(&mut self) -> Poll<(), IoError> {
        trace!("stream {:?}: shutting down", self.stream_id());

        self.connection
            .poll_flush_stream_and_wait_for_ack(self.stream_id())?;

        // if we get to this point we know all sent bytes have been acknowledged by the remote end

        self.connection.poll_forget_stream(self.stream_id())?;

        debug!("stream {:?}: shutdown", self.stream_id());

        Ok(().into())
    }
}

impl<P: Perspective> Drop for DataStream<P> {
    fn drop(&mut self) {
        let _ = self.connection().poll_forget_stream(self.stream_id());
    }
}
