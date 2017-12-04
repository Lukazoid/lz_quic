use errors::*;
use std::io::{Read, Write};
use handshake::Tag;
use protocol::{Readable, Writable};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum KeyExchangeAlgorithm {
    Curve25519,
    P256,
    Unsupported(Tag),
}

impl KeyExchangeAlgorithm {
    pub fn is_supported(self) -> bool {
        !matches!(self, KeyExchangeAlgorithm::Unsupported(_))
    }
}

impl Writable for KeyExchangeAlgorithm {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing key exchange algorithm {:?}", self);
        let tag: Tag = (*self).into();

        tag.write(writer)?;
        trace!("written key exchange algorithm {:?}", self);
        Ok(())
    }
}

impl Readable for KeyExchangeAlgorithm {
    fn read<R: Read>(reader: &mut R) -> Result<KeyExchangeAlgorithm> {
        trace!("reading key exchange algorithm");
        let tag = Tag::read(reader)?;

        let key_exchange_algorithm = KeyExchangeAlgorithm::from(tag);
        debug!("read key exchange algorithm {:?}", key_exchange_algorithm);

        Ok(key_exchange_algorithm)
    }
}

impl From<KeyExchangeAlgorithm> for Tag {
    fn from(value: KeyExchangeAlgorithm) -> Self {
        match value {
            KeyExchangeAlgorithm::Curve25519 => Tag::Curve25519,
            KeyExchangeAlgorithm::P256 => Tag::P256,
            KeyExchangeAlgorithm::Unsupported(tag) => tag,
        }
    }
}

impl From<Tag> for KeyExchangeAlgorithm {
    fn from(value: Tag) -> Self {
        match value {
            Tag::Curve25519 => KeyExchangeAlgorithm::Curve25519,
            Tag::P256 => KeyExchangeAlgorithm::P256,
            tag @ _ => KeyExchangeAlgorithm::Unsupported(tag),
        }
    }
}
