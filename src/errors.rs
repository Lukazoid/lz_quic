use tag::Tag;
use connection_id::ConnectionId;
use version::Version;
use frames::stream_offset::StreamOffset;
use stream_id::StreamId;
use std::net::SocketAddr;
use futures::{Async, Poll, Future, Stream};
use std::error::Error as StdError;
use primitives::u24::U24;
use primitives::u48::U48;

error_chain! {
    foreign_links {

    }
    errors {
        UnableToBindToUdpSocket(addr: SocketAddr) {
            description("unable to bind to UDP socket")
            display("unable to bind to UDP socket '{}'", addr)
        }
        UnableToWriteU8(value: u8) {
            description("unable to write 8-bit unsigned integer")
            display("unable to write 8-bit unsigned integer '{}'", value)
        }
        UnableToWriteU16(value: u16) {
            description("unable to write 16-bit unsigned integer")
            display("unable to write 16-bit unsigned integer '{}'", value)
        }
        UnableToWriteU24(value: U24) {
            description("unable to write 24-bit unsigned integer")
            display("unable to write 24-bit unsigned integer '{}'", value)
        }
        UnableToWriteU32(value: u32) {
            description("unable to write 32-bit unsigned integer")
            display("unable to write 32-bit unsigned integer '{}'", value)
        }
        UnableToWriteU48(value: U48) {
            description("unable to write 48-bit unsigned integer")
            display("unable to write 48-bit unsigned integer '{}'", value)
        }
        UnableToWriteU64(value: u64) {
            description("unable to write 64-bit unsigned integer")
            display("unable to write 64-bit unsigned integer '{}'", value)
        }
        UnableToReadU8 {
            description("unable to read 8-bit unsigned integer")
        }
        UnableToReadU16 {
            description("unable to read 16-bit unsigned integer")
        }
        UnableToReadU24 {
            description("unable to read 24-bit unsigned integer")
        }
        UnableToReadU32 {
            description("unable to read 32-bit unsigned integer")
        }
        UnableToReadU48 {
            description("unable to read 48-bit unsigned integer")
        }
        UnableToReadU64 {
            description("unable to read 64-bit unsigned integer")
        }
        UnableToWriteString(value: String) {
            description("unable to write string")
            display("unable to write string '{}'", value)
        }
        UnableToReadString {
            description("unable to read string")
        }
        UnableToReadBytes {
            description("unable to read bytes")
        }
        UnableToWriteBytes(length: usize){
            description("unable to write bytes")
            display("unable to write '{}' bytes", length)
        }
        InvalidTagValue(tag: Tag) {
            description("invalid QUIC tag value")
            display("invalid value for QUIC tag '{}'", tag)
        }
        MissingTag(tag: Tag) {
            description("missing QUIC tag value")
            display("missing QUIC tag '{}'", tag)
        }
        InvalidProofType(tag: Tag) {
            description("invalid proof type")
            display("QUIC tag '{}' is an invalid proof type", tag)
        }
        InvalidKeyExchangeAlgorithm(tag: Tag) {
            description("invalid key exchange algorithm")
            display("QUIC tag '{}' is an invalid key exchange algorithm", tag)
        }
        InvalidCryptoHandshakeMessage(tag: Tag) {
            description("invalid crypto handshake message")
            display("QUIC tag '{}' is an invalid crypto handshake message", tag)
        }
        UnableToWriteConnectionId(connection_id: ConnectionId) {
            description("unable to write QUIC connection id")
            display("unable to write QUIC connection id '{}'", connection_id)
        }
        UnableToWriteCryptoMessageTag(tag: Tag) {
            description("unable to write crypto message QUIC tag")
            display("unable to write crypto message QUIC tag '{}'", tag)
        }
        UnableToReadCryptoMessageTag {
            description("unable to read crypo message QUIC tag")
        }
        UnableToWriteQuicVersion(version: Version) {
            description("unable to write QUIC version")
            display("unable to write QUIC version '{}'", version)
        }
        UnableToWriteTagValueMapLength {
            description("unable to write QUIC tag-value map length")
        }
        UnableToReadTagValueMapLength {
            description("unable to read QUIC tag-value map length")
        }
        UnableToReadTagValueMap {
            description("unable to read QUIC tag-value map")
        }
        UnableToWriteTagValueMapEndOffset(end_offset: u32) {
            description("unable to write QUIC tag-value map end offset")
            display("unable to write QUIC tag-value map end offset '{}'", end_offset)
        }
        UnableToWriteTagValue(tag: Tag) {
            description("unable to write QUIC tag value")
            display("unable to write value for QUIC tag '{}'", tag)
        }
        UnableToWriteTagValueMap {
            description("unable to write QUIC tag-value map")
        }
        UnableToWritePadding(num_bytes: usize) {
            description("unable to write padding")
            display("unable to write '{}' padding bytes", num_bytes)
        }
        UnableToReadPadding(num_bytes: usize) {
            description("unable to read padding")
            display("unable to read '{}' padding bytes", num_bytes)
        }
        UnableToWriteCryptoHandshakeTagValueMap {
            description("unable to write crypto handshake message tag-value map")
        }
        InvalidStreamIdLength(length: usize) {
            description("invalid stream id length")
            display("invalid stream id length '{}'", length)
        }
        InvalidStreamOffsetLength(length: usize) {
            description("invalid stream offset length")
            display("invalid stream offset length '{}'", length)
        }
        UnableToWriteStreamId(stream_id: StreamId){
            description("unable to write stream id")
            display("unable to write stream id '{}'", stream_id)
        }
        UnableToReadStreamOffset {
            description("unable to read stream offset")
        }
        UnableToWriteStreamOffset(stream_offset: StreamOffset) {
            description("unable to write stream offset")
            display("unable to write stream offset '{}'", stream_offset)
        }
        UnableToWriteStreamFrame {
            description("unable to write stream frame")
        }
        UnableToWriteAckFrame {
            description("unable to write ACK frame")
        }
        UnableToWritePaddingFrame {
            description("unable to write padding frame")
        }
        UnableToWriteResetStreamFrame {
            description("unable to write reset stream frame")
        }
        UnableToWriteConnectionCloseFrame {
            description("unable to write connection close frame")
        }
        UnableToWriteGoAwayFrame {
            description("unable to write go away frame")
        }
        UnableToWriteWindowUpdateFrame {
            description("unable to write window update frame")
        }
        UnableToWriteBlockedFrame {
            description("unable to write blocked frame")
        }
        UnableToWriteStopWaitingFrame {
            description("unable to write stop waiting frame")
        }
        UnableToWritePingFrame {
            description("unable to write ping frame")
        }
        UnableToWriteVersionNegotiationPacket {
            description("unable to write version negotiation packet")
        }
        UnableToWritePublicResetPacket {
            description("unable to write public reset packet")
        }
        UnableToWriteRegularPacket {
            description("unable to write regular packet")
        }
        UnableToReadCryptoRejectionMessage {
            description("unable to read crypto rejection message")
        }
        UnableToReadCryptoClientHelloMessage {
            description("unable to read crypto client hello message")
        }
        UnableToReadCryptoServerConfigurationMessage {
            description("unable to read crypto server configuration message")
        }
        UnableToReadCompressedCertificateEntryType {
            description("unable to read compressed certificate entry type")
        }
        UnableToWriteCompressedCertificateEntryType {
            description("unable to write compressed certificate entry type")
        }
        UnableToReadCachedCertificateHash {
            description("unable to read cached certificate hash")
        }
        UnableToWriteCachedCertificateHash {
            description("unable to write cached certificate hash")
        }
        UnableToReadCommonCertificateSetHash {
            description("unable to read common certificate set hash")
        }
        UnableToWriteCommonCertificateSetHash {
            description("unable to write common certificate set hash")
        }
        UnableToReadCommonCertificateIndex {
            description("unable to read common certificate index")
        }
        UnableToWriteCommonCertificateIndex {
            description("unable to write common certificate index")
        }
        InvalidCompressedCertificateEntryType (entry_type: u8) {
            description("invalid compressed certificate entry type")
            display("invalid compressed certificate entry type '{}'", entry_type)
        }
        UnableToReadCompressedCertificatesUncompressedLength {
            description("unable to read compressed certificates uncompressed length")
        }
        UnableToWriteCompressedCertificateUncompressedLength (length: usize) {
            description("unable to write compressed certificate uncompressed length")
            display("unable to write compressed certificate uncompressed length '{}'", length)
        }
        UnableToReadCompressedCertificateUncompressedLength  {
            description("unable to read compressed certificate uncompressed length")
        }
        UnableToWriteCompressedCertificatesUncompressedLength (length: usize) {
            description("unable to write compressed certificates uncompressed length")
            display("unable to write compressed certificates uncompressed length '{}'", length)
        }
        UnableToWriteCompressedCertificateEntry {
            description("unable to write compressed certificate entry")
        }
        UnableToReadCompressedCertificateEntry {
            description("unable to read compressed certificate entry")
        }
        UnableToWriteCompressedChunk {
            description("unable to write compressed chunk")
        }
        UnableToWriteCertificateBytes {
            description("unable to write certificate bytes")
        }
        UnableToReadCertificateBytes {
            description("unable to read certificate bytes")
        }
        UnableToWriteCompressedCertificates {
            description("unable to write compressed certificates")
        }
        UnableToReadCompressedCertificates {
            description("unable to read compressed certificates")
        }
        UnableToFindCachedCertificateWithHash (hash: u64) {
            description("unable to find cached certificate with hash")
            display("unable to find cached certificate with hash '{}'", hash)
        }
        UnableToFindCommonCertificateSetWithHash (hash: u64) {
            description("unable to find common certificate set with hash")
            display("unable to find common certificate set with hash '{}'", hash)
        }
        UnableToFindCommonCertificateWithIndexInSet (index: usize, set_hash: u64) {
            description("unable to find certificate with index in certificate set with hash")
            display("unable to find certificate with index '{}' in certificate set with hash '{}'", index, set_hash)
        }
        CompressedCertificatesUncompressedLengthIsTooLarge (length: usize) {
            description("compress certificates uncompressed length is too large")
            display("compress certificates uncompressed length '{}' is too large", length)
        }
        NotEnoughCompressedCertificates {
            description("not enough compressed certificates")
        }
        UnableToDecompressCompressedCertificates {
            description("unable to decompress compressed certificates")
        }
        NotEnoughReplacementValues {
            description("not enough replacement values")
        }
        NotEnoughValuesToReplace {
            description("not enough values to replace")
        }
        DecryptionFailed {
            description("decryption failed")
        }
        UnableToInferPacketNumber {
            description("unable to infer packet number")
        }
        UnableToCreateCryptographicRandomNumberGenerator {
            description("unable to create cryptographic random number generator")
        }
        UnableToBindUdpSocket {
            description("unable to bind UDP socket")
        }
        UnableToWriteDiversificationNonce {
            description("unable to write diversification nonce")
        }
        UnableToReadDiversificationNonce {
            description("unable to read diversification nonce")
        }
        UnableToWritePartialPacketNumber{
            description("unable to write partial packet number")
        }
        UnableToReadPartialPacketNumber{
            description("unable to read partial packet number")
        }
        UnableToWritePublicPacketHeaderFlags {
            description("unable to write public packet header flags")
        }
        UnableToReadPublicPacketHeaderFlags {
            description("unable to read public packet header flags")
        }
        UnableToBuildPartialPacketNumber {
            description("unable to build the partial packet number")
        }
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
            future.chain_err(|| ErrorKind::Msg("An error occurred".to_owned()));

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
            stream.chain_err(|| ErrorKind::Msg("An error occurred".to_owned()));

        // Assert
        let poll = chained_err_stream.poll();
        assert!(poll.is_ok());
        assert_eq!(poll.unwrap(), Async::Ready(Some(1)));

        let poll = chained_err_stream.poll();
        assert!(poll.is_err());
    }
}