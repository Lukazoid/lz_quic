use byteorder::ReadBytesExt;
use primitives::{U24, U48, UF16, ByteOrderPrimitives};
use std::io::Result;

pub trait ReadQuicPrimitives: ReadBytesExt {
    fn read_u24<T: ByteOrderPrimitives>(&mut self) -> Result<U24> {
        let mut buf = [0; 3];
        self.read_exact(&mut buf)?;

        Ok(<T as ByteOrderPrimitives>::read_u24(&buf))
    }

    fn read_u48<T: ByteOrderPrimitives>(&mut self) -> Result<U48> {
        let mut buf = [0; 6];
        self.read_exact(&mut buf)?;

        Ok(T::read_u48(&buf))
    }

    fn read_uf16<T: ByteOrderPrimitives>(&mut self) -> Result<UF16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;

        Ok(T::read_uf16(&buf))
    }
}

impl<R: ReadBytesExt> ReadQuicPrimitives for R {}