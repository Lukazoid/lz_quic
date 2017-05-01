use errors::*;
use std::io::Write;
use byteorder::{WriteBytesExt, LittleEndian};
use primitives::u24::U24;
use primitives::u48::U48;
use write_quic_primitives::WriteQuicPrimitives;

pub trait Writable {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;

    fn bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.write(&mut vec)
            .expect("Writing to a vector should result in no errors");

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

impl<E: Writable> Writable for Vec<E> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_slice().write(writer)
    }
}


impl Writable for u8 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u8(*self)
            .chain_err(|| ErrorKind::UnableToWriteU8(*self))
    }
}

impl Writable for u16 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u16::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::UnableToWriteU16(*self))
    }
}

impl Writable for U24 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u24::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::UnableToWriteU24(*self))
    }
}

impl Writable for u32 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u32::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::UnableToWriteU32(*self))
    }
}

impl Writable for U48 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u48::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::UnableToWriteU48(*self))
    }
}

impl Writable for u64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_u64::<LittleEndian>(*self)
            .chain_err(|| ErrorKind::UnableToWriteU64(*self))
    }
}

impl Writable for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_all(self.as_bytes())
            .chain_err(|| ErrorKind::UnableToWriteString(self.to_string()))
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
