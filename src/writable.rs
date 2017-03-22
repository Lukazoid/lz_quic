use errors::*;
use std::io::Write;
use byteorder::{WriteBytesExt, LittleEndian};

pub trait Writable {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()>;

    fn bytes(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        self.write(&mut vec).expect("Writing to a vector should result in no errors");

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
        writer.write_u8(*self).chain_err(|| ErrorKind::UnableToWriteU8(*self))
    }
}

impl Writable for u64 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(*self).chain_err(|| ErrorKind::UnableToWriteU64(*self))
    }
}

impl Writable for str {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(self.as_bytes())
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