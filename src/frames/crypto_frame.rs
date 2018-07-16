use bytes::Bytes;
use conv::ValueInto;
use errors::*;
use frames::StreamOffset;
use protocol::{Readable, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CryptoFrame {
    pub offset: StreamOffset,
    pub data: Bytes,
}

impl Readable for CryptoFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _context: &Self::Context) -> Result<Self> {
        trace!("reading crypto frame");

        let offset = Readable::read(reader).chain_err(|| ErrorKind::FailedToReadCryptoFrame)?;

        let length = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadCryptoFrame)?;
        let data = Readable::read(&mut reader.take(length.into_inner()))
            .chain_err(|| ErrorKind::FailedToReadCryptoFrame)?;

        let crypto_frame = Self { offset, data };
        debug!("read crypto frame {:?}", crypto_frame);

        Ok(crypto_frame)
    }
}

impl Writable for CryptoFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing crypto frame {:?}", self);

        self.offset
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteCryptoFrame)?;

        let length: VarInt = self.data
            .len()
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteCryptoFrame)?;
        length
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteCryptoFrame)?;

        self.data
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteCryptoFrame)?;

        debug!("written crypto frame {:?}", self);

        Ok(())
    }
}
