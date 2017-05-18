/// A public key which is fine to expose to third parties.
///
/// Keys of this kind are usually transferred between the endpoints to be used in a form of key exchange.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

// impl<I:IntoIterator<Item=u8>> From<I> for PublicKey{
//     default fn from(value: I) -> Self {
//         value.into_iter().collect().into()
//     }
// }

impl From<Vec<u8>> for PublicKey {
    fn from(value: Vec<u8>) -> Self {
        PublicKey(value)
    }
}

impl<'a> From<&'a [u8]> for PublicKey {
    fn from(value:&'a [u8]) -> Self {
        value.to_vec().into()
    }
}
