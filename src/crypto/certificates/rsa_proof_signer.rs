use errors::*;
use crypto::certificates::ProofSigner;
use ring::signature::{self, RSAKeyPair, RSASigningState};
use ring::rand::SystemRandom;
use std::sync::{Arc, Mutex};
use untrusted::Input;

pub struct RsaProofSigner {
    // TODO LH Eventually introduce a pool of RSASigningState
    rsa_signing_state: Mutex<RSASigningState>,
}

impl RsaProofSigner {
    pub fn from_pkcs8(private_key: &[u8]) -> Result<Self> {
        let rsa_key_pair = RSAKeyPair::from_pkcs8(Input::from(private_key))
            .map_err(|e| Error::from(format!("{:?}", e)))
            .chain_err(|| ErrorKind::FailedToParseRsaKeyPair)?;

        let rsa_signing_state = RSASigningState::new(Arc::new(rsa_key_pair))
            .chain_err(|| ErrorKind::FailedToBuildRsaSigningState)?;

        Ok(RsaProofSigner {
            rsa_signing_state: Mutex::new(rsa_signing_state),
        })
    }
}

impl ProofSigner for RsaProofSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut rsa_signing_state = self.rsa_signing_state.lock().unwrap();

        let mut signature = vec![0u8; rsa_signing_state.key_pair().public_modulus_len()];

        let rng = SystemRandom::new();

        rsa_signing_state
            .sign(
                &signature::RSA_PSS_SHA256,
                &rng,
                data,
                &mut signature,
            )
            .chain_err(|| ErrorKind::FailedToSignServerProof)?;

        Ok(signature)
    }
}