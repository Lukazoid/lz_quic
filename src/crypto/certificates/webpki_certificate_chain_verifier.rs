use errors::*;
use crypto::certificates::{Certificate, CertificateChain, CertificateChainVerifier, TrustAnchor};
use webpki::{self, EndEntityCert, SignatureAlgorithm};
use untrusted::Input;
use time::{self, Timespec};

/// A `CertificateChainVerifier` using the verify methods from `webpki`.
#[derive(Debug)]
pub struct WebpkiCertificateChainVerifier {
    trust_anchors: Vec<TrustAnchor>
}

static SUPPORTED_SIGNATURE_ALGORITHMS : &'static[&'static SignatureAlgorithm] = &[
        &webpki::ECDSA_P256_SHA256,
        &webpki::ECDSA_P384_SHA384,
        &webpki::RSA_PKCS1_2048_8192_SHA256,
        &webpki::RSA_PKCS1_2048_8192_SHA384,
        &webpki::RSA_PKCS1_2048_8192_SHA512,
        &webpki::RSA_PKCS1_3072_8192_SHA384,
        &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
        &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
        &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY
    ];


fn as_webpki_cert<'a>(certificate: &'a Certificate) -> Result<EndEntityCert<'a>> {

    // We have to map the error as webpki::Error does not currently implement the Error trait (see https://github.com/briansmith/webpki/pull/3)
    EndEntityCert::from(Input::from(certificate.bytes()))
        .map_err(|e| Error::from(format!("{:?}", e)))
        .chain_err(||ErrorKind::UnableToParseCertificateFromCertificateChain)
}

impl<'a> WebpkiCertificateChainVerifier
{
    pub fn new(trust_anchors: Vec<TrustAnchor>) -> Self {
        WebpkiCertificateChainVerifier{
            trust_anchors: trust_anchors
        }
    }
}

impl CertificateChainVerifier for WebpkiCertificateChainVerifier {
    fn verify(&self, certificate_chain: &CertificateChain, host_name: &str) -> Result<()> {
        if let Some(leaf_certificate) = certificate_chain.leaf_certificate() {
            let webpki_cert = as_webpki_cert(leaf_certificate)?;

            // We have to map the error as webpki::Error does not currently implement the Error trait (see https://github.com/briansmith/webpki/pull/3)
            webpki_cert.verify_is_valid_for_dns_name(Input::from(host_name.as_bytes()))
                .map_err(|e| Error::from(format!("{:?}", e)))
                .chain_err(||ErrorKind::CertificateInvalidForDnsName(host_name.to_owned()))?;

            let intermediate_certificates: Vec<_> = certificate_chain.intermediate_certificates()
                .into_iter()
                .map(|c| Input::from(c.bytes()))
                .collect();
                
            let now = time::now_utc().to_timespec();

            let webpki_trust_anchors : Vec<_> = self.trust_anchors.iter().map(|ta|ta.as_webpki_trust_anchor()).collect();

            // We have to map the error as webpki::Error does not currently implement the Error trait (see https://github.com/briansmith/webpki/pull/3)
            webpki_cert.verify_is_valid_tls_server_cert(SUPPORTED_SIGNATURE_ALGORITHMS, &webpki_trust_anchors, &intermediate_certificates, now)
                .map_err(|e| Error::from(format!("{:?}", e)))
                .chain_err(||ErrorKind::InvalidTlsCertificate)?;

            Ok(())
        } else {
            bail!(ErrorKind::CertificateChainIsEmpty);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crypto::certificates::CertificateChain;
    use webpki_roots;
    use crypto::certificates::TrustAnchor;

    #[test]
    pub fn verify_with_empty_chain_returns_error() {
        let webpki_certificate_chain_verifier = WebpkiCertificateChainVerifier::new(webpki_roots::ROOTS.iter().map(TrustAnchor::from_webpki_trust_anchor).collect());
        let certificate_chain = CertificateChain::from(vec![]);

        let verify_result = webpki_certificate_chain_verifier.verify(&certificate_chain, "google.com");
        
        assert!(verify_result.is_err());
    }

    #[test]
    pub fn verify_with_correct_chain_succeeds() {
        let webpki_certificate_chain_verifier = WebpkiCertificateChainVerifier::new(webpki_roots::ROOTS.iter().map(TrustAnchor::from_webpki_trust_anchor).collect());
        
        let google_certificate = Certificate::from(include_bytes!("google.com.cer").to_vec());
        let google_internet_authority_certificate = Certificate::from(include_bytes!("google_internet_authority_g2.cer").to_vec());

        let certificate_chain = CertificateChain::from(vec![google_certificate, google_internet_authority_certificate]);

        let verify_result = webpki_certificate_chain_verifier.verify(&certificate_chain, "google.com");
        
        verify_result.unwrap();
    }
  
    #[test]
    pub fn verify_with_wrong_host_name_returns_error() {
        let webpki_certificate_chain_verifier = WebpkiCertificateChainVerifier::new(webpki_roots::ROOTS.iter().map(TrustAnchor::from_webpki_trust_anchor).collect());
        
        let google_certificate = Certificate::from(include_bytes!("google.com.cer").to_vec());
        let google_internet_authority_certificate = Certificate::from(include_bytes!("google_internet_authority_g2.cer").to_vec());

        let certificate_chain = CertificateChain::from(vec![google_certificate, google_internet_authority_certificate]);

        let verify_result = webpki_certificate_chain_verifier.verify(&certificate_chain, "google.com.notreally.tk");
        
        assert!(verify_result.is_err());
    }
}

