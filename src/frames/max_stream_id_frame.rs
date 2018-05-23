use errors::*;
use protocol::{Readable, StreamId, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaxStreamIdFrame {
    pub maximum_stream_id: StreamId,
}

impl Readable for MaxStreamIdFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading max stream id frame");
        let maximum_stream_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadMaxStreamIdFrame)?;

        let max_stream_data_frame = Self { maximum_stream_id };

        debug!("read max stream id frame {:?}", max_stream_data_frame);

        Ok(max_stream_data_frame)
    }
}

impl Writable for MaxStreamIdFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing max stream id frame {:?}", self);

        self.maximum_stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteMaxStreamDataFrame)?;

        debug!("written max stream id frame {:?}", self);
        Ok(())
    }
}
