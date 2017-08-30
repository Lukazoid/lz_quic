use errors::*;
use std::io::{Cursor, Read};
use byteorder::{LittleEndian, ReadBytesExt};
use primitives::{ReadQuicPrimitives, U24, U48};
use std::marker::PhantomData;
use std::iter::FromIterator;

#[derive(Debug)]
pub struct ReadableIterator<'a, R> {
    length: u64,
    cursor: Cursor<&'a[u8]>,
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
        let mut cursor = Cursor::new(bytes);

        Readable::read(&mut cursor)
    }

    fn iterator_from_bytes<'a>(bytes: &'a [u8]) -> ReadableIterator<'a, Self>
    where
        Self: Sized,
    {
        ReadableIterator{
            length: bytes.len() as u64,
            cursor: Cursor::new(bytes),
            _phantom: PhantomData::default()
        }
    }
    
    fn collect_from_bytes<C: FromIterator<Self>>(bytes: &[u8]) -> Result<C>
    where
        Self: Sized,
    {
        let mut error = None;
        
        let collection: C = Self::iterator_from_bytes(bytes)
            .scan(&mut error, |error, result| {
                match result {
                    Ok(value) => Some(value),
                    Err(err) => {
                        **error = Some(err);
                        None
                    }
                }
            })
            .collect();
        
        error.map(Err)
            .unwrap_or(Ok(collection))
    }

}

impl Readable for Vec<u8> {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        let mut vec = Vec::new();

        reader
            .read_to_end(&mut vec)
            .map(|_| vec)
            .chain_err(|| ErrorKind::FailedToReadBytes)
    }
}

impl Readable for u8 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        reader.read_u8().chain_err(|| ErrorKind::FailedToReadU8)
    }
}

impl Readable for u16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        reader
            .read_u16::<LittleEndian>()
            .chain_err(|| ErrorKind::FailedToReadU16)
    }
}

impl Readable for U24 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        ReadQuicPrimitives::read_u24::<LittleEndian>(reader)
            .chain_err(|| ErrorKind::FailedToReadU24)
    }
}

impl Readable for u32 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        reader
            .read_u32::<LittleEndian>()
            .chain_err(|| ErrorKind::FailedToReadU32)
    }
}

impl Readable for U48 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        reader
            .read_u48::<LittleEndian>()
            .chain_err(|| ErrorKind::FailedToReadU48)
    }
}

impl Readable for u64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        reader
            .read_u64::<LittleEndian>()
            .chain_err(|| ErrorKind::FailedToReadU64)
    }
}

impl Readable for String {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
    {
        let mut string = String::new();
        reader
            .read_to_string(&mut string)
            .map(|_| string)
            .chain_err(|| ErrorKind::FailedToReadString)
    }
}