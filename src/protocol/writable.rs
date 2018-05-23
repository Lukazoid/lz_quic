use byteorder::{NetworkEndian, WriteBytesExt};
use debugit::DebugIt;
use errors::*;
use smallvec::{Array, SmallVec};
use std::io::{Cursor, Write};

pub trait Writable {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;

    fn write_to_slice(&self, slice: &mut [u8]) -> Result<()> {
        trace!("writing {:?} to slice", DebugIt(self));

        let mut cursor = Cursor::new(slice);
        self.write(&mut cursor)?;

        let slice = cursor.into_inner();

        debug!("written {:?} to slice {:?}", DebugIt(self), slice);

        Ok(())
    }

    fn write_to_small_vec<A: Array<Item = u8>>(&self, small_vec: &mut SmallVec<A>) -> Result<()> {
        trace!("writing {:?} to small vector", DebugIt(self));

        self.write(small_vec)?;

        debug!(
            "written {:?} to small vector {:?}",
            DebugIt(self),
            small_vec
        );

        Ok(())
    }

    fn write_to_vec(&self, vec: &mut Vec<u8>) -> Result<()> {
        trace!("writing {:?} to vector", DebugIt(self));

        self.write(vec)?;

        debug!("written {:?} to vector {:?}", DebugIt(self), vec);

        Ok(())
    }

    // TODO LH Change this so it returns bytes::Bytes
    fn bytes(&self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.write_to_vec(&mut vec)?;

        Ok(vec)
    }

    fn bytes_small<A: Array<Item = u8>>(&self) -> Result<SmallVec<A>> {
        let mut small_vec = SmallVec::new();
        self.write_to_small_vec(&mut small_vec)?;

        Ok(small_vec)
    }
}

impl<E: Writable> Writable for Option<E> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        if let Some(value) = self {
            value.write(writer)?;
        }

        Ok(())
    }
}

impl<E: Writable> Writable for [E] {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing values {:?}", DebugIt(self));

        for value in self {
            value.write(writer)?;
        }

        debug!("written values {:?}", DebugIt(self));

        Ok(())
    }
}

impl<A: Array<Item = E>, E: Writable> Writable for SmallVec<A> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_ref().write(writer)
    }
}

impl<E: Writable> Writable for Vec<E> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_slice().write(writer)
    }
}

impl Writable for u8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing byte {}", self);

        writer
            .write_u8(*self)
            .chain_err(|| ErrorKind::FailedToWriteU8(*self))?;

        debug!("written byte {}", self);

        Ok(())
    }
}

impl Writable for u16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing 16-bit unsigned integer {}", self);

        writer
            .write_u16::<NetworkEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU16(*self))?;

        debug!("written 16-bit unsigned integer {}", self);

        Ok(())
    }
}

impl Writable for u32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing 32-bit unsigned integer {}", self);

        writer
            .write_u32::<NetworkEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU32(*self))?;

        debug!("written 32-bit unsigned integer {}", self);

        Ok(())
    }
}

impl Writable for u64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing 64-bit unsigned integer {}", self);

        writer
            .write_u64::<NetworkEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU64(*self))?;

        debug!("written 64-bit unsigned integer {}", self);

        Ok(())
    }
}

impl Writable for u128 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing 128-bit unsigned integer {}", self);

        writer
            .write_u128::<NetworkEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU128(*self))?;

        debug!("written 128-bit unsigned integer {}", self);

        Ok(())
    }
}

impl Writable for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing string {}", self);

        writer
            .write_all(self.as_bytes())
            .chain_err(|| ErrorKind::FailedToWriteString(self.to_owned()))?;

        debug!("written string {}", self);

        Ok(())
    }
}

impl Writable for String {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        (self as &str).write(writer)
    }
}

impl<'a, T: Writable + 'a> Writable for &'a T {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        (*self).write(writer)
    }
}
