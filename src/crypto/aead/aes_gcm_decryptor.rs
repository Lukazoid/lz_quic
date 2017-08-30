use errors::*;
use crypto::{InitializationVector, SecretKey};
use openssl::symm::{decrypt_aead, Cipher};
use crypto::aead::{self, AeadDecryptor};
use packets::PacketNumber;

#[derive(Debug)]
pub struct AesGcmDecryptor {
    secret_key: SecretKey,
    iv: InitializationVector,
}

impl AesGcmDecryptor {
    pub fn new(secret_key: SecretKey, iv: InitializationVector) -> Self {
        AesGcmDecryptor {
            secret_key: secret_key,
            iv: iv,
        }
    }
}

impl AeadDecryptor for AesGcmDecryptor {
    fn decrypt(
        &self,
        associated_data: &[u8],
        cipher_text: &[u8],
        packet_number: PacketNumber,
    ) -> Result<Vec<u8>> {

        let nonce = aead::make_nonce(&self.iv, packet_number);

        let cipher = Cipher::aes_128_gcm();

        let tag_start = cipher_text.len() - 12;
        let (tag, cipher_text) = cipher_text.split_at(cipher_text.len() - 12);

        // Using openssl here as it is the only crate which currently supports variable tag widths
        decrypt_aead(
            cipher,
            self.secret_key.bytes(),
            Some(&nonce),
            associated_data,
            cipher_text,
            tag,
        ).chain_err(|| ErrorKind::FailedToPerformAesGcmDecryption)
    }
}
