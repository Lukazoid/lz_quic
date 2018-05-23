use conv::ValueInto;
use errors::*;
use protocol::{Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ApplicationCloseFrame {
    application_error_code: u16,
    reason_phrase: String,
}

impl Readable for ApplicationCloseFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading application close frame");

        let application_error_code =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadApplicationCloseFrame)?;

        let reason_phrase_len =
            VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadApplicationCloseFrame)?;

        let reason_phrase = Readable::read(&mut reader.take(reason_phrase_len.into()))
            .chain_err(|| ErrorKind::FailedToReadApplicationCloseFrame)?;

        let application_close_frame = Self {
            application_error_code,
            reason_phrase,
        };

        debug!("read application close frame {:?}", application_close_frame);

        Ok(application_close_frame)
    }
}

impl Writable for ApplicationCloseFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing application close frame {:?}", self);

        self.application_error_code
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteApplicationCloseFrame)?;

        let reason_phrase_len: VarInt = self.reason_phrase
            .len()
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteApplicationCloseFrame)?;
        reason_phrase_len
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteApplicationCloseFrame)?;

        self.reason_phrase
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteApplicationCloseFrame)?;

        debug!("written application close frame {:?}", self);

        Ok(())
    }
}
