use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    data: Vec<u8>,
}

impl Certificate {
    pub fn bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Hash for Certificate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.data);
    }
}

impl From<Vec<u8>> for Certificate {
    fn from(value: Vec<u8>) -> Self {        
        Self { data: value }
    }
}

impl<'a> From<&'a [u8]> for Certificate{
    fn from(value:&'a [u8]) -> Self {
        value.to_vec().into()
    }
}