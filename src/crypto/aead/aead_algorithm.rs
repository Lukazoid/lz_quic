use errors::*;
use std::io::{Read, Write};
use handshake::Tag;
use protocol::{Readable, Writable};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum AeadAlgorithm {
    AesGcm,
    Salsa20Poly1305,
    Unsupported(Tag),
}

impl Writable for AeadAlgorithm {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let tag: Tag = (*self).into();

        tag.write(writer)
    }
}

impl Readable for AeadAlgorithm {
    fn read<R: Read>(reader: &mut R) -> Result<AeadAlgorithm> {
        let tag = Tag::read(reader)?;

        Ok(AeadAlgorithm::from(tag))
    }
}

impl From<AeadAlgorithm> for Tag {
    fn from(value: AeadAlgorithm) -> Self {
        match value {
            AeadAlgorithm::AesGcm => Tag::AesGcm,
            AeadAlgorithm::Salsa20Poly1305 => Tag::Salsa20Poly1305,
            AeadAlgorithm::Unsupported(tag) => tag,
        }
    }
}

impl From<Tag> for AeadAlgorithm {
    fn from(value: Tag) -> Self {
        match value {
            Tag::AesGcm => AeadAlgorithm::AesGcm,
            Tag::Salsa20Poly1305 => AeadAlgorithm::Salsa20Poly1305,
            tag @ _ => AeadAlgorithm::Unsupported(tag),
        }
    }
}
