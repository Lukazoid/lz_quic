use errors::*;
use conv::TryFrom;
use std::io::{Read, Write};
use tag::Tag;
use readable::Readable;
use writable::Writable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Proof {
    X509,
}

impl Writable for Proof {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let tag: Tag = (*self).into();

        tag.write(writer)
    }
}

impl Readable for Proof {
    fn read<R: Read>(reader: &mut R) -> Result<Proof> {
        let tag = Tag::read(reader)?;

        Proof::try_from(tag)
    }
}

impl From<Proof> for Tag {
    fn from(value: Proof) -> Self {
        match value {
            Proof::X509 => Tag::X509,
        }
    }
}

impl TryFrom<Tag> for Proof {
    type Err = Error;
    
    fn try_from(value: Tag) -> Result<Self> {
        Ok(match value {
            Tag::X509 => Proof::X509,
            tag @ _ => bail!(ErrorKind::InvalidProofType(tag)),
        })
    }
}