use errors::*;
use crypto::certificates::Certificate;
use crypto::signing::{SignatureVerifier, Signature};
use untrusted::Input;
use webpki::{self, EndEntityCert};

#[derive(Debug, Default)]
pub struct WebPkiSignatureVerifier;

impl SignatureVerifier for WebPkiSignatureVerifier {
    fn verify(&self, certificate: &Certificate, data: &[u8], signature: &Signature) -> Result<()> {
        // We have to map the error as webpki::Error does not currently implement the Error trait (see https://github.com/briansmith/webpki/pull/3)
        let end_entity_cert = EndEntityCert::from(Input::from(certificate.bytes()))
            .map_err(|e| Error::from(format!("{:?}", e)))
            .chain_err(|| ErrorKind::FailedToParseCertificate)?;

        // currently webpki exposes no way to check what algorithm the public key is using
        // so we will just try each supported algorithm in turn, this is probably slow but
        // current we have no choice
        let algorithms = [
            &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
            &webpki::ECDSA_P256_SHA256,
        ];

        let data = Input::from(data);
        let signature = Input::from(signature.bytes());

        for algorithm in algorithms.into_iter() {
            match end_entity_cert.verify_signature(algorithm, data, signature) {
                Ok(_) => {
                    return Ok(());
                }
                Err(webpki::Error::UnsupportedSignatureAlgorithmForPublicKey) => {
                    // try the next algorithm
                }
                Err(e) => {
                    return Err(Error::from(format!("{:?}", e)))
                        .chain_err(||ErrorKind::FailedToVerifyServerProof);
                }
            }
        }

        Err(Error::from(ErrorKind::FailedToVerifyServerProof))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_verifies_correctly() {
        let proof_verifier = WebPkiSignatureVerifier::default();

        let certificate = Certificate::from(include_bytes!("../certificates/example.cer").to_vec());
        let signature_data = include_bytes!("readme.signature.dat");
        let signature = Signature::from(signature_data as &[u8]);
        let data = include_bytes!("readme.md");

        proof_verifier.verify(&certificate, data, &signature).unwrap();
    }
}
