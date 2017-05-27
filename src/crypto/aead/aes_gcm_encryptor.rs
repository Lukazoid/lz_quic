use errors::*;
use crypto::aead::{self, AeadEncryptor};
use openssl::symm::{encrypt_aead, Cipher};
use packets::PacketNumber;
use crypto::{InitializationVector, SecretKey};

#[derive(Debug)]
pub struct AesGcmEncryptor {
    secret_key: SecretKey,
    iv: InitializationVector,
}

impl AesGcmEncryptor {
    pub fn new(secret_key: SecretKey, iv: InitializationVector) -> AesGcmEncryptor {
        AesGcmEncryptor {
            secret_key: secret_key,
            iv: iv,
        }
    }
}



impl AeadEncryptor for AesGcmEncryptor {
    fn encrypt(&mut self,
               associated_data: &[u8],
               plain_text: &[u8],
               packet_number: PacketNumber)
               -> Result<Vec<u8>> {

        let nonce = aead::make_nonce(&self.iv, packet_number);

        let cipher = Cipher::aes_128_gcm();

        let mut tag = [0u8; 12];

        // Using openssl here as it is the only crate which currently supports variable tag widths
        let mut encrypted = encrypt_aead(cipher,
                                         self.secret_key.bytes(),
                                         Some(&nonce),
                                         associated_data,
                                         plain_text,
                                         &mut tag)
                .chain_err(|| ErrorKind::UnableToPerformAesGcmEncryption)?;

        encrypted.extend_from_slice(&tag);

        Ok(encrypted)
    }
}

