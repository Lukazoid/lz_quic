use smallvec::SmallVec;

/// A key which is "shared" between both endpoints.
///
/// Shared only implies that the endpoints have the same key, often determined through some form of key exchange.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct SharedKey(SmallVec<[u8;32]>);

impl SharedKey {
    pub fn from_iterator<I: IntoIterator<Item=u8>>(value: I) -> Self {
        SharedKey(value.into_iter().collect())
    }
    
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> From<&'a [u8]> for SharedKey {
    fn from(value: &'a [u8]) -> Self {
        SharedKey(value.into())
    }
}

