use errors::*;
use std::io::{Read, Write};
use protocol::{Readable, Writable};
use num::bigint::{BigInt, Sign};
use std::fmt::{Display, Formatter, Result as FmtResult};
use rand::Rng;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerConfigurationId([u8; 16]);

impl ServerConfigurationId {
    pub fn generate<R: Rng>(rng: &mut R) -> ServerConfigurationId {
        trace!("generating new server configuration id");

        let mut bytes = [0u8; 16];

        rng.fill_bytes(&mut bytes);

        let server_configuration_id = ServerConfigurationId(bytes);

        debug!("generated new server configuration id {:?}", server_configuration_id);

        server_configuration_id
    }

    fn to_big_int(&self) -> BigInt {
        BigInt::from_bytes_le(Sign::Plus, &self.0)
    }
}

impl From<[u8; 16]> for ServerConfigurationId {
    fn from(value: [u8; 16]) -> Self{
        ServerConfigurationId(value)
    }
}

impl Display for ServerConfigurationId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.to_big_int().fmt(f)
    }
}

impl Readable for ServerConfigurationId {
    fn read<R: Read>(reader: &mut R) -> Result<ServerConfigurationId> {
        trace!("reading server configuration id");

        let mut buf = [0u8; 16];
        let server_configuration_id = reader.read_exact(&mut buf)
            .chain_err(|| ErrorKind::FailedToReadBytes)
            .map(|_| ServerConfigurationId(buf))?;

        debug!("read server configuration id {:?}", server_configuration_id);

        Ok(server_configuration_id)
    }
}

impl Writable for ServerConfigurationId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing server configuration id {:?}", self);

        writer.write_all(&self.0)
            .chain_err(|| ErrorKind::FailedToWriteBytes(self.0.len()))?;

        debug!("written server configuration id {:?}", self);

        Ok(())
    }
}
