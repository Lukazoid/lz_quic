use std::fmt::{Result as FmtResult, Formatter, Debug};

pub struct SecretKey(Vec<u8>);

impl Debug for SecretKey {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "SecretKey (redacted)") 
    }
}

impl SecretKey {
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for SecretKey {
    fn from(value: Vec<u8>) -> Self {
        SecretKey(value)
    }
}

impl<'a> From<&'a [u8]> for SecretKey {
    fn from(value:&'a [u8]) -> Self {
        value.to_vec().into()
    }
}
