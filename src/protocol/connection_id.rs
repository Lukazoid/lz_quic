use errors::*;
use protocol::{Readable, Writable};
use rand::Rng;
use smallvec::SmallVec;
use std::io::{Read, Write};

/// A unique identifier for a connection.
/// This will always be required when communicating with the server to identify the client however it is not required when the server communicates with the client as the client can identify the server purely based upon the port opened.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ConnectionId([u8; 18]);

impl Readable for ConnectionId {
    fn read<R: Read>(reader: &mut R) -> Result<ConnectionId> {
        trace!("reading connection id");

        // read a maximum of 18 bytes
        let bytes: SmallVec<[u8; 20]> = Readable::read(&mut reader.take(18))?;

        let mut inner = [0; 18];
        let start_offset = inner.len() - bytes.len();
        inner[start_offset..].copy_from_slice(&bytes[..]);

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
        let mut bytes = [0; 18];
        rng.fill_bytes(&mut bytes);
        let connection_id = ConnectionId(bytes);
        debug!("generated new connection id {:?}", connection_id);

        connection_id
    }
}
