use errors::*;
use frames::stream_offset::{StreamOffset, StreamOffsetLength};
use protocol::{Writable};
use frames::{StreamFrame, AckFrame};
use std::io::Write;
use byteorder::WriteBytesExt;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
        trace!("writing frame {:?}", self);

        match self {
            &Frame::Stream(ref stream_frame) => {
                let mut type_flags = STREAM.bits();
                // TODO LH Add the data length field

                let mut payload = Vec::with_capacity(12);

                let stream_id_length = stream_frame
                    .stream_id
                    .write(&mut payload)
                    .chain_err(|| ErrorKind::FailedToWriteStreamId(stream_frame.stream_id))
                    .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;

                // TODO LH Check this is all still correct
                
                let offset_header_length = stream_frame
                    .offset
                    .write(&mut payload)
                    .chain_err(|| ErrorKind::FailedToWriteStreamOffset(stream_frame.offset))
                    .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;

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
                    .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;

                writer
                    .write_all(&payload)
                    .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;
            }
            &Frame::Ack(ref ack_frame) => {
                let type_flags = ACK.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;
            }
            &Frame::Padding => {
                let type_flags = FrameTypeFlags::empty().bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWritePaddingFrame)?;
            }
            &Frame::ResetStream => {
                let type_flags = RESET_STREAM.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteResetStreamFrame)?;
            }
            &Frame::ConnectionClose => {
                let type_flags = CONNECTION_CLOSE.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteConnectionCloseFrame)?;
            }
            &Frame::GoAway => {
                let type_flags = GO_AWAY.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteGoAwayFrame)?;
            }
            &Frame::WindowUpdate => {
                let type_flags = WINDOW_UPDATE.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteWindowUpdateFrame)?;
            }
            &Frame::Blocked => {
                let type_flags = BLOCKED.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteBlockedFrame)?;
            }
            &Frame::StopWaiting => {
                let type_flags = STOP_WAITING.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWriteStopWaitingFrame)?;
            }
            &Frame::Ping => {
                let type_flags = PING.bits();
                writer
                    .write_u8(type_flags)
                    .chain_err(|| ErrorKind::FailedToWritePingFrame)?;
            }
        }

        debug!("written frame {:?}", self);
        
        Ok(())
    }
}
