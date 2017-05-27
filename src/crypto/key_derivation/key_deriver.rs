use errors::*;
use handshake::{ClientHelloMessage, ServerConfiguration};
use crypto::{SharedKey, DiversificationNonce};
use crypto::certificates::Certificate;
use crypto::key_derivation::DerivedKeys;

pub trait KeyDeriver {
    fn derive_keys(&self, shared_key: &SharedKey, nonce: &[u8], client_hello_message: &ClientHelloMessage, server_configuration: &ServerConfiguration, leaf_certificate: &Certificate, diversification_nonce: Option<&DiversificationNonce>) -> Result<DerivedKeys>;
}

