use errors::*;
use crypto::certificates::Certificate;

pub trait ProofVerifier {
    fn verify(&self, certificate: &Certificate, data: &[u8], proof: &[u8]) -> Result<()>;
}
