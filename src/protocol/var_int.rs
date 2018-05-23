use byteorder::{NetworkEndian, ReadBytesExt};
use conv::{ValueFrom, ValueInto};
use errors::*;
use num::{FromPrimitive, Unsigned};
use protocol::{Readable, Writable};
use std::error::Error as StdError;
use std::io::{Read, Write};
use std::mem;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct VarInt(u64);

impl VarInt {
    pub fn into_inner(self) -> u64 {
        self.0
    }
}

impl From<u8> for VarInt {
    fn from(value: u8) -> Self {
        VarInt(value as u64)
    }
}

impl From<u16> for VarInt {
    fn from(value: u16) -> Self {
        VarInt(value as u64)
    }
}
impl From<u32> for VarInt {
    fn from(value: u32) -> Self {
        VarInt(value as u64)
    }
}

impl<T: ValueInto<u64>> ValueFrom<T> for VarInt
where
    <T as ValueInto<u64>>::Err: StdError + Send + 'static,
{
    type Err = Error;

    fn value_from(value: T) -> Result<VarInt> {
        let int_value = value
            .value_into()
            .chain_err(|| ErrorKind::ValueIsTooLargeToBeStoredAsAVarInt)?;

        if int_value > 4611686018427387903 {
            bail!(ErrorKind::IntegerValueIsTooLargeToBeStoredAsAVarInt(
                int_value
            ));
        }

        Ok(VarInt(int_value))
    }
}

impl From<VarInt> for u64 {
    fn from(value: VarInt) -> Self {
        value.into_inner()
    }
}

impl Readable for VarInt {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
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

        trace!(
            "reading variable length integer of length {:?}",
            total_length
        );

        let remaining_bytes_count = total_length - 1;

        let all_bytes = if remaining_bytes_count > 0 {
            let next_bytes = reader
                .read_uint::<NetworkEndian>(remaining_bytes_count)
                .chain_err(|| ErrorKind::FailedToReadBytes)?;

            (first_byte_value << (remaining_bytes_count * 8)) | next_bytes
        } else {
            first_byte_value
        };

        let var_int = VarInt(all_bytes);

        debug!("read variable length integer {:?}", var_int);

        Ok(var_int)
    }
}

impl Writable for VarInt {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing variable length integer {:?}", self);

        let int_value: u64 = self.into_inner();

        match int_value {
            0...63 => (int_value as u8).write(writer)?,
            64...16383 => (0x4000 | int_value as u16).write(writer)?,
            16384...1073741823 => (0x80000000 | int_value as u32).write(writer)?,
            1073741824...4611686018427387903 => (0xC000000000000000 | int_value).write(writer)?,
            _ => bail!(ErrorKind::IntegerValueIsTooLargeToBeStoredAsAVarInt(
                int_value
            )),
        };

        debug!("written variable length integer {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::VarInt;
    use conv::ValueFrom;
    use errors::*;
    use protocol::{Readable, Writable};
    use std::io::Cursor;

    #[test]
    fn read_var_int_of_1_byte_reads_correctly() {
        let mut reader = Cursor::new(vec![0x25, 0xff, 0xff]);

        let var_int = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(37u8))
    }

    #[test]
    fn read_var_int_of_2_bytes_reads_correctly() {
        let mut reader = Cursor::new(vec![0x7b, 0xbd, 0xff, 0xff]);

        let var_int = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(15293u16))
    }

    #[test]
    fn read_var_int_of_4_bytes_reads_correctly() {
        let mut reader = Cursor::new(vec![0x9d, 0x7f, 0x3e, 0x7d, 0xff, 0xff]);

        let var_int = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(494878333u32))
    }

    #[test]
    fn read_var_int_of_8_bytes_reads_correctly() {
        let mut reader = Cursor::new(vec![
            0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c, 0xff, 0xff
        ]);

        let var_int = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::value_from(151288809941952652u64).unwrap())
    }

    #[test]
    fn read_var_int_into_larger_integer_reads_correctly() {
        let mut reader = Cursor::new(vec![0b00101101, 0xff, 0xff]);

        let var_int = VarInt::read(&mut reader).unwrap();

        assert_eq!(var_int, VarInt::from(0b00101101u32))
    }

    #[test]
    fn write_var_int_of_1_byte_writes_correctly() {
        let var_int = VarInt::from(37u8);

        let bytes = var_int.bytes().unwrap();

        assert_eq!(&bytes[..], &[0x25]);
    }

    #[test]
    fn write_var_int_of_2_bytes_writes_correctly() {
        let var_int = VarInt::from(15293u16);

        let bytes = var_int.bytes().unwrap();

        assert_eq!(&bytes[..], &[0x7b, 0xbd]);
    }

    #[test]
    fn write_var_int_of_4_bytes_writes_correctly() {
        let var_int = VarInt::from(494878333u32);

        let bytes = var_int.bytes().unwrap();

        assert_eq!(&bytes[..], &[0x9d, 0x7f, 0x3e, 0x7d]);
    }

    #[test]
    fn write_var_int_of_8_bytes_writes_correctly() {
        let var_int = VarInt::value_from(151288809941952652u64).unwrap();

        let bytes = var_int.bytes().unwrap();

        assert_eq!(
            &bytes[..],
            &[0xc2, 0x19, 0x7c, 0x5e, 0xff, 0x14, 0xe8, 0x8c]
        );
    }

    #[test]
    fn try_from_returns_error_for_int_too_large() {
        let error = VarInt::value_from(u64::max_value()).unwrap_err();

        assert_matches!(
            *error.kind(),
            ErrorKind::IntegerValueIsTooLargeToBeStoredAsAVarInt(::std::u64::MAX)
        );
    }
}
