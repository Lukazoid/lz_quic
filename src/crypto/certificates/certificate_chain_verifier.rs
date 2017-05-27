use errors::*;
use crypto::certificates::CertificateChain;

/// Trait for types which verify a `CertificateChain` is valid.
pub trait CertificateChainVerifier {
    /// Verify that `certificate_chain` is valid for use with `host_name`.
    ///
    /// # Errors
    /// When `certificate_chain` was not valid for use with `host_name`.
    fn verify(&self, certificate_chain: &CertificateChain, host_name: &str) -> Result<()>;
}

