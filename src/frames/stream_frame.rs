use bytes::Bytes;
use conv::ValueInto;
use errors::*;
use frames::StreamOffset;
use protocol::{Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamFrame {
    pub finished: bool,
    pub offset: StreamOffset,
    pub stream_id: StreamId,
    pub data: Bytes,
}

impl StreamFrame {
    pub fn has_offset(&self) -> bool {
        !self.offset.is_zero()
    }
}

#[derive(Debug)]
pub struct ReadStreamFrameContext {
    pub is_offset_present: bool,
    pub is_length_present: bool,
    pub finished: bool,
}

impl Readable for StreamFrame {
    type Context = ReadStreamFrameContext;

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading stream frame");

        let stream_id = Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStreamFrame)?;
        let offset = if context.is_offset_present {
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStreamFrame)?
        } else {
            0u64.into()
        };

        let data = if context.is_length_present {
            let length = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadStreamFrame)?;
            Readable::read(&mut reader.take(length.into_inner()))
        } else {
            Readable::read(reader)
        }.chain_err(|| ErrorKind::FailedToReadStreamFrame)?;

        let stream_frame = Self {
            finished: context.finished,
            offset,
            stream_id,
            data,
        };
        debug!("read stream frame {:?}", stream_frame);

        Ok(stream_frame)
    }
}

impl Writable for StreamFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stream frame {:?}", self);

        self.stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;

        if self.has_offset() {
            self.offset
                .write(writer)
                .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;
        }

        let length: VarInt = self.data
            .len()
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;
        length
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;

        self.data
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStreamFrame)?;

        debug!("written stream frame {:?}", self);

        Ok(())
    }
}
