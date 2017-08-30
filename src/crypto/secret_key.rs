use smallvec::SmallVec;
use std::fmt::{Result as FmtResult, Formatter, Debug};

#[derive(PartialEq, Eq, Hash)]
pub struct SecretKey(SmallVec<[u8;16]>);

impl Debug for SecretKey {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "SecretKey (redacted)")
    }
}

impl SecretKey {
    pub fn from_iterator<I: IntoIterator<Item=u8>>(value: I) -> Self {
        SecretKey(value.into_iter().collect())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> From<&'a [u8]> for SecretKey {
    fn from(value: &'a [u8]) -> Self {
        SecretKey(value.into())
    }
}

