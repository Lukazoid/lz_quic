use errors::*;
use std::io::{Read, Write};
use rand::Rng;
use writable::Writable;
use readable::Readable;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A unique identifier for a connection.
/// This will always be required when communicating with the server to identify the client however it is not required when the server communicates with the client as the client can identify the server purely based upon the port opened.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ConnectionId(u64);

impl Display for ConnectionId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Readable for ConnectionId {
    fn read<R: Read>(reader: &mut R) -> Result<ConnectionId> {
        let inner = u64::read(reader)?;

        Ok(ConnectionId(inner))
    }
}

impl Writable for ConnectionId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.write(writer)?;

        Ok(())
    }
}

impl ConnectionId {
    pub fn generate<R: Rng>(rng: &mut R) -> ConnectionId {
        let inner = rng.next_u64();

        ConnectionId(inner)
    }
}