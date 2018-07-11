use byteorder::{NetworkEndian, WriteBytesExt};
use bytes::{Bytes, BytesMut};
use debugit::DebugIt;
use errors::*;
use smallvec::{Array, SmallVec};
use std::collections::HashSet;
use std::hash::{BuildHasher, Hash};
use std::io::{Cursor, Result as IoResult, Write};

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

    fn bytes_small<A>(&self) -> Result<SmallVec<A>>
    where
        A: Array<Item = u8>,
    {
        let mut small_vec = SmallVec::new();

        self.write(&mut small_vec)?;

        Ok(small_vec)
    }

    fn bytes(&self) -> Result<BytesMut> {
        let bytes = BytesMut::new();

        let mut writer = GrowingBytesMutWriter::new(bytes);

        self.write(&mut writer)?;

        Ok(writer.into_inner())
    }

    fn bytes_vec(&self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();

        self.write(&mut vec)?;

        Ok(vec)
    }
}

struct GrowingBytesMutWriter {
    bytes_mut: BytesMut,
}

impl GrowingBytesMutWriter {
    pub fn new(bytes_mut: BytesMut) -> Self {
        Self { bytes_mut }
    }

    pub fn into_inner(self) -> BytesMut {
        self.bytes_mut
    }
}

impl Write for GrowingBytesMutWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.bytes_mut.extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
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

impl Writable for Bytes {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing bytes {:?}", DebugIt(self));

        writer
            .write_all(self)
            .chain_err(|| ErrorKind::FailedToWriteBytes(self.len()))?;

        debug!("written bytes {:?}", DebugIt(self));

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

impl<E: Writable + Eq + Hash, S: BuildHasher> Writable for HashSet<E, S> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing values {:?}", DebugIt(self));

        for value in self {
            value.write(writer)?;
        }

        debug!("written values {:?}", DebugIt(self));

        Ok(())
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
