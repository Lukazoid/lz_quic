use errors::*;
use rand::Rng;
use std::fmt::{Display, Formatter, Result as FmtResult};
use byteorder::{ReadBytesExt, LittleEndian, WriteBytesExt};
use std::io::{Read, Write};
use conv::TryFrom;

/// An enum for the serialized length of a `StreamId`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum StreamIdLength {
    OneByte,
    TwoBytes,
    ThreeBytes,
    FourBytes,
}

impl TryFrom<usize> for StreamIdLength {
    type Err = Error;

    fn try_from(value: usize) -> Result<StreamIdLength> {
        let length = match value {
            1 => StreamIdLength::OneByte,
            2 => StreamIdLength::TwoBytes,
            3 => StreamIdLength::ThreeBytes,
            4 => StreamIdLength::FourBytes,

            _ => bail!(ErrorKind::InvalidStreamIdLength(value)),
        };

        Ok(length)
    }
}

impl From<StreamIdLength> for usize {
    fn from(value: StreamIdLength) -> usize {
        match value {
            StreamIdLength::OneByte => 1,
            StreamIdLength::TwoBytes => 2,
            StreamIdLength::ThreeBytes => 3,
            StreamIdLength::FourBytes => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct StreamId(u32);

impl StreamId {
    pub fn generate<R: Rng>(rng: &mut R) -> StreamId {
        let inner = rng.next_u32();

        StreamId(inner)
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<StreamIdLength> {
        let inner = self.0;

        let leading_zeros = inner.leading_zeros();
        let leading_bytes = leading_zeros / 8;
        let header_length = 4 - leading_bytes;

        let header_length = StreamIdLength::try_from(header_length as usize)?;

        let byte_count = header_length.into();
        assert!(byte_count > 0);
        writer
            .write_uint::<LittleEndian>(inner as u64, byte_count)
            .chain_err(|| ErrorKind::UnableToWriteBytes(byte_count))?;

        Ok(header_length)
    }

    pub fn read<R: Read>(reader: &mut R, length: StreamIdLength) -> Result<StreamId> {
        let byte_count: usize = length.into();
        let inner = reader
            .read_uint::<LittleEndian>(byte_count)
            .chain_err(|| ErrorKind::UnableToReadBytes)? as u32;

        Ok(StreamId(inner))
    }
}

impl Display for StreamId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

