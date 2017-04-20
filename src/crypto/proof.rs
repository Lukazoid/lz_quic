use errors::*;
use std::convert::TryFrom;
use std::io::{Read, Write};
use quic_tag::QuicTag;
use readable::Readable;
use writable::Writable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Proof {
    X509,
}

impl Writable for Proof {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let quic_tag: QuicTag = self.into();

        quic_tag.write(writer)
    }
}

impl Readable for Proof {
    fn read<R: Read>(reader: &mut R) -> Result<Proof> {
        let quic_tag = QuicTag::read(reader)?;

        Proof::try_from(quic_tag)
    }
}

impl<'a> From<&'a Proof> for QuicTag {
    fn from(value: &'a Proof) -> Self {
        match *value {
            Proof::X509 => QuicTag::X509,
        }
    }
}

impl From<Proof> for QuicTag {
    fn from(value: Proof) -> Self {
        (&value).into()
    }
}

impl<'a> TryFrom<&'a QuicTag> for Proof {
    type Error = Error;

    fn try_from(value: &'a QuicTag) -> Result<Self> {
        Ok(match *value {
            QuicTag::X509 => Proof::X509,
            quic_tag @ _ => bail!(ErrorKind::InvalidProofType(quic_tag)),
        })
    }
}

impl TryFrom<QuicTag> for Proof {
    fn try_from(value: QuicTag) -> Result<Self> {
    type Error = Error;
        Proof::try_from(&value)
    }
}