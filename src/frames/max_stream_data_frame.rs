use conv::ValueInto;
use errors::*;
use protocol::{Readable, StreamId, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaxStreamDataFrame {
    pub stream_id: StreamId,
    pub maximum_stream_data: u64,
}

impl Readable for MaxStreamDataFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading max stream data frame");
        let stream_id =
            Readable::read(reader).chain_err(|| ErrorKind::FailedToReadMaxStreamDataFrame)?;

        let maximum_stream_data =
            VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadMaxStreamDataFrame)?;

        let max_stream_data_frame = Self {
            stream_id,
            maximum_stream_data: maximum_stream_data.into(),
        };

        debug!("read max stream data frame {:?}", max_stream_data_frame);

        Ok(max_stream_data_frame)
    }
}

impl Writable for MaxStreamDataFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing max stream data frame {:?}", self);

        self.stream_id
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteMaxStreamDataFrame)?;

        let maximum_stream_data: VarInt = self.maximum_stream_data
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteMaxStreamDataFrame)?;
        maximum_stream_data
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteMaxStreamDataFrame)?;

        debug!("written max stream data frame {:?}", self);
        Ok(())
    }
}
