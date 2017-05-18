use errors::*;
use crypto::{PublicKey, SharedKey, SecretKey};

/// For types which can perform key exchange.
///
/// This allows shared `PublicKey` values to be used to determine a common `SharedKey` without revealing the `SecretKey`.
///
/// Each value is only valid for calculating one `SharedKey`, this helps ensure endpoints do not mistakenly use the same `SharedKey` values.
pub trait KeyExchange {
    /// The `PublicKey` which may be shared between endpoints.
    fn public_key(&self) -> &PublicKey;

    /// Calculates the `SharedKey` which is the same between both endpoints.
    ///
    /// # Errors
    /// When `other_public_key` is invalid.
    fn calculate_shared_key(self, other_public_key: &PublicKey) -> Result<SharedKey>;
}

