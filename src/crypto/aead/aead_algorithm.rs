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
        trace!("writing aead algorithm {:?}", self);
        let tag: Tag = (*self).into();

        tag.write(writer)?;
        debug!("written aead algorithm {:?}", self);
        Ok(())
    }
}

impl Readable for AeadAlgorithm {
    fn read<R: Read>(reader: &mut R) -> Result<AeadAlgorithm> {
        trace!("reading aead algorithm");
        let tag = Tag::read(reader)?;

        let aead_algorithm = AeadAlgorithm::from(tag);
        debug!("read aead algorithm {:?}", aead_algorithm);
        Ok(aead_algorithm)
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
