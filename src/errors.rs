use handshake::Tag;
use protocol::{ConnectionId, StreamId, Version};
use frames::StreamOffset;
use std::net::SocketAddr;
use futures::{Async, Poll, Future, Stream};
use std::error::Error as StdError;
use primitives::{U24, U48};

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
        FailedToWriteU24(value: U24) {
            description("failed to write 24-bit unsigned integer")
            display("failed to write 24-bit unsigned integer '{}'", value)
        }
        FailedToWriteU32(value: u32) {
            description("failed to write 32-bit unsigned integer")
            display("failed to write 32-bit unsigned integer '{}'", value)
        }
        FailedToWriteU48(value: U48) {
            description("failed to write 48-bit unsigned integer")
            display("failed to write 48-bit unsigned integer '{}'", value)
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
        FailedToReadU24 {
            description("failed to read 24-bit unsigned integer")
        }
        FailedToReadU32 {
            description("failed to read 32-bit unsigned integer")
        }
        FailedToReadU48 {
            description("failed to read 48-bit unsigned integer")
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
        InvalidHandshakeMessage(tag: Tag) {
            description("invalid crypto handshake message")
            display("QUIC tag '{}' is an invalid crypto handshake message", tag)
        }
        FailedToWriteConnectionId(connection_id: ConnectionId) {
            description("failed to write QUIC connection id")
            display("failed to write QUIC connection id '{}'", connection_id)
        }
        FailedToWriteCryptoMessageTag(tag: Tag) {
            description("failed to write crypto message QUIC tag")
            display("failed to write crypto message QUIC tag '{}'", tag)
        }
        FailedToReadCryptoMessageTag {
            description("failed to read crypo message QUIC tag")
        }
        FailedToWriteQuicVersion(version: Version) {
            description("failed to write QUIC version")
            display("failed to write QUIC version '{}'", version)
        }
        FailedToWriteTagValueMapLength {
            description("failed to write QUIC tag-value map length")
        }
        FailedToReadTagValueMapLength {
            description("failed to read QUIC tag-value map length")
        }
        FailedToReadTagValueMap {
            description("failed to read QUIC tag-value map")
        }
        FailedToWriteTagValueMapEndOffset(end_offset: u32) {
            description("failed to write QUIC tag-value map end offset")
            display("failed to write QUIC tag-value map end offset '{}'", end_offset)
        }
        FailedToWriteTagValue(tag: Tag) {
            description("failed to write QUIC tag value")
            display("failed to write value for QUIC tag '{}'", tag)
        }
        FailedToWriteTagValueMap {
            description("failed to write QUIC tag-value map")
        }
        FailedToWritePadding(num_bytes: usize) {
            description("failed to write padding")
            display("failed to write '{}' padding bytes", num_bytes)
        }
        FailedToReadPadding(num_bytes: usize) {
            description("failed to read padding")
            display("failed to read '{}' padding bytes", num_bytes)
        }
        FailedToWriteCryptoHandshakeTagValueMap {
            description("failed to write crypto handshake message tag-value map")
        }
        InvalidStreamIdLength(length: usize) {
            description("invalid stream id length")
            display("invalid stream id length '{}'", length)
        }
        InvalidStreamOffsetLength(length: usize) {
            description("invalid stream offset length")
            display("invalid stream offset length '{}'", length)
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
        FailedToReadCryptoRejectionMessage {
            description("failed to read crypto rejection message")
        }
        FailedToReadCryptoClientHelloMessage {
            description("failed to read crypto client hello message")
        }
        FailedToReadCryptoServerConfigurationMessage {
            description("failed to read crypto server configuration message")
        }
        FailedToReadCryptoServerHelloMessage {
            description("failed to read crypto server hello message")
        }
        FailedToReadCompressedCertificateEntryType {
            description("failed to read compressed certificate entry type")
        }
        FailedToWriteCompressedCertificateEntryType {
            description("failed to write compressed certificate entry type")
        }
        FailedToReadCachedCertificateHash {
            description("failed to read cached certificate hash")
        }
        FailedToWriteCachedCertificateHash {
            description("failed to write cached certificate hash")
        }
        FailedToReadCommonCertificateSetHash {
            description("failed to read common certificate set hash")
        }
        FailedToWriteCommonCertificateSetHash {
            description("failed to write common certificate set hash")
        }
        FailedToReadCommonCertificateIndex {
            description("failed to read common certificate index")
        }
        FailedToWriteCommonCertificateIndex {
            description("failed to write common certificate index")
        }
        InvalidCompressedCertificateEntryType (entry_type: u8) {
            description("invalid compressed certificate entry type")
            display("invalid compressed certificate entry type '{}'", entry_type)
        }
        FailedToReadCompressedCertificatesUncompressedLength {
            description("failed to read compressed certificates uncompressed length")
        }
        FailedToWriteCompressedCertificateUncompressedLength (length: usize) {
            description("failed to write compressed certificate uncompressed length")
            display("failed to write compressed certificate uncompressed length '{}'", length)
        }
        FailedToReadCompressedCertificateUncompressedLength  {
            description("failed to read compressed certificate uncompressed length")
        }
        FailedToWriteCompressedCertificatesUncompressedLength (length: usize) {
            description("failed to write compressed certificates uncompressed length")
            display("failed to write compressed certificates uncompressed length '{}'", length)
        }
        FailedToWriteCompressedCertificateEntry {
            description("failed to write compressed certificate entry")
        }
        FailedToReadCompressedCertificateEntry {
            description("failed to read compressed certificate entry")
        }
        FailedToWriteCompressedChunk {
            description("failed to write compressed chunk")
        }
        FailedToWriteCertificateBytes {
            description("failed to write certificate bytes")
        }
        FailedToReadCertificateBytes {
            description("failed to read certificate bytes")
        }
        FailedToWriteCompressedCertificates {
            description("failed to write compressed certificates")
        }
        FailedToReadCompressedCertificates {
            description("failed to read compressed certificates")
        }
        FailedToFindCachedCertificateWithHash (hash: u64) {
            description("failed to find cached certificate with hash")
            display("failed to find cached certificate with hash '{}'", hash)
        }
        FailedToFindCommonCertificateSetWithHash (hash: u64) {
            description("failed to find common certificate set with hash")
            display("failed to find common certificate set with hash '{}'", hash)
        }
        FailedToFindCommonCertificateWithIndexInSet (index: usize, set_hash: u64) {
            description("failed to find certificate with index in certificate set with hash")
            display("failed to find certificate with index '{}' in certificate set with hash '{}'", index, set_hash)
        }
        CompressedCertificatesUncompressedLengthIsTooLarge (length: usize) {
            description("compress certificates uncompressed length is too large")
            display("compress certificates uncompressed length '{}' is too large", length)
        }
        NotEnoughCompressedCertificates {
            description("not enough compressed certificates")
        }
        FailedToDecompressCompressedCertificates {
            description("failed to decompress compressed certificates")
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
        FailedToInferPacketNumber {
            description("failed to infer packet number")
        }
        FailedToCreateCryptographicRandomNumberGenerator {
            description("failed to create cryptographic random number generator")
        }
        FailedToBindUdpSocket {
            description("failed to bind UDP socket")
        }
        FailedToWriteDiversificationNonce {
            description("failed to write diversification nonce")
        }
        FailedToReadDiversificationNonce {
            description("failed to read diversification nonce")
        }
        FailedToWritePartialPacketNumber{
            description("failed to write partial packet number")
        }
        FailedToReadPartialPacketNumber{
            description("failed to read partial packet number")
        }
        FailedToWritePublicPacketHeaderFlags {
            description("failed to write public packet header flags")
        }
        FailedToReadPublicPacketHeaderFlags {
            description("failed to read public packet header flags")
        }
        FailedToBuildPartialPacketNumber {
            description("failed to build the partial packet number")
        }
        CipherTextTooShort (actual_length: usize, minimum_length: usize) {
            description("the cipher text is too short")
            display("the cipher text of length '{}' is too short and must be atleast {} bytes", actual_length, minimum_length)
        }
        FailedToAuthenticateReceivedData {
            description("failed to authenticate received data")
        }
        FailedToCreateEphemerealPrivateKey {
            description("failed to create ephemereal private key")
        }
        FailedToComputePublicKey {
            description("failed to compute public key")
        }
        FailedToPerformKeyAgreement {
            description("failed to perform key agreement")
        }
        FailedToParseCertificateFromCertificateChain {
            description("failed to parse certificate from certificate chain")
        }
        FailedToParseCertificate {
            description("failed to parse certificate")
        }
        FailedToVerifyServerProof { 
            description("failed to verify server proof")
        }
        CertificateInvalidForDnsName(dns_name: String){
            description("certificate invalid for dns name")
            display("certificate invalid for dns name '{}'", dns_name)
        }
        InvalidTlsCertificate {
            description("invalid TLS certificate")
        }
        CertificateChainIsEmpty {
            description("certificate chain is empty")
        }
        FailedToPerformAesGcmEncryption {
            description("failed to perform AES GCM encryption")
        }
        FailedToPerformAesGcmDecryption {
            description("failed to perform AES GCM decryption")
        }
        FailedToGetLocalAddress {
            description("failed to get local address")
        }
        APublicKeyMustBeSpecified {
            description("a public key must be specified")
        }
        PublicKeyBytesTooLongForU24 {
            description("public key bytes too long for U24")
        }
        FailedToReadPublicKeyBytes {
            description("failed to read public key bytes")
        }
        FailedToWritePublicKeyBytes {
            description("failed to write public key bytes")
        }
        ASupportedKeyExchangeAlgorithmMustBeSpecified {
            description("a supported key exchange algorithm must be specified")
        }
        NoNonForwardSecureAead {
            description("no non-forward secure AEAD")
        }
        NoForwardSecureAead {
            description("no forward secure AEAD")
        }
        InvalidCryptoMessageType(tag: Tag) {
            description("invalid crypto message type")
            display("invalid crypto message type '{}'", tag)
        }
        InvalidStreamId(value: u32) {
            description("invalid stream id")
            display("invalid stream id '{}'", value)
        }
        ReceivedUnencryptedServerHello {
            description("received unencrypted server hello")
        }
        KeyExchangeAlgorithmAndPublicKeyCountsMustMatch {
            description("key exchange algorithm and public key counts must match")
        }
        ServerConfigurationIsRequiredBeforeForwardSecureEncryptionCanBeEstablished {
            description("server configuration is required before forward secure encryption can be established")
        }
        UnableToVerifyWithoutACertificateChain {
            description("unable to verify without a certificate chain")
        }
        UnableToSignWithoutACertificateChain {
            description("unable to sign without a certificate chain")
        }
        UnableToDeriveKeysWithoutALeafCertificate {
            description("unable to derive keys without a leaf certificate")
        }
        TheClientNonceHasAlreadyBeenGenerated {
            description("the client nonce has already been generated")
        }
        FailedToReadClientNonce {
            description("failed to read client nonce")
        }
        FailedToWriteClientNonce {
            description("failed to write client nonce")
        }
        FailedToReadServerNonce {
            description("failed to read server nonce")
        }
        FailedToWriteServerNonce {
            description("failed to write server nonce")
        }
        ServerConfigurationExpired {
            description("server configuration expired")
        }
        FailedToWriteSourceAddressToken {
            description("failed to write source address token")
        }
        FailedToReadSourceAddressToken {
            description("failed to read source address token")
        }
        FailedToParseRsaKeyPair {
            description("failed to parse RSA key-pair")
        }
        FailedToDetermineTimeSinceUnixEpoch {
            description("failed to determine time since unix epoch")
        }
        FailedToBuildRsaSigningState {
            description("failed to build RSA signing state")
        }
        FailedToSignServerProof {
            description("failed to sign server proof")
        }
        ServerProofProvidedBeforeClientHelloSent {
            description("server proof provided before client hello sent")
        }
        FailedToReadSignatureBytes {
            description("failed to read signature bytes")
        }
        FailedToWriteSignatureBytes {
            description("failed to write signature bytes")
        }
        ClientNonceIsRequiredBeforeForwardSecureEncryptionCanBeEstablished {
            description("client nonce is required before forward secure encryption can be established")
        }
        ServerNonceIsRequiredBeforeForwardSecureEncryptionCanBeEstablished {
            description("server nonce is required before forward secure encryption can be established")
        }
        UnableToUpgradeCryptoAsItIsAlreadyAtNonForwardSecureStage {
            description("unable to upgrade crypto as it is already at non-forward secure stage")
        }
        UnableToUpgradeCryptoAsItIsAlreadyAtForwardSecureStage {
            description("unable to upgrade crypto as it is already at forward secure stage")
        }
        UnableToUpgradeCryptoFromUnencryptedToForwardSecureStage {
            description("unable to upgrade crypto from unencrypted to forward secure stage")
        }
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
    pub fn downcast_cause<T: StdError + 'static>(&self) -> Option<Option<&T>>
    {
        self.1.next_error.as_ref()
            .map(|e| e.downcast_ref::<T>())
    }

    /// Determines whether this or one of the `Error` causes satisfies `predicate`.
    pub fn has_error<P:FnMut(&Error) -> bool>(&self, predicate: P) -> bool {
        self.errors().any(predicate)
    }

    /// Returns an `Iterator` which iterates over `self` and all `Error` causes.
    pub fn errors(&self) -> ErrorsIterator {
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