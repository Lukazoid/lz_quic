use crypto::{InitializationVector, SecretKey};

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct DerivedKeys {
    pub local_key: SecretKey,
    pub local_iv: InitializationVector,
    pub remote_key: SecretKey,
    pub remote_iv: InitializationVector,
}

