use errors::*;
use crypto::certificates::CertificateChain;

pub trait CertificateChainVerifier {
    fn verify(&self, certificate_chain: &CertificateChain, host_name: &str) -> Result<()>;
}
