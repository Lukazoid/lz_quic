use errors::*;
use protocol::{Writable, Readable};
use std::io::{Read, Write};
use rand::{Rng, OsRng};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerNonce([u8;32]);

impl ServerNonce {
    pub fn generate() -> Result<Self> {
        let mut nonce = [0u8;32];
        
        let mut rng = OsRng::new()
            .chain_err(|| {
                ErrorKind::FailedToCreateCryptographicRandomNumberGenerator
            })?;

        // 20 bytes of random data
        rng.fill_bytes(&mut nonce);

        Ok(ServerNonce(nonce))
    }
    
    pub fn bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Writable for ServerNonce {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_all(&self.0)
            .chain_err(|| ErrorKind::FailedToWriteServerNonce)
    }
}
impl Readable for ServerNonce {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut value = [0u8; 32];
        reader
            .read_exact(&mut value)
            .chain_err(|| ErrorKind::FailedToReadServerNonce)?;

        Ok(ServerNonce(value))
    }
}