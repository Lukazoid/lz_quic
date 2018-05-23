use conv::ValueInto;
use errors::*;
use protocol::{Readable, VarInt, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaxDataFrame {
    pub maximum_data: u64,
}

impl Readable for MaxDataFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading max data frame");

        let maximum_data = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadMaxDataFrame)?;

        let max_data_frame = Self {
            maximum_data: maximum_data.into(),
        };

        debug!("read max data frame {:?}", max_data_frame);

        Ok(max_data_frame)
    }
}

impl Writable for MaxDataFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing max data frame {:?}", self);

        let maximum_data: VarInt = self.maximum_data
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteMaxDataFrame)?;
        maximum_data
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteMaxDataFrame)?;

        debug!("written max data frame {:?}", self);

        Ok(())
    }
}
