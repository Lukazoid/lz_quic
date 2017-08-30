use errors::*;
use crypto::{PublicKey, SharedKey};
use crypto::key_exchange::KeyExchange;
use ring::agreement::{self, Algorithm, EphemeralPrivateKey};
use ring::rand::SystemRandom;
use untrusted::Input;
use std::fmt::{Result as FmtResult, Formatter, Debug};

/// A struct for implementing `KeyExchange` using `::ring::agreement::Algorithm`.
pub struct RingKeyExchange {
    ephemeral_private_key: EphemeralPrivateKey,
    public_key: PublicKey,
}

impl Debug for RingKeyExchange {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "RingKeyExchange {{ public_key: {:?}, ephemeral_private_key: redacted }}", &self.public_key) 
    }
}

impl RingKeyExchange {
    /// Consructs a new `RingKeyExchange`.
    ///
    /// # Errors
    /// When key exchange private or public key could not be calculated, this could be due to an issue with the system crypto random number generator.
    pub fn new(algorithm: &'static Algorithm) -> Result<Self> {
        let rng = SystemRandom::new();

        let ephemeral_private_key = EphemeralPrivateKey::generate(algorithm, &rng)
            .chain_err(||ErrorKind::FailedToCreateEphemerealPrivateKey)?;
      
        let mut public_key_bytes = vec![0u8; ephemeral_private_key.public_key_len()];

        ephemeral_private_key.compute_public_key(&mut public_key_bytes)
            .chain_err(||ErrorKind::FailedToComputePublicKey)?;

        Ok(Self {
            ephemeral_private_key: ephemeral_private_key,
            public_key: public_key_bytes.as_slice().into()
        })     
    }
}

impl KeyExchange for RingKeyExchange {
    fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    fn calculate_shared_key(self, other_public_key: &PublicKey) -> Result<SharedKey> {
        let ephemeral_private_key = self.ephemeral_private_key;

        let peer_public_key = Input::from(other_public_key.bytes());
        
        let algorithm = ephemeral_private_key.algorithm();
        agreement::agree_ephemeral(ephemeral_private_key, 
            algorithm, 
            peer_public_key,
            Error::from(ErrorKind::FailedToPerformKeyAgreement),
            |shared_key| Ok(SharedKey::from(shared_key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::agreement;

    #[test]
    pub fn calculate_shared_key_works() {
        let endpoint_a = RingKeyExchange::new(&agreement::X25519).unwrap();
        let endpoint_b = RingKeyExchange::new(&agreement::X25519).unwrap();

        let endpoint_a_public_key = endpoint_a.public_key().clone();

        let shared_key_a = endpoint_a.calculate_shared_key(endpoint_b.public_key()).unwrap();
        let shared_key_b = endpoint_b.calculate_shared_key(&endpoint_a_public_key).unwrap();

        assert_eq!(shared_key_a, shared_key_b);
    }
}