use errors::*;
use smallvec::SmallVec;
use std::io::{Read, Write};
use protocol::{Readable, Writable};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SourceAddressToken(SmallVec<[u8; 32]>);

impl<'a> From<&'a [u8]> for SourceAddressToken {
    fn from(value:&'a [u8]) -> Self {
        SourceAddressToken(value.into())
    }
}

impl Readable for SourceAddressToken {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading source address token");

        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes)
            .chain_err(|| ErrorKind::FailedToReadSourceAddressToken)?;

        let source_address_token = bytes.as_slice().into();

        debug!("read source address token {:?}", source_address_token);

        Ok(source_address_token)
    }
}

impl Writable for SourceAddressToken {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing source address token {:?}", self);

        writer.write_all(self.0.as_ref())
            .chain_err(|| ErrorKind::FailedToWriteSourceAddressToken)?;

        debug!("written source address token {:?}", self);
        
        Ok(())
    }
}