use errors::*;
use protocol::{Readable, Writable};
use std::io::{Read, Write};
use rand::{OsRng, Rng};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerNonce([u8; 32]);

impl ServerNonce {
    pub fn generate() -> Result<Self> {
        trace!("generating a new server nonce");

        let mut nonce = [0u8; 32];

        let mut rng = OsRng::new().chain_err(|| {
            ErrorKind::FailedToCreateCryptographicRandomNumberGenerator
        })?;

        // 20 bytes of random data
        rng.fill_bytes(&mut nonce);

        let server_nonce = ServerNonce(nonce);

        debug!("generated a new server nonce {:?}", server_nonce);

        Ok(server_nonce)
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0[..]
    }
}

impl Writable for ServerNonce {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing server nonce {:?}", self);

        writer
            .write_all(&self.0)
            .chain_err(|| ErrorKind::FailedToWriteServerNonce)?;

        debug!("written server nonce {:?}", self);

        Ok(())
    }
}
impl Readable for ServerNonce {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading server nonce");

        let mut value = [0u8; 32];
        reader
            .read_exact(&mut value)
            .chain_err(|| ErrorKind::FailedToReadServerNonce)?;

        let server_nonce = ServerNonce(value);

        debug!("read server nonce {:?}", server_nonce);

        Ok(server_nonce)
    }
}
