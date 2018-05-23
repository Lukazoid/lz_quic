use conv::ValueInto;
use errors::*;
use protocol::{ConnectionId, Readable, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewConnectionIdFrame {
    pub sequence: u64,
    pub length: u8,
    pub connection_id: ConnectionId,
    pub stateless_reset_token: u128,
}

impl Readable for NewConnectionIdFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading new connection id frame");

        let sequence =
            VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;
        let length =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;
        let connection_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;
        let stateless_reset_token =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;

        let new_connection_id_frame = Self {
            sequence: sequence.into(),
            length,
            connection_id,
            stateless_reset_token,
        };

        debug!("read new connection id frame {:?}", new_connection_id_frame);

        Ok(new_connection_id_frame)
    }
}

impl Writable for NewConnectionIdFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing new connection id frame {:?}", self);

        let sequence: VarInt = self.sequence
            .value_into()
            .chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;
        sequence
            .write(writer)
            .chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;

        self.length
            .write(writer)
            .chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;
        self.connection_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;
        self.stateless_reset_token
            .write(writer)
            .chain_err(|| ErrorKind::FailedToReadNewConnectionIdFrame)?;

        debug!("written new connection id frame {:?}", self);

        Ok(())
    }
}
