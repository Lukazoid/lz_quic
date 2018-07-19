use conv::{ValueFrom, ValueInto};
use errors::*;
use protocol::{Readable, VarInt, Writable};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::{Read, Write};
use std::ops::{Add, AddAssign};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StreamOffset(pub VarInt);

impl StreamOffset {
    pub fn into_inner(self) -> VarInt {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0.into_inner() == 0
    }
}

impl From<StreamOffset> for VarInt {
    fn from(value: StreamOffset) -> Self {
        value.into_inner()
    }
}

impl From<StreamOffset> for u64 {
    fn from(value: StreamOffset) -> Self {
        value.into_inner().into_inner()
    }
}

impl<T: ValueInto<VarInt>> Add<T> for StreamOffset {
    type Output = StreamOffset;

    fn add(self, rhs: T) -> Self::Output {
        StreamOffset(self.into_inner() + rhs)
    }
}

impl<T: ValueInto<VarInt>> AddAssign<T> for StreamOffset {
    fn add_assign(&mut self, rhs: T) {
        *self = *self + rhs;
    }
}

impl From<VarInt> for StreamOffset {
    fn from(value: VarInt) -> StreamOffset {
        StreamOffset(value)
    }
}

impl From<u8> for StreamOffset {
    fn from(value: u8) -> StreamOffset {
        StreamOffset(value.into())
    }
}

impl From<u16> for StreamOffset {
    fn from(value: u16) -> StreamOffset {
        StreamOffset(value.into())
    }
}

impl From<u32> for StreamOffset {
    fn from(value: u32) -> StreamOffset {
        StreamOffset(value.into())
    }
}

impl Writable for StreamOffset {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stream offset {:?}", self);

        self.0.write(writer)?;

        debug!("written stream offset {:?}", self);

        Ok(())
    }
}

impl Readable for StreamOffset {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading stream offset");

        let var_int = VarInt::read(reader)?;

        let stream_offset = StreamOffset(var_int);

        debug!("read stream offset {:?}", stream_offset);

        Ok(stream_offset)
    }
}

impl Display for StreamOffset {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}
