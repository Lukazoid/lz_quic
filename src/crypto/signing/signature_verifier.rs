use errors::*;
use crypto::certificates::Certificate;
use crypto::signing::Signature;

pub trait SignatureVerifier {
    fn verify(&self, certificate: &Certificate, data: &[u8], proof: &Signature) -> Result<()>;
}
