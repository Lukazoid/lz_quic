use errors::*;
use ring::agreement;
use crypto::{PublicKey, SharedKey};
use crypto::key_exchange::{KeyExchange, RingKeyExchange};

/// A `KeyExchange` implementation which uses Curve25519.
#[derive(Debug)]
pub struct Curve25519KeyExchange {
    inner: RingKeyExchange,
}

impl Curve25519KeyExchange {
    pub fn new() -> Result<Self> {
        let inner = RingKeyExchange::new(&agreement::X25519)?;
        Ok(Self { inner: inner })
    }
}

impl KeyExchange for Curve25519KeyExchange {
    fn public_key(&self) -> &PublicKey {
        self.inner.public_key()
    }

    fn calculate_shared_key(self, other_public_key: &PublicKey) -> Result<SharedKey> {
        self.inner.calculate_shared_key(other_public_key)
    }
}

