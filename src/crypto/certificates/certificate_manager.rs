use errors::*;
use crypto::certificates::{Certificate, CertificateChain, CertificateChainVerifier, TrustAnchor,
                           WebpkiCertificateChainVerifier};
use crypto::signing::{Signature, SignatureVerifier, Signer};
use crypto::certificates::CERTIFICATE_COMPRESSOR;
use std::io::Cursor;
use std::collections::HashMap;
use lz_fnv::Fnv1a;
use std::hash::{Hash, Hasher};
use handshake::{ClientHelloMessage, HandshakeMessage, ServerConfiguration};
use ring::digest::{digest, Digest, SHA256};
use protocol::Writable;

#[derive(Debug, Default)]
pub struct CertificateManager {
    certificate_chain_verifier: Option<WebpkiCertificateChainVerifier>,
    certificate_chain: Option<CertificateChain>,
}

fn build_signature_input(
    client_hello_message: &ClientHelloMessage,
    server_configuration: &ServerConfiguration,
) -> Digest {
    // The signature is calculated over:
    let mut signature_input = Vec::new();

    // 1. The label “QUIC server config signature”
    // TODO LH Is this the actual label, the go and C++ impl have:
    // "QUIC CHLO and server config signature\x00"
    signature_input.extend(b"QUIC server config signature");

    // 2. The 32 bit length of the hash in the next field in bytes (which is 8).
    8u32.write_to_vec(&mut signature_input);

    // 3. The SHA256 hash of the CHLO
    let mut client_hello_bytes = Vec::new();
    HandshakeMessage::write_client_hello(&mut client_hello_bytes, client_hello_message)
        .expect("there should be no issue writing to a vec");

    signature_input.extend(digest(&SHA256, client_hello_bytes.as_slice()).as_ref());

    // 4. An 0x00 byte
    signature_input.push(0u8);

    // 5. The serialised server config.
    HandshakeMessage::write_server_configuration(&mut signature_input, server_configuration)
        .expect("there should be no issue writing to a vec");

    digest(&SHA256, signature_input.as_slice())
}

impl CertificateManager {
    pub fn skip_verify() -> Self {
        Self {
            certificate_chain_verifier: None,
            certificate_chain: None,
        }
    }

    pub fn with_trust_anchors<I: IntoIterator<Item = TrustAnchor>>(trust_anchors: I) -> Self {
        Self {
            certificate_chain_verifier: Some(WebpkiCertificateChainVerifier::new(trust_anchors)),
            certificate_chain: None,
        }
    }

    pub fn set_data(&mut self, data: &[u8]) -> Result<()> {
        let certificate_chain = CERTIFICATE_COMPRESSOR
            .decompress_certificate_chain(&HashMap::with_capacity(0), &mut Cursor::new(data))?;

        self.certificate_chain = Some(certificate_chain);

        Ok(())
    }

    pub fn common_certificate_set_hashes(&self) -> Vec<u64> {
        CERTIFICATE_COMPRESSOR.common_certificate_set_hashes()
    }

    pub fn leaf_certificate(&self) -> Option<&Certificate> {
        self.certificate_chain
            .as_ref()
            .and_then(|chain| chain.leaf_certificate())
    }

    pub fn leaf_certificate_hash(&self) -> Option<u64> {
        self.leaf_certificate().map(|cert| {
            let mut hasher = Fnv1a::<u64>::default();
            cert.hash(&mut hasher);

            hasher.finish()
        })
    }

    pub fn sign_server_proof<S: Signer>(
        &self,
        signer: &S,
        client_hello_message: &ClientHelloMessage,
        server_configuration: &ServerConfiguration,
    ) -> Result<Signature> {
        let signature_input = build_signature_input(client_hello_message, server_configuration);

        signer.sign(signature_input.as_ref())
    }

    pub fn verify_server_proof<V: SignatureVerifier>(
        &self,
        verifier: &V,
        signature: &Signature,
        client_hello_message: &ClientHelloMessage,
        server_configuration: &ServerConfiguration,
    ) -> Result<()> {
        let leaf_certificate = self.leaf_certificate().ok_or_else(|| {
            Error::from(ErrorKind::UnableToVerifyWithoutACertificateChain)
        })?;

        let signature_input = build_signature_input(client_hello_message, server_configuration);

        verifier.verify(leaf_certificate, signature_input.as_ref(), signature)
    }

    pub fn verify(&self, host_name: &str) -> Result<()> {
        let certificate_chain = self.certificate_chain.as_ref().ok_or_else(|| {
            Error::from(ErrorKind::UnableToVerifyWithoutACertificateChain)
        })?;

        if let Some(ref verifier) = self.certificate_chain_verifier {
            verifier.verify(certificate_chain, host_name)
        } else {
            Ok(())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use handshake::{ClientHelloMessage, ServerConfiguration, ServerConfigurationId};
    use crypto::certificates::{Certificate, CertificateChain, CERTIFICATE_COMPRESSOR};
    use crypto::signing::{RsaSigner, WebPkiSignatureVerifier};
    use crypto::{Proof, SharedKey};
    use protocol::version;
    use std::collections::HashSet;
    use smallvec::SmallVec;
    use untrusted::Input;
    use webpki::{self, EndEntityCert};

    static EXAMPLE_KEY_BYTES: &'static [u8] = include_bytes!("example.pkcs8");
    static EXAMPLE_CERTIFICATE_BYTES: &'static [u8] = include_bytes!("example.cer");

    #[test]
    pub fn verify_server_proof_verifies_signed_server_proof() {
        let example_certificate = Certificate::from(EXAMPLE_CERTIFICATE_BYTES.to_vec());

        let certificate_chain = CertificateChain::from(vec![example_certificate]);

        let mut compressed_certificate_chain = Vec::new();
        CERTIFICATE_COMPRESSOR
            .compress_certificate_chain(
                certificate_chain.certificates(),
                &HashSet::with_capacity(0),
                &HashSet::with_capacity(0),
                &mut compressed_certificate_chain,
            )
            .unwrap();

        let mut certificate_manager = CertificateManager::skip_verify();
        certificate_manager
            .set_data(&compressed_certificate_chain)
            .unwrap();

        let proof_signer = RsaSigner::from_pkcs8(EXAMPLE_KEY_BYTES).unwrap();

        let client_hello_message = ClientHelloMessage {
            server_name: Some("example.com".to_owned()),
            source_address_token: Some([1u8, 2, 3][..].into()),
            proof_demands: [Proof::X509].as_ref().into(),
            common_certificate_sets: vec![],
            cached_certificates: vec![],
            version: version::DRAFT_IETF_01,
            leaf_certificate: certificate_manager.leaf_certificate_hash(),
        };

        let server_configuration = ServerConfiguration {
            server_configuration_id: ServerConfigurationId::from([0u8; 16]),
            key_exchange_algorithms: SmallVec::default(),
            aead_algorithms: SmallVec::default(),
            public_keys: SmallVec::default(),
            orbit: 0,
            expiry_time: 0,
            versions: SmallVec::default(),
            shared_key: SharedKey::from(&[0u8; 10][..]),
        };

        let signature = certificate_manager
            .sign_server_proof(&proof_signer, &client_hello_message, &server_configuration)
            .unwrap();

        let proof_verifier = WebPkiSignatureVerifier::default();
        certificate_manager
            .verify_server_proof(
                &proof_verifier,
                &signature,
                &client_hello_message,
                &server_configuration,
            )
            .unwrap();
    }
}
