use errors::*;
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt};
use writable::Writable;
use readable::Readable;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct QuicVersion(u32);

impl Display for QuicVersion {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Writable for QuicVersion {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.write(writer)?;

        Ok(())
    }
}

impl Readable for QuicVersion {
    fn read<R: Read>(reader: &mut R) -> Result<QuicVersion> {
        let quic_version = u32::read(reader)
            .map(QuicVersion)?;

        Ok(quic_version)
    }
}

impl QuicVersion {
    pub const DRAFT_IETF_01: QuicVersion = QuicVersion(0x100000FF);
}