use errors::*;
use std::io::{Cursor, Read};
use byteorder::{NetworkEndian, ReadBytesExt};
use primitives::{ReadQuicPrimitives, U24, U48};
use std::marker::PhantomData;
use std::iter::FromIterator;
use debugit::DebugIt;

#[derive(Debug)]
pub struct ReadableIterator<'a, R> {
    length: u64,
    cursor: Cursor<&'a [u8]>,
    _phantom: PhantomData<R>,
}

impl<'a, R: Readable> Iterator for ReadableIterator<'a, R> {
    type Item = Result<R>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.position() < self.length {
            Some(R::read(&mut self.cursor))
        } else {
            None
        }
    }
}

pub trait Readable {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading from bytes {:?}", bytes);

        let mut cursor = Cursor::new(bytes);

        let read_value = Readable::read(&mut cursor)?;

        debug!("read {:?} from bytes {:?}", DebugIt(&read_value), bytes);

        Ok(read_value)
    }

    fn collect<C: FromIterator<Self>, R: Read>(reader: &mut R) -> Result<C>
    where
        Self: Sized,
    {
        trace!("collecting from reader");

        let bytes: Vec<u8> = Readable::read(reader)?;

        let collection = Readable::collect_from_bytes(&bytes[..])?;

        debug!("collected {:?} from reader", DebugIt(&collection));

        Ok(collection)
    }

    fn iterator_from_bytes<'a>(bytes: &'a [u8]) -> ReadableIterator<'a, Self>
    where
        Self: Sized,
    {
        trace!("reading from bytes {:?}", bytes);

        ReadableIterator {
            length: bytes.len() as u64,
            cursor: Cursor::new(bytes),
            _phantom: PhantomData::default(),
        }
    }

    fn collect_from_bytes<C: FromIterator<Self>>(bytes: &[u8]) -> Result<C>
    where
        Self: Sized,
    {
        trace!("collecting from bytes {:?}", bytes);

        let collection = Self::iterator_from_bytes(bytes).collect::<Result<C>>()?;

        debug!(
            "collected {:?} from bytes {:?}",
            DebugIt(&collection),
            bytes
        );

        Ok(collection)
    }
}

impl Readable for Vec<u8> {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading byte vector");

        let mut vec = Vec::new();

        reader.read_to_end(&mut vec)
            .chain_err(|| ErrorKind::FailedToReadBytes)?;

        debug!("read byte vector {:?}", vec);

        Ok(vec)
    }
}

impl Readable for u8 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading byte");
        let byte = reader.read_u8().chain_err(|| ErrorKind::FailedToReadU8)?;

        debug!("read byte {}", byte);

        Ok(byte)
    }
}

impl Readable for u16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading unsigned 16-bit integer");

        let value = reader
            .read_u16::<NetworkEndian>()
            .chain_err(|| ErrorKind::FailedToReadU16)?;

        debug!("read unsigned 16-bit integer {}", value);

        Ok(value)
    }
}

impl Readable for U24 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading unsigned 24-bit integer");

        let value = ReadQuicPrimitives::read_u24::<NetworkEndian>(reader)
            .chain_err(|| ErrorKind::FailedToReadU24)?;

        debug!("read unsigned 24-bit integer {}", value);

        Ok(value)
    }
}

impl Readable for u32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading unsigned 32-bit integer");

        let value = reader
            .read_u32::<NetworkEndian>()
            .chain_err(|| ErrorKind::FailedToReadU32)?;

        debug!("read unsigned 32-bit integer {}", value);

        Ok(value)
    }
}

impl Readable for U48 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading unsigned 48-bit integer");

        let value = reader
            .read_u48::<NetworkEndian>()
            .chain_err(|| ErrorKind::FailedToReadU48)?;

        debug!("read unsigned 48-bit integer {}", value);

        Ok(value)
    }
}

impl Readable for u64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading unsigned 64-bit integer");

        let value = reader
            .read_u64::<NetworkEndian>()
            .chain_err(|| ErrorKind::FailedToReadU64)?;

        debug!("read unsigned 64-bit integer {}", value);

        Ok(value)
    }
}

impl Readable for String {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading string");

        let mut string = String::new();
        reader
            .read_to_string(&mut string)
            .chain_err(|| ErrorKind::FailedToReadString)?;

        debug!("read string {}", string);

        Ok(string)
    }
}
