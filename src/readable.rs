use errors::*;
use std::io::{Read, Cursor};
use byteorder::{LittleEndian, ReadBytesExt};

pub trait Readable {
    fn read<R: Read>(reader: &mut R) -> Result<Self> where Self: Sized;

    fn from_bytes(bytes: &[u8]) -> Result<Self>
        where Self: Sized
    {
        let mut cursor = Cursor::new(bytes);

        Readable::read(&mut cursor)
    }

    fn collection_from_bytes(bytes: &[u8]) -> Result<Vec<Self>>
        where Self: Sized
    {
        let length = bytes.len() as u64;
        let mut cursor = Cursor::new(bytes);
        let mut vec = Vec::new();
        while cursor.position() < length {
            let element = Readable::read(&mut cursor)?;
            vec.push(element);
        }

        Ok(vec)
    }
}

impl Readable for u64 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
        where Self: Sized
    {
        reader.read_u64::<LittleEndian>().chain_err(|| ErrorKind::UnableToReadU64)
    }
}

impl Readable for String {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
        where Self: Sized
    {
        let mut string = String::new();
        reader.read_to_string(&mut string)
            .map(|_| string)
            .chain_err(|| ErrorKind::UnableToReadString)
    }
}

impl Readable for u16 {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
        where Self: Sized
    {
        reader.read_u16::<LittleEndian>()
            .chain_err(|| ErrorKind::UnableToReadU16)
    }
}

impl Readable for Vec<u8> {
    fn read<R: Read>(reader: &mut R) -> Result<Self>
        where Self: Sized
    {
        let mut vec = Vec::new();
        reader.read_to_end(&mut vec)
            .map(|_| vec)
            .chain_err(||ErrorKind::UnableToReadBytes)
    }
}