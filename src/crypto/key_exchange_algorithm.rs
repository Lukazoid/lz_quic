use errors::*;
use std::convert::TryFrom;
use std::io::{Read, Write};
use tag::Tag;
use readable::Readable;
use writable::Writable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum KeyExchangeAlgorithm {
    Curve25519,
    P256,
}

impl Writable for KeyExchangeAlgorithm {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let tag: Tag = self.into();

        tag.write(writer)
    }
}

impl Readable for KeyExchangeAlgorithm {
    fn read<R: Read>(reader: &mut R) -> Result<KeyExchangeAlgorithm> {
        let tag = Tag::read(reader)?;

        KeyExchangeAlgorithm::try_from(tag)
    }
}

impl<'a> From<&'a KeyExchangeAlgorithm> for Tag {
    fn from(value: &'a KeyExchangeAlgorithm) -> Self {
        match *value {
            KeyExchangeAlgorithm::Curve25519 => Tag::Curve25519,
            KeyExchangeAlgorithm::P256 => Tag::P256,
        }
    }
}


impl From<KeyExchangeAlgorithm> for Tag {
    fn from(value: KeyExchangeAlgorithm) -> Self {
        (&value).into()
    }
}
impl<'a> TryFrom<&'a Tag> for KeyExchangeAlgorithm {
    type Error = Error;

    fn try_from(value: &'a Tag) -> Result<Self> {
        Ok(match *value {
            Tag::Curve25519 => KeyExchangeAlgorithm::Curve25519,
            Tag::P256 => KeyExchangeAlgorithm::P256,
            tag @ _ => bail!(ErrorKind::InvalidKeyExchangeAlgorithm(tag)),
        })
    }
}

impl TryFrom<Tag> for KeyExchangeAlgorithm {
    type Error = Error;

    fn try_from(value: Tag) -> Result<Self> {
        KeyExchangeAlgorithm::try_from(&value)
    }
}