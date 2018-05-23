use errors::*;
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathResponseFrame {
    pub data: u8,
}

impl Readable for PathResponseFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading path response frame");

        let data = Readable::read(reader).chain_err(|| ErrorKind::FailedToReadPathResponseFrame)?;

        let path_response_frame = Self { data };

        debug!("read path response frame {:?}", path_response_frame);

        Ok(path_response_frame)
    }
}

impl Writable for PathResponseFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing path response frame {:?}", self);

        self.data
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWritePathResponseFrame)?;

        debug!("written path response frame {:?}", self);

        Ok(())
    }
}
