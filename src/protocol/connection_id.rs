use bytes::Bytes;
use errors::*;
use protocol::{Readable, Writable};
use rand::{OsRng, Rng};
use std::io::{Read, Write};

/// A unique identifier for a connection.
/// This will always be required when communicating with the server to identify the client however it is not required when the server communicates with the client as the client can identify the server purely based upon the port opened.
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ConnectionId([u8; 18]);

impl Readable for ConnectionId {
    type Context = ();
    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<ConnectionId> {
        trace!("reading connection id");

        // read a maximum of 18 bytes
        let bytes: Bytes = Readable::read(&mut reader.take(18))?;

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
    pub fn generate_with_rng<R: Rng>(rng: &mut R) -> ConnectionId {
        trace!("generating new connection id");
        let mut bytes = [0; 18];
        rng.fill_bytes(&mut bytes);
        let connection_id = ConnectionId(bytes);
        debug!("generated new connection id {:?}", connection_id);

        connection_id
    }

    pub fn generate() -> Result<ConnectionId> {
        let mut rng =
            OsRng::new().chain_err(|| ErrorKind::FailedToCreateCryptographicRandomNumberGenerator)?;

        Ok(ConnectionId::generate_with_rng(&mut rng))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionId;
    use protocol::{self, Readable};

    #[test]
    fn read_write_connection_id() {
        let connection_id = ConnectionId::generate().unwrap();

        protocol::test_write_read(&connection_id).unwrap();
    }

    #[test]
    fn read_of_short_connection_id() {
        let bytes = [21, 54, 213, 17];

        let connection_id = ConnectionId::from_bytes(&bytes).unwrap();

        assert_eq!(
            connection_id,
            ConnectionId([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 21, 54, 213, 17])
        );
    }
}
