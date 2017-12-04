use errors::*; 
use crypto::certificates::{Certificate, CertificateChain, CertificateChainVerifier, TrustAnchor}; 
use webpki::{self, EndEntityCert, SignatureAlgorithm, TLSServerTrustAnchors, Time}; 
use untrusted::Input; 
use std::time::{self, SystemTime}; 
 
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
    trace!("creating webpki certificate");
    // We have to map the error as webpki::Error does not currently implement the Error trait (see https://github.com/briansmith/webpki/pull/3) 
    let end_entity_cert = EndEntityCert::from(Input::from(certificate.bytes())) 
        .map_err(|e| Error::from(format!("{:?}", e))) 
        .chain_err(||ErrorKind::FailedToParseCertificateFromCertificateChain)?;
    debug!("created webpki certificate");
    Ok(end_entity_cert)
} 
 
impl<'a> WebpkiCertificateChainVerifier { 
    pub fn new<I:IntoIterator<Item=TrustAnchor>>(trust_anchors: I) -> Self {
        trace!("creating webpki certificate chain verifier");
        let verifier = WebpkiCertificateChainVerifier{ 
            trust_anchors: trust_anchors.into_iter().collect() 
        };
        debug!("created webpki certificate chain verifier");
        verifier
    } 
} 
 
impl CertificateChainVerifier for WebpkiCertificateChainVerifier { 
    fn verify(&self, certificate_chain: &CertificateChain, host_name: &str) -> Result<()> {
        trace!("verifying certificate chain for host {}", host_name);
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
                 
            let now = Time::try_from(SystemTime::now()) 
                .chain_err(||ErrorKind::FailedToDetermineTimeSinceUnixEpoch)?; 
 
            let webpki_trust_anchors : Vec<_> = self.trust_anchors.iter().map(|ta|ta.as_webpki_trust_anchor()).collect(); 
 
            // We have to map the error as webpki::Error does not currently implement the Error trait (see https://github.com/briansmith/webpki/pull/3) 
            webpki_cert.verify_is_valid_tls_server_cert(SUPPORTED_SIGNATURE_ALGORITHMS, &TLSServerTrustAnchors(webpki_trust_anchors.as_slice()), &intermediate_certificates, now) 
                .map_err(|e| Error::from(format!("{:?}", e))) 
                .chain_err(||ErrorKind::InvalidTlsCertificate)?; 
            
            trace!("verified certificate chain for host {}", host_name);
            Ok(())
        } else { 
            bail!(ErrorKind::CertificateChainIsEmpty); 
        }
    } 
}