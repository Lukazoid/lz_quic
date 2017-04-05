use quic_tag::QuicTag;
use quic_connection_id::QuicConnectionId;
use quic_version::QuicVersion;
use frames::stream_frame::{StreamId, StreamOffset};
use std::io::Error as IoError;
error_chain! {
    foreign_links {
        Io(IoError);
    }
    errors {
        U32ToU24IntegerOverflow(value: u32){
            description("overflow when creating 24-bit unsigned integer from 32-bit unsigned integer")
            display("overflow when creating 24-bit unsigned integer from 32-bit unsigned integer '{}'", value)
        }
        U64ToU48IntegerOverflow(value: u64){
            description("overflow when creating 48-bit unsigned integer from 64-bit unsigned integer")
            display("overflow when creating 48-bit unsigned integer from 64-bit unsigned integer '{}'", value)
        }
        UnableToWriteU8(value: u8) {
            description("unable to write unsigned 8-bit integer")
            display("unable to write unsigned 8-bit integer '{}'", value)
        }
        UnableToWriteU16(value: u16) {
            description("unable to write unsigned 16-bit integer")
            display("unable to write unsigned 16-bit integer '{}'", value)
        }
        UnableToWriteU32(value: u32) {
            description("unable to write unsigned 32-bit integer")
            display("unable to write unsigned 32-bit integer '{}'", value)
        }
        UnableToWriteU64(value: u64) {
            description("unable to write 64-bit unsigned integer")
            display("unable to write 64 bit unsigned integer '{}'", value)
        }
        UnableToReadU8 {
            description("unable to read unsigned 8-bit integer")
        }
        UnableToReadU16 {
            description("unable to read 16-bit unsigned integer")
        }
        UnableToReadU32 {
            description("unable to read 32-bit unsigned integer")
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
        InvalidQuicTagValue(quic_tag: QuicTag) {
            description("invalid QUIC tag value")
            display("invalid value for QUIC tag '{}'", quic_tag)
        }
        MissingQuicTag(quic_tag: QuicTag) {
            description("missing QUIC tag value")
            display("missing QUIC tag '{}'", quic_tag)
        }
        InvalidProofType(quic_tag: QuicTag) {
            description("invalid proof type")
            display("QUIC tag '{}' is an invalid proof type", quic_tag)
        }
        InvalidKeyExchangeAlgorithm(quic_tag: QuicTag) {
            description("invalid key exchange algorithm")
            display("QUIC tag '{}' is an invalid key exchange algorithm", quic_tag)
        }
        InvalidCryptoHandshakeMessage(quic_tag: QuicTag) {
            description("invalid crypto handshake message")
            display("QUIC tag '{}' is an invalid crypto handshake message", quic_tag)
        }
        UnableToWriteQuicConnectionId(quic_connection_id: QuicConnectionId) {
            description("unable to write QUIC connection id")
            display("unable to write QUIC connection id '{}'", quic_connection_id)
        }
        UnableToWriteCryptoMessageQuicTag(quic_tag: QuicTag) {
            description("unable to write crypto message QUIC tag")
            display("unable to write crypto message QUIC tag '{}'", quic_tag)
        }
        UnableToReadCryptoMessageQuicTag {
            description("unable to read crypo message QUIC tag")
        }
        UnableToWriteQuicVersion(quic_version: QuicVersion) {
            description("unable to write QUIC version")
            display("unable to write QUIC version '{}'", quic_version)
        }
        UnableToWriteQuicTagValueMapLength {
            description("unable to write QUIC tag-value map length")
        }
        UnableToReadQuicTagValueMapLength {
            description("unable to read QUIC tag-value map length")
        }
        UnableToReadQuicTagValueMap {
            description("unable to read QUIC tag-value map")
        }
        UnableToWriteQuicTagValueMapEndOffset(end_offset: u32) {
            description("unable to write QUIC tag-value map end offset")
            display("unable to write QUIC tag-value map end offset '{}'", end_offset)
        }
        UnableToWriteQuicTagValue(quic_tag: QuicTag) {
            description("unable to write QUIC tag value")
            display("unable to write value for QUIC tag '{}'", quic_tag)
        }
        UnableToWriteQuicTagValueMap {
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
    }
}