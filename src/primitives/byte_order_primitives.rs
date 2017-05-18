use byteorder::ByteOrder;
use primitives::{U24, U48, UF16};
use std::convert::From;
use conv::TryFrom;

pub trait ByteOrderPrimitives: ByteOrder {
    fn read_u24(buf: &[u8]) -> U24 {
        let read_bytes = Self::read_uint(buf, 3) as u32;

        U24::try_from(read_bytes).unwrap()
    }

    fn read_u48(buf: &[u8]) -> U48 {
        let read_bytes = Self::read_uint(buf, 6);

        U48::try_from(read_bytes).unwrap()
    }

    fn read_uf16(buf: &[u8]) -> UF16 {
        let read_bytes = Self::read_u16(buf);

        UF16::from_binary(read_bytes)
    }

    fn write_u24(buf: &mut [u8], n: U24) {
        let std_int = u32::from(n);
        Self::write_uint(buf, std_int as u64, 3);
    }

    fn write_u48(buf: &mut [u8], n: U48) {
        let std_int: u64 = u64::from(n);
        Self::write_uint(buf, std_int, 6);
    }

    fn write_uf16(buf: &mut [u8], n: UF16) {
        Self::write_u16(buf, n.to_binary())
    }
}

impl<B: ByteOrder> ByteOrderPrimitives for B {}
