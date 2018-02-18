use protocol::{ConnectionId, StreamId, Version};
use frames::StreamOffset;
use std::net::SocketAddr;
use futures::{Async, Poll, Future, Stream};
use std::error::Error as StdError;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

error_chain! {
    foreign_links {

    }
    errors {
        FailedToBindToUdpSocket(addr: SocketAddr) {
            description("failed to bind to UDP socket")
            display("failed to bind to UDP socket '{}'", addr)
        }
        FailedToWriteU8(value: u8) {
            description("failed to write 8-bit unsigned integer")
            display("failed to write 8-bit unsigned integer '{}'", value)
        }
        FailedToWriteU16(value: u16) {
            description("failed to write 16-bit unsigned integer")
            display("failed to write 16-bit unsigned integer '{}'", value)
        }
        FailedToWriteU32(value: u32) {
            description("failed to write 32-bit unsigned integer")
            display("failed to write 32-bit unsigned integer '{}'", value)
        }
        FailedToWriteU64(value: u64) {
            description("failed to write 64-bit unsigned integer")
            display("failed to write 64-bit unsigned integer '{}'", value)
        }
        FailedToReadU8 {
            description("failed to read 8-bit unsigned integer")
        }
        FailedToReadU16 {
            description("failed to read 16-bit unsigned integer")
        }
        FailedToReadU32 {
            description("failed to read 32-bit unsigned integer")
        }
        FailedToReadU64 {
            description("failed to read 64-bit unsigned integer")
        }
        FailedToWriteString(value: String) {
            description("failed to write string")
            display("failed to write string '{}'", value)
        }
        FailedToReadString {
            description("failed to read string")
        }
        FailedToReadBytes {
            description("failed to read bytes")
        }
        FailedToWriteBytes(length: usize){
            description("failed to write bytes")
            display("failed to write '{}' bytes", length)
        }
        FailedToWriteConnectionId(connection_id: ConnectionId) {
            description("failed to write QUIC connection id")
            display("failed to write QUIC connection id '{}'", connection_id)
        }
        FailedToWriteQuicVersion(version: Version) {
            description("failed to write QUIC version")
            display("failed to write QUIC version '{}'", version)
        }
        FailedToWritePadding(num_bytes: usize) {
            description("failed to write padding")
            display("failed to write '{}' padding bytes", num_bytes)
        }
        FailedToReadPadding(num_bytes: usize) {
            description("failed to read padding")
            display("failed to read '{}' padding bytes", num_bytes)
        }
        VarIntValueIsTooLargeToFitInIntegerOfSize(bytes: u64, integer_size: usize) {
            description("variable width integer value is too large to fit in integer")
            display("variable width integer value '{}' is too large to fit in integer of size '{}'", bytes, integer_size)
        }
        IntegerValueIsTooLargeToBeStoredAsAVarInt(value: u64) {
            description("integer value is too large to be stored as a variable width integer")
            display("integer value '{}' is too large to be stored as a variable width integer", value)
        }
        UnknownStreamId(stream_id: StreamId) {
            description("unknown stream id")
            display("unknown stream id '{}'", stream_id)
        }
        FailedToWriteStreamId(stream_id: StreamId){
            description("failed to write stream id")
            display("failed to write stream id '{}'", stream_id)
        }
        FailedToReadStreamOffset {
            description("failed to read stream offset")
        }
        FailedToWriteStreamOffset(stream_offset: StreamOffset) {
            description("failed to write stream offset")
            display("failed to write stream offset '{}'", stream_offset)
        }
        FailedToWriteStreamFrame {
            description("failed to write stream frame")
        }
        FailedToWriteAckFrame {
            description("failed to write ACK frame")
        }
        FailedToWritePaddingFrame {
            description("failed to write padding frame")
        }
        FailedToWriteResetStreamFrame {
            description("failed to write reset stream frame")
        }
        FailedToWriteConnectionCloseFrame {
            description("failed to write connection close frame")
        }
        FailedToWriteGoAwayFrame {
            description("failed to write go away frame")
        }
        FailedToWriteWindowUpdateFrame {
            description("failed to write window update frame")
        }
        FailedToWriteBlockedFrame {
            description("failed to write blocked frame")
        }
        FailedToWriteStopWaitingFrame {
            description("failed to write stop waiting frame")
        }
        FailedToWritePingFrame {
            description("failed to write ping frame")
        }
        FailedToWriteVersionNegotiationPacket {
            description("failed to write version negotiation packet")
        }
        FailedToWritePublicResetPacket {
            description("failed to write public reset packet")
        }
        FailedToWriteRegularPacket {
            description("failed to write regular packet")
        }
        NotEnoughReplacementValues {
            description("not enough replacement values")
        }
        NotEnoughValuesToReplace {
            description("not enough values to replace")
        }
        FailedToInferPacketNumber {
            description("failed to infer packet number")
        }
        FailedToCreateCryptographicRandomNumberGenerator {
            description("failed to create cryptographic random number generator")
        }
        FailedToBindUdpSocket {
            description("failed to bind UDP socket")
        }
        FailedToWritePartialPacketNumber{
            description("failed to write partial packet number")
        }
        FailedToReadPartialPacketNumber{
            description("failed to read partial packet number")
        }
        FailedToWritePacketHeaderFlags {
            description("failed to write public packet header flags")
        }
        FailedToReadPacketHeaderFlags {
            description("failed to read public packet header flags")
        }
        FailedToBuildPartialPacketNumber {
            description("failed to build the partial packet number")
        }
        FailedToGetLocalAddress {
            description("failed to get local address")
        }
        InvalidLongHeaderPacketType(packet_type: u8) {
            description("invalid long header packet type")
            display("invalid long header packet type '{}'", packet_type)
        }
        InvalidShortHeaderPacketType(packet_type: u8) {
            description("invalid short header packet type")
            display("invalid short header packet type '{}'", packet_type)
        }
        ReachedMaximumPacketNumber {
            description("reached maximum packet number")
        }
        ValueExceedsTheMaximumPacketNumberValue {
            description("value exeeds the maximum packet number value")
        }
        HostIsNotAValidDomainName(host: String) {
            description("host is not a valid domain name")
            display("host '{}' is not a valid domain name", host)
        }
        FailedToPerformTlsHandshake(host: String) {
            description("failed to perform TLS handshake")
            display("failed to perform TLS handshake to host '{}'", host)
        }
        DataStreamClosed {
            description("the data stream has been closed")
        }
        FailedToReadStreamData(stream_id: StreamId) {
            description("failed to read stream data")
            display("failed to read stream data for stream '{}'", stream_id)
        }
    }
}

impl<'a> From<&'a ErrorKind> for IoErrorKind {
    fn from(error: &'a ErrorKind) -> Self {
        match *error {
            ErrorKind::DataStreamClosed => IoErrorKind::NotConnected,
            _ => IoErrorKind::Other,
        }
    }
}

impl From<ErrorKind> for IoErrorKind {
    fn from(error: ErrorKind) -> Self {
        IoErrorKind::from(&error)
    }
}

impl From<Error> for IoError {
    fn from(error: Error) -> Self {        
        IoError::new(error.kind().into(), error.to_string())
    }
}

pub struct ErrorsIterator<'a> {
    current_error: Option<&'a Error>
}

impl<'a> Iterator for ErrorsIterator<'a> {
    type Item = &'a Error;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_error) = self.current_error.take() {
            if let Some(next_error) = current_error.downcast_cause() {
                self.current_error = next_error;
            }

            Some(current_error) 
        } else {
            None
        }
    }
}

impl Error {
    /// Attempts to downcast the `Error::cause()` to `T`.
    ///
    /// # Returns
    /// `None` if there was no cause.
    /// `Some(None)` if there was a cause which could not be downcast to `T`.
    /// `Some(Some(T))` if the cause was successfully downcast to `T`.
    pub(crate) fn downcast_cause<T: StdError + 'static>(&self) -> Option<Option<&T>>
    {
        self.1.next_error.as_ref()
            .map(|e| e.downcast_ref::<T>())
    }

    /// Determines whether this or one of the `Error` causes satisfies `predicate`.
    pub(crate) fn has_error<P:FnMut(&Error) -> bool>(&self, predicate: P) -> bool {
        self.errors().any(predicate)
    }

    /// Returns an `Iterator` which iterates over `self` and all `Error` causes.
    pub(crate) fn errors(&self) -> ErrorsIterator {
        ErrorsIterator { current_error: Some(self) }
    }
}

pub struct ChainErrStream<S, C> {
    stream: S,
    callback: C,
}

impl<E, S, C, EK> Stream for ChainErrStream<S, C>
    where E: 'static + StdError + Send,
          S: Stream<Error = E>,
          EK: Into<ErrorKind>,
          C: FnMut() -> EK
{
    type Item = S::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.stream.poll().chain_err(|| (self.callback)())
    }
}

pub trait StreamExt: Stream {
    fn chain_err<C, EK>(self, callback: C) -> ChainErrStream<Self, C>
        where C: FnMut() -> EK,
              EK: Into<ErrorKind>,
              Self: Sized,
              Self::Error: StdError;
}

impl<S: Stream> StreamExt for S {
    fn chain_err<C, EK>(self, callback: C) -> ChainErrStream<Self, C>
        where C: FnMut() -> EK,
              EK: Into<ErrorKind>,
              Self: Sized,
              Self::Error: StdError
    {
        ChainErrStream {
            stream: self,
            callback: callback,
        }
    }
}

pub struct ChainErrFuture<F, C> {
    future: F,
    callback: Option<C>,
}

impl<E, F, C, EK> Future for ChainErrFuture<F, C>
    where E: 'static + StdError + Send,
          F: Future<Error = E>,
          EK: Into<ErrorKind>,
          C: FnOnce() -> EK
{
    type Item = F::Item;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let e = match self.future.poll() {
            Ok(Async::NotReady) => return Ok(Async::NotReady),
            other => other,
        };

        e.chain_err(self.callback.take().expect("cannot poll ChainErrFuture twice"))
    }
}

pub trait FutureExt: Future {
    fn chain_err<C, EK>(self, callback: C) -> ChainErrFuture<Self, C>
        where C: FnOnce() -> EK,
              EK: Into<ErrorKind>,
              Self: Sized,
              Self::Error: StdError;
}

impl<F: Future> FutureExt for F {
    fn chain_err<C, EK>(self, callback: C) -> ChainErrFuture<Self, C>
        where C: FnOnce() -> EK,
              EK: Into<ErrorKind>,
              Self: Sized,
              Self::Error: StdError
    {
        ChainErrFuture {
            future: self,
            callback: Some(callback),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::Async;
    use futures::future::{self, Future};
    use futures::stream::{self, Stream};
    use std::io::{Error as IoError, ErrorKind as IoErrorKind};

    #[test]
    pub fn future_chain_err_chains_err() {
        // Arrange
        let future = future::err::<(), IoError>(IoError::from(IoErrorKind::InvalidData));

        // Act
        let mut chained_err_future =
            future.chain_err(|| "An error occurred");

        // Assert
        let poll = chained_err_future.poll();
        assert!(poll.is_err())
    }

    #[test]
    pub fn stream_chain_err_chains_err() {
        // Arrange
        let stream = stream::iter(vec![Ok(1), Err(IoError::from(IoErrorKind::InvalidData))]);

        // Act
        let mut chained_err_stream =
            stream.chain_err(|| "An error occurred");

        // Assert
        let poll = chained_err_stream.poll();
        assert!(poll.is_ok());
        assert_eq!(poll.unwrap(), Async::Ready(Some(1)));

        let poll = chained_err_stream.poll();
        assert!(poll.is_err());
    }

    #[test]
    pub fn has_error_returns_true_for_matching_error() {
        let first_cause : Error = "The first error".into();
        let chained_result : Result<()> = Err(first_cause).chain_err(||"The next error");

        let error = chained_result.unwrap_err();

        assert!(error.has_error(|e| matches!(e.kind(), &ErrorKind::Msg(ref msg) if msg == "The first error")));
    }

    #[test]
    pub fn errors_returns_errors() {
        let first_cause : Error = "The first error".into();
        let chained_result : Result<()> = Err(first_cause).chain_err(||"The next error");

        let error = chained_result.unwrap_err();

        assert_eq!(error.errors().count(), 2);
    }

    #[test]
    pub fn errors_only_returns_errors() {
        let first_cause = IoError::new(IoErrorKind::InvalidData, "first cause");
        let chained_result : Result<()> = Err(first_cause).chain_err(||"The next error");

        let error = chained_result.unwrap_err();

        assert_eq!(error.errors().count(), 1);
    }
}