use errors::*;
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DiversificationNonce([u8; 32]);

impl DiversificationNonce {
    pub fn bytes(&self) -> &[u8]{
        &self.0
    }
}

impl From<[u8; 32]> for DiversificationNonce {
    fn from(value: [u8; 32]) -> Self {
        DiversificationNonce(value)
    }
}

impl Writable for DiversificationNonce {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_all(&self.0)
            .chain_err(|| ErrorKind::UnableToWriteDiversificationNonce)
    }
}

impl Readable for DiversificationNonce {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut value = [0u8; 32];
        reader
            .read_exact(&mut value)
            .chain_err(|| ErrorKind::UnableToReadDiversificationNonce)?;

        Ok(DiversificationNonce(value))
    }
}

