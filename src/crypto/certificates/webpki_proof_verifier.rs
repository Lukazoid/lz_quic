use errors::*;
use crypto::certificates::{Certificate, ProofVerifier};
use untrusted::Input;
use webpki::{self, EndEntityCert};

#[derive(Debug, Default)]
pub struct WebPkiProofVerifier;

impl ProofVerifier for WebPkiProofVerifier {
    fn verify(&self, certificate: &Certificate, data: &[u8], proof: &[u8]) -> Result<()> {
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
        let proof = Input::from(proof);

        for algorithm in algorithms.into_iter() {
            match end_entity_cert.verify_signature(algorithm, data, proof) {
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
        let proof_verifier = WebPkiProofVerifier::default();

        let certificate = Certificate::from(include_bytes!("example.cer").to_vec());
        let signature = include_bytes!("readme.signature.dat");
        let proof = include_bytes!("readme.md");

        proof_verifier.verify(&certificate, proof, signature).unwrap();
    }
}
