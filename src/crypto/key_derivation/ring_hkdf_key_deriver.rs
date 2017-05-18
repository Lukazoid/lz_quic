use errors::*;
use protocol::{Perspective, ConnectionId};
use crypto::key_derivation::{DerivedKeys, KeyDeriver};
use crypto::{InitializationVector, SharedKey, SecretKey};
use ring::hkdf;
use ring::hmac::SigningKey;
use ring::digest::{self, Algorithm};
use protocol::Writable;

#[derive(Debug)]
pub struct RingHkdfKeyDeriver {
    forward_secure: bool,
    perspective: Perspective,
    algorithm: &'static Algorithm,
    key_len: usize,
    secret: Vec<u8>,
    connection_id: ConnectionId,
}

// impl RingHkdfKeyDeriver {
//     pub fn new(algorithm: &'static Algorithm, forward_secure: bool) -> RingHkdfKeyDeriver {

//         RingHkdfKeyDeriver {
//             algorithm: algorithm,
//             forward_secure: forward_secure,
//         }
//     }
// }

impl KeyDeriver for RingHkdfKeyDeriver {
    fn derive_keys(&self) -> Result<DerivedKeys> {
        let salt = [0u8; 0];
        let salt = SigningKey::new(self.algorithm, &salt);

        let mut info = Vec::new();
        info.extend_from_slice(if self.forward_secure {
                                   b"QUIC forward secure key expansion\0"
                               } else {
                                   b"QUIC key expansion\0"
                               });

        self.connection_id.write_to_vec(&mut info);

        let mut out = vec![0u8; (2 * self.key_len) + (2 * 4)];
        hkdf::extract_and_expand(&salt, &self.secret, &info, &mut out);

        unimplemented!()
    }
}

