use errors::*;
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use rand::Rng;
use writable::Writable;
use readable::Readable;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A unique identifier for a connection.
/// This will always be required when communicating with the server to identify the client however it is not required when the server communicates with the client as the client can identify the server purely based upon the port opened.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct QuicConnectionId(u64);

impl Display for QuicConnectionId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Readable for QuicConnectionId {
    fn read<R: Read>(reader: &mut R) -> Result<QuicConnectionId> {
        let inner = reader.read_u64::<LittleEndian>()?;

        Ok(QuicConnectionId(inner))
    }
}

impl Writable for QuicConnectionId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_u64::<LittleEndian>(self.0)?;

        Ok(())
    }
}

impl QuicConnectionId {
    pub fn generate<R: Rng>(rng: &mut R) -> QuicConnectionId {
        let inner = rng.next_u64();

        QuicConnectionId(inner)
    }
}