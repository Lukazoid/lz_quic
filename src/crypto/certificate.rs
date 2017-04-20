use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Certificate {
    data: Vec<u8>,
}

impl Certificate {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data: data }
    }

    pub fn bytes(&self) -> &Vec<u8> {
        &self.data
    }
}

impl Hash for Certificate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.data);
    }
}
