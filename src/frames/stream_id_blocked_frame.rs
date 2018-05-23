use conv::ValueInto;
use errors::*;
use protocol::{Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamIdBlockedFrame {
    pub stream_id: StreamId,
}

impl Readable for StreamIdBlockedFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading stream id blocked frame");

        let stream_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStreamIdBlockedFrame)?;
        let offset =
            VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadStreamIdBlockedFrame)?;

        let stream_id_blocked_frame = Self { stream_id };

        debug!("read stream id blocked frame {:?}", stream_id_blocked_frame);

        Ok(stream_id_blocked_frame)
    }
}

impl Writable for StreamIdBlockedFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stream id blocked frame {:?}", self);

        self.stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStreamIdBlockedFrame)?;

        debug!("written stream id blocked frame {:?}", self);

        Ok(())
    }
}
