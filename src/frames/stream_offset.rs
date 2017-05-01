use errors::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::convert::TryFrom;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StreamOffsetLength {
    ZeroBytes,
    TwoBytes,
    ThreeBytes,
    FourBytes,
    FiveBytes,
    SixBytes,
    SevenBytes,
    EightBytes,
}

impl TryFrom<usize> for StreamOffsetLength {
    type Error = Error;

    fn try_from(value: usize) -> Result<StreamOffsetLength> {
        let length = match value {
            0 => StreamOffsetLength::ZeroBytes,
            1 | 2 => StreamOffsetLength::TwoBytes,
            3 => StreamOffsetLength::ThreeBytes,
            4 => StreamOffsetLength::FourBytes,
            5 => StreamOffsetLength::FiveBytes,
            6 => StreamOffsetLength::SixBytes,
            7 => StreamOffsetLength::SevenBytes,
            8 => StreamOffsetLength::EightBytes,
            _ => bail!(ErrorKind::InvalidStreamOffsetLength(value)),
        };

        Ok(length)
    }
}

impl From<StreamOffsetLength> for usize {
    fn from(value: StreamOffsetLength) -> usize {
        match value {
            StreamOffsetLength::ZeroBytes => 0,
            StreamOffsetLength::TwoBytes => 2,
            StreamOffsetLength::ThreeBytes => 3,
            StreamOffsetLength::FourBytes => 4,
            StreamOffsetLength::FiveBytes => 5,
            StreamOffsetLength::SixBytes => 6,
            StreamOffsetLength::SevenBytes => 7,
            StreamOffsetLength::EightBytes => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StreamOffset(u64);

impl From<u64> for StreamOffset {
    fn from(value: u64) -> StreamOffset {
        StreamOffset(value)
    }
}

impl StreamOffset {
    pub fn read<R: Read>(reader: &mut R, length: StreamOffsetLength) -> Result<StreamOffset> {
        let byte_count: usize = length.into();

        let inner = reader
            .read_uint::<LittleEndian>(byte_count)
            .chain_err(|| ErrorKind::UnableToReadStreamOffset)? as u64;

        Ok(StreamOffset(inner))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<StreamOffsetLength> {
        let offset = self.0;
        let leading_zeros = offset.leading_zeros();
        let leading_bytes = leading_zeros / 8;
        let header_length = 8 - leading_bytes;

        let header_length = StreamOffsetLength::try_from(header_length as usize)?;

        let byte_count = header_length.into();
        if byte_count > 0 {
            writer
                .write_uint::<LittleEndian>(offset, byte_count)
                .chain_err(|| ErrorKind::UnableToWriteBytes(byte_count))?;
        }
        Ok(header_length)
    }
}

impl Display for StreamOffset {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    macro_rules! write_stream_offset_header_tests {
        ($($name:ident: $value:expr,)*) => {
            mod write_stream_offset_header {
                use frames::stream_offset::{StreamOffset, StreamOffsetLength};
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (StreamOffset, StreamOffsetLength) = ($value.0.into(), $value.1);
                    let mut vector = Vec::new();
                    let length = input.write(&mut vector).unwrap();
                    assert_eq!(expected, length);
                }
            )*
            }
        }
    }

    write_stream_offset_header_tests!{
        zero_fits_into_zero_bytes : (0u64, StreamOffsetLength::ZeroBytes),
        overflows_to_two_bytes : (1u64, StreamOffsetLength::TwoBytes),

        maximum_of_two_bytes_fits_into_two_bytes : (65535u64, StreamOffsetLength::TwoBytes),
        overflows_to_three_bytes : (65536u64, StreamOffsetLength::ThreeBytes),

        maximum_of_three_bytes_fits_into_three_bytes : (16777215u64, StreamOffsetLength::ThreeBytes),
        overflows_to_four_bytes : (16777216u64, StreamOffsetLength::FourBytes),

        maximum_of_four_bytes_fits_into_four_bytes : (4294967295u64, StreamOffsetLength::FourBytes),
        overflows_to_five_bytes : (4294967296u64, StreamOffsetLength::FiveBytes),

        maximum_of_five_bytes_fits_into_five_bytes : (1099511627775u64, StreamOffsetLength::FiveBytes),
        overflows_to_six_bytes : (1099511627776u64, StreamOffsetLength::SixBytes),

        maximum_of_six_bytes_fits_into_six_bytes : (281474976710655u64, StreamOffsetLength::SixBytes),
        overflows_to_seven_bytes : (281474976710656u64, StreamOffsetLength::SevenBytes),

        maximum_of_seven_bytes_fits_into_seven_bytes : (72057594037927935u64, StreamOffsetLength::SevenBytes),
        overflows_to_eight_bytes : (72057594037927936u64, StreamOffsetLength::EightBytes),
    }
}