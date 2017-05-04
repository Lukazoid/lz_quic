use errors::*;
use ring::agreement;
use crypto::ring_key_exchange::RingKeyExchange;
use crypto::key_exchange::KeyExchange;

pub struct P256KeyExchange {
    inner: RingKeyExchange,
}

impl P256KeyExchange {
    pub fn new() -> Result<Self> {
        let inner = RingKeyExchange::new(&agreement::ECDH_P256)?;
        Ok(Self {
            inner: inner
        })     
    }
}

impl KeyExchange for P256KeyExchange {
    fn public_key(&self) -> &[u8] {
        self.inner.public_key()
    }

    fn calculate_shared_key(self, other_public_key: &[u8]) -> Result<Vec<u8>> {
        self.inner.calculate_shared_key(other_public_key)
    }
}