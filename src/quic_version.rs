use errors::*;
use std::io::{Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
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
        Ok(writer.write_u32::<LittleEndian>(self.0)?)
    }
}

impl Readable for QuicVersion {
    fn read<R: Read>(reader: &mut R) -> Result<QuicVersion> {
        let quic_version = reader.read_u32::<LittleEndian>()
            .map(QuicVersion)?;

        Ok(quic_version)
    }
}

impl QuicVersion {
    pub const DRAFT_IETF_01: QuicVersion = QuicVersion(0x100000FF);
}