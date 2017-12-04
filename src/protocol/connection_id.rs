use errors::*;
use std::io::{Read, Write};
use rand::Rng;
use protocol::{Readable, Writable};
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
        trace!("reading connection id");
        let inner = u64::read(reader)?;
        let connection_id = ConnectionId(inner);
        debug!("read connection id {:?}", connection_id);

        Ok(connection_id)
    }
}

impl Writable for ConnectionId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing connection id {:?}", self);
        self.0.write(writer)?;
        debug!("written connection id {:?}", self);

        Ok(())
    }
}

impl ConnectionId {
    pub fn generate<R: Rng>(rng: &mut R) -> ConnectionId {
        trace!("generating new connection id");
        let inner = rng.next_u64();
        let connection_id = ConnectionId(inner);
        debug!("generated new connection id {:?}", connection_id);
        
        connection_id
    }
}