#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum EncryptionLevel {
    Unencrypted,
    NonForwardSecure,
    ForwardSecure,
}