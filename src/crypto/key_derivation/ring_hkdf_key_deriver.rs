use errors::*;
use handshake::{ClientHelloMessage, ServerConfiguration, HandshakeMessage};
use protocol::{Perspective, ConnectionId};
use crypto::key_derivation::{DerivedKeys, KeyDeriver};
use crypto::certificates::Certificate;
use crypto::{InitializationVector, SharedKey, SecretKey, DiversificationNonce};
use ring::hkdf;
use ring::hmac::SigningKey;
use ring::digest::{self, Algorithm};
use protocol::Writable;

#[derive(Debug)]
pub struct RingHkdfKeyDeriver {
    algorithm: &'static Algorithm,
    is_forward_secure: bool,
    perspective: Perspective,
    key_len: usize,
    connection_id: ConnectionId,
}

impl RingHkdfKeyDeriver {
    pub fn new(is_forward_secure: bool, perspective: Perspective, connection_id: ConnectionId, algorithm: &'static Algorithm, key_len: usize) -> RingHkdfKeyDeriver {
        RingHkdfKeyDeriver {
            algorithm: algorithm,
            is_forward_secure: is_forward_secure,
            perspective: perspective,
            key_len: key_len,
            connection_id: connection_id
        }
    }
}

impl RingHkdfKeyDeriver {
    fn build_prk_input(&self, client_hello_message: &ClientHelloMessage, server_configuration: &ServerConfiguration, leaf_certificate: &Certificate) -> Vec<u8> {
        let mut info = Vec::new();

        // 1. The label “QUIC key expansion”.
        // When the forward-secret keys are derived, the same inputs are used except that info uses the label “QUIC forward secure key expansion”.
        info.extend_from_slice(if self.is_forward_secure {
                                   b"QUIC forward secure key expansion"
                               } else {
                                   b"QUIC key expansion"
                               });

        // 2. An 0x00 byte.
        info.push(0);

        // 3. The GUID of the connection from the packet layer.
        self.connection_id.write_to_vec(&mut info);

        // TODO LH Write the rest of the data to derive the keys
        // 4. The client hello message.
        HandshakeMessage::write_client_hello(&mut info, client_hello_message).expect("there should be no issue writing to a vec");

        // 5. The server config message.
        HandshakeMessage::write_server_configuration(&mut info, server_configuration).expect("there should be no issue writing to a vec");

        // 6. The DER encoded contents of the leaf certificate
        info.extend_from_slice(leaf_certificate.bytes());

        info
    }

    fn hkdf_expand(&self, salt: &SigningKey, shared_key: &SharedKey, len: usize, client_hello: &ClientHelloMessage, server_configuration: &ServerConfiguration, leaf_certificate: &Certificate) -> Vec<u8> {
        let info = self.build_prk_input(client_hello, server_configuration, leaf_certificate);

        let mut out = vec![0u8; len];
        hkdf::extract_and_expand(salt, shared_key.bytes(), &info, &mut out);

        out
    }

    fn diversify(&self, key: &mut SecretKey, iv: &mut InitializationVector, diversification_nonce: &DiversificationNonce) {
        let mut key_len;
        let mut iv_len;
        let mut out;
        {
            let key_bytes = key.bytes();    
            key_len = key_bytes.len();

            let iv_bytes = iv.bytes();
            iv_len = iv_bytes.len();

            // The concatenation of the server write key plus the server write IV from the found round is the input keying material (IKM) for the HKDF-Extract function.
            let ikm = [key_bytes, iv_bytes].concat();

            // The salt input is the diversification nonce.
            let salt = SigningKey::new(self.algorithm, diversification_nonce.bytes());

            out = vec![0u8; key_len + iv_len];
            // The info input (context and application specific information) is the label "QUIC key diversification".
            hkdf::extract_and_expand(&salt, &ikm, b"QUIC key diversification", &mut out);
        }

        // Key material is assigned in this order:
        // 1. Server write key.
        *key = SecretKey::from(&out[..key_len]);
        
        // 2. Server write IV.
        *iv = InitializationVector::from(&out[key_len..]);       
    }
}

impl KeyDeriver for RingHkdfKeyDeriver {
    fn derive_keys(&self, shared_key: &SharedKey, nonce: &[u8], client_hello_message: &ClientHelloMessage, server_configuration: &ServerConfiguration, leaf_certificate: &Certificate, diversification_nonce: Option<&DiversificationNonce>) -> Result<DerivedKeys> {
        let salt = SigningKey::new(self.algorithm, nonce);

        let iv_start = 2*self.key_len;
        let iv_length = 4;

        let key_material = self.hkdf_expand(&salt, shared_key, (2 * self.key_len) + (2 * iv_length), client_hello_message, server_configuration, leaf_certificate);
        
        let keys = &key_material[..iv_start];

        // Key material is assigned in this order:
        // 1. Client write key.
        let client_key = SecretKey::from(&keys[..self.key_len]);

        // 2. Server write key.
        let mut server_key = SecretKey::from(&keys[self.key_len..]);

        let ivs = &key_material[iv_start..];

        // 3. Client write IV.
        let client_iv = InitializationVector::from(&ivs[..iv_length]);

        // 4. Server write IV.
        let mut server_iv = InitializationVector::from(&ivs[iv_length..]);

        // When the server’s initial keys are derived, they must be diversified to ensure that the server is able to provide entropy into the HKDF.
        if let Some(diversification_nonce) = diversification_nonce {
            self.diversify(&mut server_key, &mut server_iv, diversification_nonce);
        }

        let derived_keys = match self.perspective {
            Perspective::Client => {
                DerivedKeys {
                    local_key: client_key,
                    local_iv: client_iv,
                    remote_key: server_key,
                    remote_iv: server_iv,
                }
            }
            Perspective::Server => {
                DerivedKeys {
                    local_key: server_key,
                    local_iv: server_iv,
                    remote_key: client_key,
                    remote_iv: client_iv,
                }
            }
        };

        Ok(derived_keys)
    }
}

