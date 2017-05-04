use errors::*;
use crypto::key_exchange::KeyExchange;
use ring::agreement::{self, Algorithm, EphemeralPrivateKey};
use ring::rand::SystemRandom;
use untrusted::Input;

pub struct RingKeyExchange {
    algorithm: &'static Algorithm,
    ephemeral_private_key: EphemeralPrivateKey,
    public_key: Vec<u8>,
}

impl RingKeyExchange {
    pub fn new(algorithm: &'static Algorithm) -> Result<Self> {
        let rng = SystemRandom::new();

        let ephemeral_private_key = EphemeralPrivateKey::generate(algorithm, &rng)
            .chain_err(||ErrorKind::UnableToCreateEphemerealPrivateKey)?;
      
        let mut public_key = vec![0u8; ephemeral_private_key.public_key_len()];

        ephemeral_private_key.compute_public_key(&mut public_key)
            .chain_err(||ErrorKind::UnableToComputePublicKey)?;

        Ok(Self {
            algorithm: algorithm,
            ephemeral_private_key: ephemeral_private_key,
            public_key: public_key
        })     
    }
}

impl KeyExchange for RingKeyExchange {
    fn public_key(&self) -> &[u8] {
        &self.public_key
    }

    fn calculate_shared_key(self, other_public_key: &[u8]) -> Result<Vec<u8>> {
        let ephemeral_private_key = self.ephemeral_private_key;

        let peer_public_key = Input::from(other_public_key);
        
        agreement::agree_ephemeral(ephemeral_private_key, 
            self.algorithm, 
            peer_public_key,
            Error::from(ErrorKind::UnableToPerformKeyAgreement),
            |shared_key| Ok(shared_key.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::agreement;

    #[test]
    pub fn calculate_shared_key_works() {
        let endpointA = RingKeyExchange::new(&agreement::X25519).unwrap();
        let endpointB = RingKeyExchange::new(&agreement::X25519).unwrap();

        let shared_key = endpointB.calculate_shared_key(endpointA.public_key()).unwrap();
    }
}