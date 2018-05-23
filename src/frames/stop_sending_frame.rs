use errors::*;
use protocol::{Readable, StreamId, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StopSendingFrame {
    pub stream_id: StreamId,
    pub application_error_code: u16,
}

impl Readable for StopSendingFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading stop sending frame");

        let stream_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStopSendingFrame)?;
        let application_error_code =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadStopSendingFrame)?;

        let stop_sending_frame = Self {
            stream_id,
            application_error_code,
        };

        debug!("read stop sending frame {:?}", stop_sending_frame);

        Ok(stop_sending_frame)
    }
}

impl Writable for StopSendingFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stop sending frame {:?}", self);

        self.stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStopSendingFrame)?;
        self.application_error_code
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteStopSendingFrame)?;

        debug!("written stop sending frame {:?}", self);

        Ok(())
    }
}
