use conv::ValueInto;
use errors::*;
use frames::StreamOffset;
use protocol::{Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamBlockedFrame {
    pub stream_id: StreamId,
    pub offset: StreamOffset,
}

impl Readable for StreamBlockedFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading stream blocked frame");

        let stream_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStreamBlockedFrame)?;
        let offset =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStreamBlockedFrame)?;

        let stream_blocked_frame = Self { stream_id, offset };

        debug!("read stream blocked frame {:?}", stream_blocked_frame);

        Ok(stream_blocked_frame)
    }
}

impl Writable for StreamBlockedFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stream blocked frame {:?}", self);

        self.stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStreamBlockedFrame)?;

        self.offset
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStreamBlockedFrame)?;

        debug!("written stream blocked frame {:?}", self);

        Ok(())
    }
}
