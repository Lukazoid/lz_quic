use byteorder::WriteBytesExt;
use super::primitives::u24::U24;
use super::primitives::u48::U48;
use super::primitives::uf16::UF16;
use super::byte_order_primitives::ByteOrderPrimitives;
use std::io::Result;

pub trait WriteQuicPrimitives: WriteBytesExt {
    fn write_u24<T: ByteOrderPrimitives>(&mut self, n: U24) -> Result<()> {
        let mut buf = [0; 3];
        T::write_u24(&mut buf, n);
        self.write_all(&buf)
    }

    fn write_u48<T: ByteOrderPrimitives>(&mut self, n: U48) -> Result<()> {
        let mut buf = [0; 6];
        T::write_u48(&mut buf, n);
        self.write_all(&buf)
    }

    fn write_uf16<T: ByteOrderPrimitives>(&mut self, n: UF16) -> Result<()> {
        let mut buf = [0; 2];
        T::write_uf16(&mut buf, n);
        self.write_all(&buf)
    }
}

impl<W: WriteBytesExt> WriteQuicPrimitives for W {}