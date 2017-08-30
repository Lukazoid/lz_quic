use errors::*;
use protocol::{Writable, Readable};
use std::io::{Read, Write};
use time::{self, Timespec};
use byteorder::{BigEndian, ByteOrder};
use rand::{Rng, OsRng};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ClientNonce([u8;32]);

impl ClientNonce {
    pub fn generate(server_orbit: u64) -> Result<Self> {
        let mut nonce = [0u8;32];

        let seconds_since_epoch = time::now_utc().to_timespec().sec as u32;

        // 4 bytes of timestamp (big-endian, UNIX epoch seconds)
        BigEndian::write_u32(&mut nonce[0..4], seconds_since_epoch);

        // 8 bytes of server orbit
        server_orbit.write_to_slice(&mut nonce[4..12]);
        
        let mut rng = OsRng::new()
            .chain_err(|| {
                ErrorKind::FailedToCreateCryptographicRandomNumberGenerator
            })?;

        // 20 bytes of random data
        rng.fill_bytes(&mut nonce[12..32]);

        Ok(ClientNonce(nonce))
    }
}

impl Writable for ClientNonce {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_all(&self.0)
            .chain_err(|| ErrorKind::FailedToWriteClientNonce)
    }
}
impl Readable for ClientNonce {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut value = [0u8; 32];
        reader
            .read_exact(&mut value)
            .chain_err(|| ErrorKind::FailedToReadClientNonce)?;

        Ok(ClientNonce(value))
    }
}