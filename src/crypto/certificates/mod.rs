mod trust_anchor;
pub use self::trust_anchor::TrustAnchor;

mod certificate;
pub use self::certificate::Certificate;

mod certificate_set;
pub use self::certificate_set::CertificateSet;

mod certificate_chain;
pub use self::certificate_chain::CertificateChain;

mod certificate_verification_options;
pub use self::certificate_verification_options::CertificateVerificationOptions;

mod certificate_chain_verifier;
pub use self::certificate_chain_verifier::CertificateChainVerifier;

mod webpki_certificate_chain_verifier;
pub use self::webpki_certificate_chain_verifier::WebpkiCertificateChainVerifier;

mod common_certificate_set_2;
mod common_certificate_set_3;

fn build_common_certificate_sets() -> Vec<CertificateSet> {
    vec![common_certificate_set_2::build_common_certificate_set_2(),
         common_certificate_set_3::build_common_certificate_set_3()]
}

mod certificate_compressor;
pub use self::certificate_compressor::CertificateCompressor;

lazy_static! {
    pub static ref CERTIFICATE_COMPRESSOR: CertificateCompressor = CertificateCompressor::new(build_common_certificate_sets());
}

mod certificate_manager;
pub use self::certificate_manager::CertificateManager;