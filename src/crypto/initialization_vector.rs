#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct InitializationVector(Vec<u8>);

impl InitializationVector {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for InitializationVector {
    fn from(value: Vec<u8>) -> Self {
        InitializationVector(value)
    }
}

impl<'a> From<&'a [u8]> for InitializationVector {
    fn from(value: &'a [u8]) -> Self {
        value.to_vec().into()
    }
}

