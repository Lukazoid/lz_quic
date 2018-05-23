use errors::*;
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathChallengeFrame {
    pub data: u8,
}

impl Readable for PathChallengeFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading path challenge frame");

        let data = Readable::read(reader).chain_err(|| ErrorKind::FailedToReadPathChallengeFrame)?;

        let path_challenge_frame = Self { data };

        debug!("read path challenge frame {:?}", path_challenge_frame);

        Ok(path_challenge_frame)
    }
}

impl Writable for PathChallengeFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing path challenge frame {:?}", self);

        self.data
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWritePathChallengeFrame)?;

        debug!("written path challenge frame {:?}", self);

        Ok(())
    }
}
