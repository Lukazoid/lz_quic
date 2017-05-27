use errors::*;
use handshake::{ClientHelloMessage, ServerConfiguration};
use ring::digest::SHA256;
use protocol::{Perspective, ConnectionId};
use crypto::{SharedKey, DiversificationNonce};
use crypto::key_derivation::{DerivedKeys, KeyDeriver, RingHkdfKeyDeriver};
use crypto::certificates::Certificate;

#[derive(Debug)]
pub struct Sha256HkdfKeyDeriver {
    inner: RingHkdfKeyDeriver
}

impl Sha256HkdfKeyDeriver{
    pub fn new(is_forward_secure: bool, perspective: Perspective, connection_id: ConnectionId, key_len: usize) -> Self {
        Sha256HkdfKeyDeriver {
           inner: RingHkdfKeyDeriver::new(is_forward_secure, perspective, connection_id, &SHA256, key_len)
        }
    }
}
impl KeyDeriver for Sha256HkdfKeyDeriver {
    fn derive_keys(&self, shared_key: &SharedKey, nonce: &[u8], client_hello_message: &ClientHelloMessage, server_configuration: &ServerConfiguration, leaf_certificate: &Certificate, diversification_nonce: Option<&DiversificationNonce>) -> Result<DerivedKeys> {
        self.inner.derive_keys(shared_key, nonce, client_hello_message, server_configuration, leaf_certificate, diversification_nonce)
    }
}