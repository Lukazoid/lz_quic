use conv::ValueInto;
use errors::*;
use protocol::{ErrorCode, Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConnectionCloseFrame {
    error_code: ErrorCode,
    reason_phrase: String,
}

impl Readable for ConnectionCloseFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading connection close frame");

        let error_code =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadConnectionCloseFrame)?;

        let reason_phrase_len =
            VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadConnectionCloseFrame)?;

        let reason_phrase = Readable::read(&mut reader.take(reason_phrase_len.into()))
            .chain_err(|| ErrorKind::FailedToReadConnectionCloseFrame)?;

        let connection_close_frame = Self {
            error_code,
            reason_phrase,
        };

        debug!("read connection close frame {:?}", connection_close_frame);

        Ok(connection_close_frame)
    }
}

impl Writable for ConnectionCloseFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing connection close frame {:?}", self);

        self.error_code
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteConnectionCloseFrame)?;

        let reason_phrase_len: VarInt = self.reason_phrase
            .len()
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteConnectionCloseFrame)?;
        reason_phrase_len
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteConnectionCloseFrame)?;

        self.reason_phrase
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteConnectionCloseFrame)?;

        debug!("written connection close frame {:?}", self);

        Ok(())
    }
}
