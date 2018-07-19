use errors::*;
use protocol::{Readable, StreamOffset, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BlockedFrame {
    pub offset: StreamOffset,
}

impl Readable for BlockedFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading blocked frame");

        let offset = Readable::read(reader).chain_err(|| ErrorKind::FailedToReadBlockedFrame)?;

        let blocked_frame = Self { offset };

        debug!("read blocked frame {:?}", blocked_frame);

        Ok(blocked_frame)
    }
}

impl Writable for BlockedFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing blocked frame {:?}", self);

        self.offset
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteBlockedFrame)?;

        debug!("written blocked frame {:?}", self);

        Ok(())
    }
}
