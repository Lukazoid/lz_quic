use conv::ValueInto;
use errors::*;
use frames::StreamOffset;
use protocol::{Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ResetStreamFrame {
    pub stream_id: StreamId,
    pub application_error_code: u16,
    pub final_offset: StreamOffset,
}

impl Readable for ResetStreamFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading reset stream frame");

        let stream_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadResetStreamFrame)?;
        let application_error_code =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadResetStreamFrame)?;
        let final_offset =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadResetStreamFrame)?;

        let reset_stream_frame = Self {
            stream_id,
            application_error_code,
            final_offset,
        };

        debug!("read reset stream frame {:?}", reset_stream_frame);

        Ok(reset_stream_frame)
    }
}

impl Writable for ResetStreamFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing reset stream frame {:?}", self);

        self.stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteResetStreamFrame)?;
        self.application_error_code
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteResetStreamFrame)?;
        self.final_offset
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteResetStreamFrame)?;

        debug!("written reset stream frame {:?}", self);

        Ok(())
    }
}
