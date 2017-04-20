use errors::*;
use std::convert::TryFrom;
use std::io::{Read, Write};
use quic_tag::QuicTag;
use readable::Readable;
use writable::Writable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum KeyExchangeAlgorithm {
    Curve25519,
    P256,
}

impl Writable for KeyExchangeAlgorithm {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let quic_tag: QuicTag = self.into();

        quic_tag.write(writer)
    }
}

impl Readable for KeyExchangeAlgorithm {
    fn read<R: Read>(reader: &mut R) -> Result<KeyExchangeAlgorithm> {
        let quic_tag = QuicTag::read(reader)?;

        KeyExchangeAlgorithm::try_from(quic_tag)
    }
}

impl<'a> From<&'a KeyExchangeAlgorithm> for QuicTag {
    fn from(value: &'a KeyExchangeAlgorithm) -> Self {
        match *value {
            KeyExchangeAlgorithm::Curve25519 => QuicTag::Curve25519,
            KeyExchangeAlgorithm::P256 => QuicTag::P256,
        }
    }
}


impl From<KeyExchangeAlgorithm> for QuicTag {
    fn from(value: KeyExchangeAlgorithm) -> Self {
        (&value).into()
    }
}
impl<'a> TryFrom<&'a QuicTag> for KeyExchangeAlgorithm {
    type Error = Error;

    fn try_from(value: &'a QuicTag) -> Result<Self> {
        Ok(match *value {
            QuicTag::Curve25519 => KeyExchangeAlgorithm::Curve25519,
            QuicTag::P256 => KeyExchangeAlgorithm::P256,
            quic_tag @ _ => bail!(ErrorKind::InvalidKeyExchangeAlgorithm(quic_tag)),
        })
    }
}

impl TryFrom<QuicTag> for KeyExchangeAlgorithm {
    type Error = Error;

    fn try_from(value: QuicTag) -> Result<Self> {
        KeyExchangeAlgorithm::try_from(&value)
    }
}