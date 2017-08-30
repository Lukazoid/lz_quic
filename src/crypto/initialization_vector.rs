use smallvec::SmallVec;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InitializationVector(SmallVec<[u8;4]>);

impl InitializationVector {
    pub fn from_iterator<I: IntoIterator<Item=u8>>(value: I) -> Self {
        InitializationVector(value.into_iter().collect())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> From<&'a [u8]> for InitializationVector {
    fn from(value: &'a [u8]) -> Self {
        InitializationVector(value.into())
    }
}