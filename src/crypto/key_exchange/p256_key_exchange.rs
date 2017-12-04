use errors::*;
use ring::agreement;
use crypto::{PublicKey, SharedKey};
use crypto::key_exchange::{KeyExchange, RingKeyExchange};

#[derive(Debug)]
pub struct P256KeyExchange {
    inner: RingKeyExchange,
}

impl P256KeyExchange {
    pub fn new() -> Result<Self> {
        trace!("creating new p256 key exchange");
        let inner = RingKeyExchange::new(&agreement::ECDH_P256)?;
        let key_exchange = Self { inner: inner };
        trace!("created new p256 key exchange");
        Ok(key_exchange)
    }
}

impl KeyExchange for P256KeyExchange {
    fn public_key(&self) -> &PublicKey {
        self.inner.public_key()
    }

    fn calculate_shared_key(self, other_public_key: &PublicKey) -> Result<SharedKey> {
        self.inner.calculate_shared_key(other_public_key)
    }
}
