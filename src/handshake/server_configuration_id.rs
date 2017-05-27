use errors::*;
use std::io::{Read, Write};
use protocol::{Readable, Writable};
use num::bigint::{BigInt, Sign};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerConfigurationId([u8; 16]);

impl ServerConfigurationId {
    fn to_big_int(&self) -> BigInt {
        BigInt::from_bytes_le(Sign::Plus, &self.0)
    }
}

impl Display for ServerConfigurationId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.to_big_int().fmt(f)
    }
}

impl Readable for ServerConfigurationId {
    fn read<R: Read>(reader: &mut R) -> Result<ServerConfigurationId> {
        let mut buf = [0u8; 16];
        let server_configuration_id = reader.read_exact(&mut buf)
            .chain_err(|| ErrorKind::UnableToReadBytes)
            .map(|_| ServerConfigurationId(buf))?;

        Ok(server_configuration_id)
    }
}

impl Writable for ServerConfigurationId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        Ok(writer.write_all(&self.0)
            .chain_err(|| ErrorKind::UnableToWriteBytes(self.0.len()))?)
    }
}
