use errors::*;
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt};
use writable::Writable;
use readable::Readable;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u32);

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Writable for Version {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.write(writer)?;

        Ok(())
    }
}

impl Readable for Version {
    fn read<R: Read>(reader: &mut R) -> Result<Version> {
        let version = u32::read(reader)
            .map(Version)?;

        Ok(version)
    }
}

impl Version {
    pub const DRAFT_IETF_01: Version = Version(0x100000FF);
}