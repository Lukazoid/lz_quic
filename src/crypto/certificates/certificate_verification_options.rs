use crypto::certificates::TrustAnchor;

#[derive(Debug, Clone)]
pub struct CertificateVerificationOptions {
    pub certificate_authorities: Vec<TrustAnchor>,
}
