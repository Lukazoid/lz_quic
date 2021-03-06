use conv::ValueInto;
use errors::*;
use frames::{AckFrame, ApplicationCloseFrame, BlockedFrame, ConnectionCloseFrame, CryptoFrame,
             InitialPacketFrame, MaxDataFrame, MaxStreamDataFrame, MaxStreamIdFrame,
             NewConnectionIdFrame, PathChallengeFrame, PathResponseFrame, ReadStreamFrameContext,
             ResetStreamFrame, StopSendingFrame, StreamBlockedFrame, StreamFrame,
             StreamIdBlockedFrame};
use protocol::{Readable, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Frame {
    Padding,
    ResetStream(ResetStreamFrame),
    ConnectionClose(ConnectionCloseFrame),
    ApplicationClose(ApplicationCloseFrame),
    MaxData(MaxDataFrame),
    MaxStreamData(MaxStreamDataFrame),
    MaxStreamId(MaxStreamIdFrame),
    Ping,
    Blocked(BlockedFrame),
    StreamBlocked(StreamBlockedFrame),
    StreamIdBlocked(StreamIdBlockedFrame),
    NewConnectionId(NewConnectionIdFrame),
    StopSending(StopSendingFrame),
    Ack(AckFrame),
    PathChallenge(PathChallengeFrame),
    PathResponse(PathResponseFrame),
    Stream(StreamFrame),
    Crypto(CryptoFrame),
}

impl From<InitialPacketFrame> for Frame {
    fn from(value: InitialPacketFrame) -> Self {
        match value {
            InitialPacketFrame::Ack(ack_frame) => Frame::Ack(ack_frame),
            InitialPacketFrame::Crypto(crypto_frame) => Frame::Crypto(crypto_frame),
        }
    }
}

bitflags!(
    flags FrameTypeFlags : u8 {
        const RESET_STREAM      = 0x01,
        const CONNECTION_CLOSE  = 0x02,
        const APPLICATION_CLOSE = 0x03,
        const MAX_DATA          = 0x04,
        const MAX_STREAM_DATA   = 0x05,
        const MAX_STREAM_ID     = 0x06,
        const PING              = 0x07,
        const BLOCKED           = 0x08,
        const STREAM_BLOCKED    = 0x09,
        const STREAM_ID_BLOCKED = 0x0a,
        const NEW_CONNECTION_ID = 0x0b,
        const STOP_SENDING      = 0x0c,
        const ACK               = 0x0d,
        const PATH_CHALLENGE    = 0x0e,
        const PATH_RESPONSE     = 0x0f,
        const CRYPTO            = 0x18,
        
    }
);

bitflags!(
    flags StreamFrameFlags : u8 {
        const STREAM_FRAME_OFFSET_PRESENT = 0x04,
        const STREAM_FRAME_LEN_PRESENT = 0x02,
        const STREAM_FRAME_FIN = 0x01,
    }
);

impl Readable for Frame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading frame");

        let flags: u8 = VarInt::read(reader)
            .chain_err(|| ErrorKind::FailedToReadFrame)?
            .into_inner()
            .value_into()
            .unwrap();

        let read_frame = if flags == 0 {
            Frame::Padding
        } else if flags >= 0x10 && flags <= 0x17 {
            let stream_frame_flags = StreamFrameFlags::from_bits_truncate(flags);
            let read_stream_frame_context = ReadStreamFrameContext {
                is_offset_present: stream_frame_flags.contains(STREAM_FRAME_OFFSET_PRESENT),
                is_length_present: stream_frame_flags.contains(STREAM_FRAME_LEN_PRESENT),
                finished: stream_frame_flags.contains(STREAM_FRAME_FIN),
            };

            Frame::Stream(Readable::read_with_context(
                reader,
                &read_stream_frame_context,
            )?)
        } else {
            let flags = FrameTypeFlags::from_bits_truncate(flags);

            match flags {
                RESET_STREAM => Frame::ResetStream(Readable::read(reader)?),
                CONNECTION_CLOSE => Frame::ConnectionClose(Readable::read(reader)?),
                APPLICATION_CLOSE => Frame::ApplicationClose(Readable::read(reader)?),
                MAX_DATA => Frame::MaxData(Readable::read(reader)?),
                MAX_STREAM_DATA => Frame::MaxStreamData(Readable::read(reader)?),
                MAX_STREAM_ID => Frame::MaxStreamId(Readable::read(reader)?),
                PING => Frame::Ping,
                BLOCKED => Frame::Blocked(Readable::read(reader)?),
                STREAM_BLOCKED => Frame::StreamBlocked(Readable::read(reader)?),
                STREAM_ID_BLOCKED => Frame::StreamIdBlocked(Readable::read(reader)?),
                NEW_CONNECTION_ID => Frame::NewConnectionId(Readable::read(reader)?),
                STOP_SENDING => Frame::StopSending(Readable::read(reader)?),
                ACK => Frame::Ack(Readable::read(reader)?),
                PATH_CHALLENGE => Frame::PathChallenge(Readable::read(reader)?),
                PATH_RESPONSE => Frame::PathResponse(Readable::read(reader)?),
                CRYPTO => Frame::Crypto(Readable::read(reader)?),
                _ => bail!(ErrorKind::FailedToReadFrame),
            }
        };

        debug!("read frame {:?}", read_frame);

        Ok(read_frame)
    }
}

impl Writable for Frame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing frame {:?}", self);

        match self {
            Frame::Padding => {
                let type_flags = VarInt::from(FrameTypeFlags::empty().bits());

                type_flags
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePaddingFrame)?;
            }
            Frame::ResetStream(reset_stream_frame) => {
                VarInt::from(RESET_STREAM.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteResetStreamFrame)?;
                reset_stream_frame.write(writer)?;
            }
            Frame::ConnectionClose(connection_close_frame) => {
                VarInt::from(CONNECTION_CLOSE.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteConnectionCloseFrame)?;
                connection_close_frame.write(writer)?;
            }
            Frame::ApplicationClose(application_close_frame) => {
                VarInt::from(APPLICATION_CLOSE.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteApplicationCloseFrame)?;
                application_close_frame.write(writer)?;
            }
            Frame::MaxData(max_data_frame) => {
                VarInt::from(MAX_DATA.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteMaxDataFrame)?;
                max_data_frame.write(writer)?;
            }
            Frame::MaxStreamData(max_stream_data_frame) => {
                VarInt::from(MAX_STREAM_DATA.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteMaxStreamDataFrame)?;
                max_stream_data_frame.write(writer)?;
            }
            Frame::MaxStreamId(max_stream_id_frame) => {
                VarInt::from(MAX_STREAM_ID.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteMaxStreamIdFrame)?;
                max_stream_id_frame.write(writer)?;
            }
            Frame::Ping => {
                VarInt::from(PING.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePingFrame)?;
            }
            Frame::Blocked(blocked_frame) => {
                VarInt::from(BLOCKED.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteBlockedFrame)?;
                blocked_frame.write(writer)?;
            }
            Frame::StreamBlocked(stream_blocked_frame) => {
                VarInt::from(STREAM_BLOCKED.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteStreamBlockedFrame)?;
                stream_blocked_frame.write(writer)?;
            }
            Frame::StreamIdBlocked(stream_id_blocked_frame) => {
                VarInt::from(STREAM_ID_BLOCKED.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteStreamIdBlockedFrame)?;
                stream_id_blocked_frame.write(writer)?
            }
            Frame::NewConnectionId(new_connection_id_frame) => {
                VarInt::from(NEW_CONNECTION_ID.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteNewConnectionIdFrame)?;
                new_connection_id_frame.write(writer)?;
            }
            Frame::StopSending(stop_sending_frame) => {
                VarInt::from(STOP_SENDING.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteStopSendingFrame)?;
                stop_sending_frame.write(writer)?;
            }
            Frame::Ack(ack_frame) => {
                VarInt::from(ACK.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;
                ack_frame.write(writer)?;
            }
            Frame::PathChallenge(path_challenge_frame) => {
                VarInt::from(PATH_CHALLENGE.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePathChallengeFrame)?;
                path_challenge_frame.write(writer)?;
            }
            Frame::PathResponse(path_response_frame) => {
                VarInt::from(PATH_RESPONSE.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePathResponseFrame)?;
                path_response_frame.write(writer)?;
            }
            Frame::Stream(stream_frame) => {
                let mut flags = StreamFrameFlags::empty();
                if stream_frame.has_offset() {
                    flags |= STREAM_FRAME_OFFSET_PRESENT;
                }

                flags |= STREAM_FRAME_LEN_PRESENT;
                if stream_frame.finished {
                    flags |= STREAM_FRAME_FIN;
                }

                VarInt::from(0x10u8 | flags.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;
                stream_frame.write(writer)?;
            }
            Frame::Crypto(crypto_frame) => {
                VarInt::from(CRYPTO.bits())
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWriteCryptoFrame)?;
                crypto_frame.write(writer)?;
            }
        }

        debug!("written frame {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;
    use bytes::Bytes;
    use frames::{CryptoFrame, StreamFrame};
    use protocol::{self, StreamId};

    #[test]
    fn write_read_stream_frame() {
        let stream_frame = Frame::Stream(StreamFrame {
            finished: true,
            offset: 5u32.into(),
            stream_id: StreamId::first_unidirectional_client_stream_id(),
            data: Bytes::from(&[0x78, 0x91][..]),
        });

        protocol::test_write_read(&stream_frame).unwrap();
    }

    #[test]
    fn write_read_crypto_frame() {
        let crypto_frame = Frame::Crypto(CryptoFrame {
            offset: 5u32.into(),
            data: Bytes::from(&[0x78, 0x91][..]),
        });

        protocol::test_write_read(&crypto_frame).unwrap();
    }
}
