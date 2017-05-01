use errors::*;
use frames::stream_offset::{StreamOffset, StreamOffsetLength};
use stream_id::{StreamId, StreamIdLength};
use frames::stream_frame::StreamFrame;
use frames::ack_frame::AckFrame;
use std::io::Write;
use byteorder::WriteBytesExt;
use writable::Writable;

#[derive(Debug, Clone)]
pub enum Frame {
    Stream(StreamFrame),
    Ack(AckFrame),
    Padding,
    ResetStream,
    ConnectionClose,
    GoAway,
    WindowUpdate,
    Blocked,
    StopWaiting,
    Ping,
}

bitflags!(
    flags FrameTypeFlags : u8 {
        const STREAM            = 0x80,
        const ACK               = 0x40,
        const RESET_STREAM      = 0x01,
        const CONNECTION_CLOSE  = 0x02,
        const GO_AWAY           = 0x03,
        const WINDOW_UPDATE     = 0x04,
        const BLOCKED           = 0x05,
        const STOP_WAITING      = 0x06,
        const PING              = 0x07,
    }
);

impl Writable for Frame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            &Frame::Stream(ref stream_frame) => {
                let mut type_flags = STREAM.bits();
                // TODO LH Add the data length field

                let mut payload = Vec::with_capacity(12);

                let stream_id_length = stream_frame
                    .stream_id
                    .write(&mut payload)
                    .chain_err(|| ErrorKind::UnableToWriteStreamId(stream_frame.stream_id))
                    .chain_err(|| ErrorKind::UnableToWriteStreamFrame)?;

                type_flags |= match stream_id_length {
                    StreamIdLength::OneByte => 0b00,
                    StreamIdLength::TwoBytes => 0b01,
                    StreamIdLength::ThreeBytes => 0b10,
                    StreamIdLength::FourBytes => 0b11,
                };

                let offset_header_length = stream_frame
                    .offset
                    .write(&mut payload)
                    .chain_err(|| ErrorKind::UnableToWriteStreamOffset(stream_frame.offset))
                    .chain_err(|| ErrorKind::UnableToWriteStreamFrame)?;

                type_flags |= match offset_header_length {
                    StreamOffsetLength::ZeroBytes => 0,
                    StreamOffsetLength::TwoBytes => 0b00100,
                    StreamOffsetLength::ThreeBytes => 0b01000,
                    StreamOffsetLength::FourBytes => 0b01100,
                    StreamOffsetLength::FiveBytes => 0b10000,
                    StreamOffsetLength::SixBytes => 0b10100,
                    StreamOffsetLength::SevenBytes => 0b11000,
                    StreamOffsetLength::EightBytes => 0b11100,
                };

                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteStreamFrame)?;

                writer
                    .write_all(&payload)
                    .chain_err(|| ErrorKind::UnableToWriteStreamFrame)?;
            }
            &Frame::Ack(ref ack_frame) => {
                let type_flags = ACK.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteAckFrame)?;
            }
            &Frame::Padding => {
                let type_flags = FrameTypeFlags::empty().bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWritePaddingFrame)?;
            }
            &Frame::ResetStream => {
                let type_flags = RESET_STREAM.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteResetStreamFrame)?;
            }
            &Frame::ConnectionClose => {
                let type_flags = CONNECTION_CLOSE.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteConnectionCloseFrame)?;
            }
            &Frame::GoAway => {
                let type_flags = GO_AWAY.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteGoAwayFrame)?;
            }
            &Frame::WindowUpdate => {
                let type_flags = WINDOW_UPDATE.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteWindowUpdateFrame)?;
            }
            &Frame::Blocked => {
                let type_flags = BLOCKED.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteBlockedFrame)?;
            }
            &Frame::StopWaiting => {
                let type_flags = STOP_WAITING.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWriteStopWaitingFrame)?;
            }
            &Frame::Ping => {
                let type_flags = PING.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::UnableToWritePingFrame)?;
            }
        }
        Ok(())
    }
}
