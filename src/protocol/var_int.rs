use errors::*;
use protocol::{Readable, Writable};
use num::{FromPrimitive, Unsigned};
use std::io::{Read, Write};
use byteorder::{NetworkEndian, ReadBytesExt};
use std::mem;
use conv::TryFrom;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VarInt<T>(T);

impl<T> VarInt<T> {
    fn unwrap(self) -> T {
        self.0
    }
}

impl<T> Deref for VarInt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! from_primitive {
    ($type:ty) => {
        impl From<$type> for VarInt<$type> {
            fn from(value: $type) -> VarInt<$type> {
                VarInt(value)
            }
        }
    }
}

from_primitive!(u8);
from_primitive!(u16);
from_primitive!(u32);

impl<T: Copy + Into<u64>> TryFrom<T> for VarInt<T> {
    type Err = Error;

    fn try_from(value: T) -> Result<VarInt<T>> {
        let int_value = value.into();

        if int_value > 4611686018427387903 {
            bail!(ErrorKind::IntegerValueIsTooLargeToBeStoredAsAVarInt(
                int_value
            ));
        }

        Ok(VarInt(value))
    }
}

impl<T: FromPrimitive + Unsigned> Readable for VarInt<T> {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let first_byte = reader.read_u8().chain_err(|| ErrorKind::FailedToReadBytes)?;

        let length_flag = first_byte & 0b11000000;
        let first_byte_value = (first_byte & 0b00111111) as u64;

        let total_length = match length_flag >> 6 {
            0b00 => 1usize,
            0b01 => 2usize,
            0b10 => 4usize,
            0b11 => 8usize,
            _ => unreachable!(),
        };

        let remaining_bytes_count = total_length - 1;

        let all_bytes = if remaining_bytes_count > 0 {
            let next_bytes = reader
                .read_uint::<NetworkEndian>(remaining_bytes_count)
                .chain_err(|| ErrorKind::FailedToReadBytes)?;

            (first_byte_value << (remaining_bytes_count * 8)) | next_bytes
        } else {
            first_byte_value
        };

        if let Some(inner) = T::from_u64(all_bytes) {
            Ok(VarInt(inner))
        } else {
            bail!(ErrorKind::VarIntValueIsTooLargeToFitInIntegerOfSize(
                all_bytes,
                mem::size_of::<T>()
            ))
        }
    }
}

impl<T: Copy + Into<u64>> Writable for VarInt<T> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let int_value: u64 = self.0.into();

        match int_value {
            0...63 => (int_value as u8).write(writer)?,
            64...16383 => (0x4000 | int_value as u16).write(writer)?,
            16384...1073741823 => (0x80000000 | int_value as u32).write(writer)?,
            1073741824...4611686018427387903 => (0xC000000000000000 | int_value).write(writer)?,
            _ => bail!(ErrorKind::IntegerValueIsTooLargeToBeStoredAsAVarInt(
                int_value
            )),
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use errors::*;
    use super::VarInt;
    use protocol::{Readable, Writable};
    use std::io::Cursor;
    use conv::TryFrom;

    #[test]
    fn read_var_int_of_1_byte_reads_correctly() {
        let mut reader = Cursor::new(vec![0x25, 0xff, 0xff]);

        let var_int: VarInt<u8> = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(37))
    }

    #[test]
    fn read_var_int_of_2_bytes_reads_correctly() {
        let mut reader = Cursor::new(vec![0x7b, 0xbd, 0xff, 0xff]);

        let var_int: VarInt<u32> = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(15293))
    }

    #[test]
    fn read_var_int_of_4_bytes_reads_correctly() {
        let mut reader = Cursor::new(vec![0x9d, 0x7f, 0x3e, 0x7d, 0xff, 0xff]);

        let var_int: VarInt<u32> = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(494878333))
    }

    #[test]
    fn read_var_int_of_8_bytes_reads_correctly() {
        let mut reader = Cursor::new(vec![
            0xc2,
            0x19,
            0x7c,
            0x5e,
            0xff,
            0x14,
            0xe8,
            0x8c,
            0xff,
            0xff,
        ]);

        let var_int: VarInt<u64> = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::try_from(151288809941952652).unwrap())
    }

    #[test]
    fn read_var_int_into_larger_integer_reads_correctly() {
        let mut reader = Cursor::new(vec![0b00101101, 0xff, 0xff]);

        let var_int: VarInt<u32> = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(0b00101101))
    }

    #[test]
    fn read_var_int_into_integer_too_small_returns_err() {
        let mut reader = Cursor::new(vec![0b01101101, 0b11011001, 0xff, 0xff]);

        let error = VarInt::<u8>::read(&mut reader).unwrap_err();

        assert_matches!(
            *error.kind(),
            ErrorKind::VarIntValueIsTooLargeToFitInIntegerOfSize(0b0010110111011001, 1usize)
        );
    }

    #[test]
    fn write_var_int_of_1_byte_writes_correctly() {
        let var_int = VarInt::from(37u8);

        let bytes = var_int.bytes();

        assert_eq!(&bytes[..], &[0x25]);
    }

    #[test]
    fn write_var_int_of_2_bytes_writes_correctly() {
        let var_int = VarInt::from(15293u16);

        let bytes = var_int.bytes();

        assert_eq!(&bytes[..], &[0x7b, 0xbd]);
    }

    #[test]
    fn write_var_int_of_4_bytes_writes_correctly() {
        let var_int = VarInt::from(494878333u32);

        let bytes = var_int.bytes();

        assert_eq!(&bytes[..], &[0x9d, 0x7f, 0x3e, 0x7d]);
    }

    #[test]
    fn write_var_int_of_8_bytes_writes_correctly() {
        let var_int = VarInt::try_from(151288809941952652u64).unwrap();

        let bytes = var_int.bytes();

        assert_eq!(
            &bytes[..],
            &[0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c]
        );
    }

    #[test]
    fn try_from_returns_error_for_int_too_large() {
        let error = VarInt::try_from(u64::max_value()).unwrap_err();

        assert_matches!(
            *error.kind(),
            ErrorKind::IntegerValueIsTooLargeToBeStoredAsAVarInt(::std::u64::MAX)
        );
    }
}
