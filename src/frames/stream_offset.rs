use conv::ValueFrom;
use errors::*;
use protocol::{Readable, VarInt, Writable};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StreamOffset(u64);

impl StreamOffset {
    pub fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u64> for StreamOffset {
    fn from(value: u64) -> StreamOffset {
        StreamOffset(value)
    }
}

impl Writable for StreamOffset {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stream offset {:?}", self);

        let var_int = VarInt::value_from(self.0)?;

        var_int.write(writer)?;

        debug!("written stream offset {:?}", self);

        Ok(())
    }
}

impl Readable for StreamOffset {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading stream offset");

        let var_int = VarInt::read(reader)?;

        let stream_offset = StreamOffset(var_int.into_inner());

        debug!("read stream offset {:?}", stream_offset);

        Ok(stream_offset)
    }
}

impl Display for StreamOffset {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}
