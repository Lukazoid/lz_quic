use errors::*;
use std::io::{Cursor, Write};
use byteorder::{NetworkEndian, WriteBytesExt};
use smallvec::{Array, SmallVec};
use debugit::DebugIt;

pub trait Writable {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;

    fn write_to_slice(&self, slice: &mut [u8]) {
        trace!("writing {:?} to slice", DebugIt(self));

        let mut cursor = Cursor::new(slice);
        self.write(&mut cursor)
            .expect("writing to a slice should result in no errors");

        let slice = cursor.into_inner();

        debug!("written {:?} to slice {:?}", DebugIt(self), slice);
    }

    fn write_to_small_vec<A: Array<Item = u8>>(&self, small_vec: &mut SmallVec<A>) {
        trace!("writing {:?} to small vector", DebugIt(self));

        self.write(small_vec)
            .expect("writing to a small vector should result in no errors");

        debug!(
            "written {:?} to small vector {:?}",
            DebugIt(self),
            small_vec
        );
    }

    fn write_to_vec(&self, vec: &mut Vec<u8>) {
        trace!("writing {:?} to vector", DebugIt(self));

        self.write(vec)
            .expect("writing to a vector should result in no errors");

        debug!("written {:?} to vector {:?}", DebugIt(self), vec);
    }

    fn bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.write(&mut vec)
            .expect("writing to a vector should result in no errors");

        vec
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
