use errors::*;
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    NoError,
    InternalError,
    ServerBusy,
    FlowControlError,
    StreamIdError,
    StreamStateError,
    FinalOffsetError,
    FrameFormatError,
    TransportParameterError,
    VersionNegotationError,
    ProtocolViolation,
    UnsolicitedPathResponse,
    FrameError(u8),
}

impl Readable for ErrorCode {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading error code");

        let value = u16::read(reader).chain_err(|| ErrorKind::FailedToReadErrorCode)?;

        let error_code = match value {
            0x0 => ErrorCode::NoError,
            0x1 => ErrorCode::InternalError,
            0x2 => ErrorCode::ServerBusy,
            0x3 => ErrorCode::FlowControlError,
            0x4 => ErrorCode::StreamIdError,
            0x5 => ErrorCode::StreamStateError,
            0x6 => ErrorCode::FinalOffsetError,
            0x7 => ErrorCode::FrameFormatError,
            0x8 => ErrorCode::TransportParameterError,
            0x9 => ErrorCode::VersionNegotationError,
            0xa => ErrorCode::ProtocolViolation,
            0xb => ErrorCode::UnsolicitedPathResponse,
            0x100...0x1ff => {
                let frame_type = value as u8;
                ErrorCode::FrameError(frame_type)
            }
            _ => bail!(ErrorKind::FailedToReadErrorCode),
        };

        debug!("read error code {:?}", error_code);

        Ok(error_code)
    }
}
impl Writable for ErrorCode {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing error code {:?}", self);

        let bytes: u16 = match self {
            ErrorCode::NoError => 0x0,
            ErrorCode::InternalError => 0x1,
            ErrorCode::ServerBusy => 0x2,
            ErrorCode::FlowControlError => 0x3,
            ErrorCode::StreamIdError => 0x4,
            ErrorCode::StreamStateError => 0x5,
            ErrorCode::FinalOffsetError => 0x6,
            ErrorCode::FrameFormatError => 0x7,
            ErrorCode::TransportParameterError => 0x8,
            ErrorCode::VersionNegotationError => 0x9,
            ErrorCode::ProtocolViolation => 0xa,
            ErrorCode::UnsolicitedPathResponse => 0xb,
            ErrorCode::FrameError(frame_type) => (0x1u16 << 8) | *frame_type as u16,
        };

        bytes
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteErrorCode(bytes))?;

        debug!("written error code {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::ErrorCode;
    use protocol;

    #[test]
    fn round_trip_frame_error() {
        protocol::test_write_read(&ErrorCode::FrameError(208)).unwrap();
    }
}
