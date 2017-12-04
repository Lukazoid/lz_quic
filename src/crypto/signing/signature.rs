use errors::*;
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Signature(Vec<u8>);

impl Signature {
    pub fn bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl From<Vec<u8>> for Signature {
    fn from(value: Vec<u8>) -> Self {
        Signature(value)
    }
}


impl<'a> From<&'a [u8]> for Signature {
    fn from(value: &'a [u8]) -> Self {
        Signature(value.into())
    }
}

impl Readable for Signature {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading signature");
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)
            .chain_err(|| ErrorKind::FailedToReadSignatureBytes)?;
        
        let signature = bytes.into();
        debug!("read signature {:?}", signature);
        Ok(signature)
    }
}

impl Writable for Signature {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing signature {:?}", self);
        writer.write_all(self.bytes())
            .chain_err(||ErrorKind::FailedToWriteSignatureBytes)?;
        debug!("written signature {:?}", self);
        Ok(())
    }
}