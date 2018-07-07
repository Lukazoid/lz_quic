use byteorder::{NetworkEndian, ReadBytesExt};
use bytes::{Bytes, BytesMut};
use conv::ValueFrom;
use debugit::DebugIt;
use errors::*;
use smallvec::{Array, SmallVec};
use std::io::{self, Cursor, Read};
use std::iter::FromIterator;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct ReadableIterator<'a, R> {
    length: u64,
    cursor: Cursor<&'a [u8]>,
    _phantom: PhantomData<R>,
}

impl<'a, R: Readable> Iterator for ReadableIterator<'a, R>
where
    R::Context: Default,
{
    type Item = Result<R>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.position() < self.length {
            Some(R::read(&mut self.cursor))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ReadableWithContextIterator<'a, R: Readable>
where
    R::Context: 'a,
{
    length: u64,
    cursor: Cursor<&'a [u8]>,
    context: &'a R::Context,
    _phantom: PhantomData<R>,
}

impl<'a, R: Readable> Iterator for ReadableWithContextIterator<'a, R> {
    type Item = Result<R>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor.position() < self.length {
            Some(R::read_with_context(&mut self.cursor, self.context))
        } else {
            None
        }
    }
}

pub trait Readable {
    type Context;

    fn read<R: Read>(reader: &mut R) -> Result<Self>
    where
        Self: Sized,
        Self::Context: Default,
    {
        Self::read_with_context(reader, &Default::default())
    }

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self>
    where
        Self: Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized,
        Self::Context: Default,
    {
        Self::from_bytes_with_context(bytes, &Default::default())
    }

    fn from_bytes_with_context(bytes: &[u8], context: &Self::Context) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading from bytes {:?}", bytes);

        let mut cursor = Cursor::new(bytes);

        let read_value = Readable::read_with_context(&mut cursor, context)?;

        debug!("read {:?} from bytes {:?}", DebugIt(&read_value), bytes);

        Ok(read_value)
    }
    fn collect<C: FromIterator<Self>, R: Read>(reader: &mut R) -> Result<C>
    where
        Self: Sized,
        Self::Context: Default,
    {
        Self::collect_with_context(reader, &Default::default())
    }

    fn collect_with_context<C: FromIterator<Self>, R: Read>(
        reader: &mut R,
        context: &Self::Context,
    ) -> Result<C>
    where
        Self: Sized,
    {
        trace!("collecting from reader");

        let bytes: Vec<u8> = Readable::read(reader)?;

        let collection = Readable::collect_from_bytes_with_context(&bytes[..], context)?;

        debug!("collected {:?} from reader", DebugIt(&collection));

        Ok(collection)
    }

    fn iterator_from_bytes<'a>(bytes: &'a [u8]) -> ReadableIterator<'a, Self>
    where
        Self: Sized,
        Self::Context: 'a,
    {
        trace!("reading from bytes {:?}", bytes);

        ReadableIterator {
            length: u64::value_from(bytes.len()).unwrap(),
            cursor: Cursor::new(bytes),
            _phantom: PhantomData::default(),
        }
    }

    fn iterator_from_bytes_with_context<'a>(
        bytes: &'a [u8],
        context: &'a Self::Context,
    ) -> ReadableWithContextIterator<'a, Self>
    where
        Self: Sized,
        Self::Context: 'a,
    {
        trace!("reading from bytes {:?}", bytes);

        ReadableWithContextIterator {
            length: u64::value_from(bytes.len()).unwrap(),
            cursor: Cursor::new(bytes),
            context: context,
            _phantom: PhantomData::default(),
        }
    }

    fn collect_from_bytes<C: FromIterator<Self>>(bytes: &[u8]) -> Result<C>
    where
        Self: Sized,
        Self::Context: Default,
    {
        Self::collect_from_bytes_with_context(bytes, &Default::default())
    }

    fn collect_from_bytes_with_context<C: FromIterator<Self>>(
        bytes: &[u8],
        context: &Self::Context,
    ) -> Result<C>
    where
        Self: Sized,
    {
        trace!("collecting from bytes {:?}", bytes);

        let collection =
            Self::iterator_from_bytes_with_context(bytes, context).collect::<Result<C>>()?;

        debug!(
            "collected {:?} from bytes {:?}",
            DebugIt(&collection),
            bytes
        );

        Ok(collection)
    }
}

impl Readable for Bytes {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
    where
        Self: Sized,
    {
        let vec: Vec<u8> = Readable::read(reader)?;

        Ok(vec.into())
    }
}

impl Readable for BytesMut {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
    where
        Self: Sized,
    {
        let vec: Vec<u8> = Readable::read(reader)?;

        Ok(vec.into())
    }
}

impl Readable for Vec<u8> {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading byte vector");

        let mut vec = Vec::new();

        reader
            .read_to_end(&mut vec)
            .chain_err(|| ErrorKind::FailedToReadBytes)?;

        debug!("read byte vector {:?}", vec);

        Ok(vec)
    }
}

impl<A: Array<Item = u8>> Readable for SmallVec<A> {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading byte small vector");

        let mut small_vec = SmallVec::new();
        io::copy(reader, &mut small_vec).chain_err(|| ErrorKind::FailedToReadBytes)?;

        debug!("read byte small vector {:?}", small_vec);

        Ok(small_vec)
    }
}

impl Readable for u8 {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
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
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
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

impl Readable for u32 {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
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

impl Readable for u64 {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
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

impl Readable for u128 {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
    where
        Self: Sized,
    {
        trace!("reading unsigned 128-bit integer");

        let value = reader
            .read_u128::<NetworkEndian>()
            .chain_err(|| ErrorKind::FailedToReadU128)?;

        debug!("read unsigned 128-bit integer {}", value);

        Ok(value)
    }
}

impl Readable for String {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self>
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
