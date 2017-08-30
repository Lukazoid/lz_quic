use errors::*;
use std::io::{Read, Write};
use protocol::{Readable, Writable};
use smallvec::SmallVec;

/// A public key which is fine to expose to third parties.
///
/// Keys of this kind are usually transferred between the endpoints to be used in a form of key exchange.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PublicKey(SmallVec<[u8;32]>);

impl PublicKey {
    pub fn from_iterator<I: IntoIterator<Item=u8>>(iterator: I) -> Self {
        PublicKey(iterator.into_iter().collect())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> From<&'a [u8]> for PublicKey {
    fn from(value:&'a [u8]) -> Self {
        PublicKey(value.into())
    }
}

impl Readable for PublicKey {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)
            .chain_err(|| ErrorKind::FailedToReadPublicKeyBytes)?;
            
        Ok(bytes.as_slice().into())
    }
}

impl Writable for PublicKey {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(self.bytes())
            .chain_err(||ErrorKind::FailedToWritePublicKeyBytes)
    }
}