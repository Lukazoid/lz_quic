use errors::*;
use std::io::{Write, Cursor};
use byteorder::{LittleEndian, WriteBytesExt};
use primitives::{U24, U48, WriteQuicPrimitives};
use smallvec::{Array, SmallVec};

pub trait Writable {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;

    fn write_to_slice(&self, slice: &mut [u8]) {
        self.write(&mut Cursor::new(slice))
            .expect("writing to a slice should result in no errors")
    }

    fn write_to_vec(&self, vec: &mut Vec<u8>) {
        self.write(vec)
            .expect("writing to a vector should result in no errors");
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
        for value in self {
            value.write(writer)?;
        }

        Ok(())
    }
}

impl<A: Array<Item=E>, E: Writable> Writable for SmallVec<A> {
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
        writer
            .write_u8(*self)
            .chain_err(|| ErrorKind::FailedToWriteU8(*self))
    }
}

impl Writable for u16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u16::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU16(*self))
    }
}

impl Writable for U24 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        WriteQuicPrimitives::write_u24::<LittleEndian>(writer, *self)
            .chain_err(|| ErrorKind::FailedToWriteU24(*self))
    }
}

impl Writable for u32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u32::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU32(*self))
    }
}

impl Writable for U48 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u48::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU48(*self))
    }
}

impl Writable for u64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u64::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::FailedToWriteU64(*self))
    }
}

impl Writable for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_all(self.as_bytes())
            .chain_err(|| ErrorKind::FailedToWriteString(self.to_owned()))
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
