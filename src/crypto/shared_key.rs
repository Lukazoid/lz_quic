/// A key which is "shared"" between both endpoints.
///
/// Shared only implies that the endpoints have the same key, often determined through some form of key exchange.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct SharedKey(Vec<u8>);

impl SharedKey {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for SharedKey {
    fn from(value: Vec<u8>) -> Self {
        SharedKey(value)
    }
}

impl<'a> From<&'a [u8]> for SharedKey {
    fn from(value: &'a [u8]) -> Self {
        value.to_vec().into()
    }
}

