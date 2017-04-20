use errors::*;
use std::convert::TryFrom;
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
        let tag: Tag = self.into();

        tag.write(writer)
    }
}

impl Readable for Proof {
    fn read<R: Read>(reader: &mut R) -> Result<Proof> {
        let tag = Tag::read(reader)?;

        Proof::try_from(tag)
    }
}

impl<'a> From<&'a Proof> for Tag {
    fn from(value: &'a Proof) -> Self {
        match *value {
            Proof::X509 => Tag::X509,
        }
    }
}

impl From<Proof> for Tag {
    fn from(value: Proof) -> Self {
        (&value).into()
    }
}

impl<'a> TryFrom<&'a Tag> for Proof {
    type Error = Error;

    fn try_from(value: &'a Tag) -> Result<Self> {
        Ok(match *value {
            Tag::X509 => Proof::X509,
            tag @ _ => bail!(ErrorKind::InvalidProofType(tag)),
        })
    }
}

impl TryFrom<Tag> for Proof {
    type Error = Error;
    fn try_from(value: Tag) -> Result<Self> {
        Proof::try_from(&value)
    }
}