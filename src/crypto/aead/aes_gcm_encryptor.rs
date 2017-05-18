use crypto::aead::AeadEncryptor;
use openssl::symm::{encrypt_aead, Cipher};
use packets::PacketNumber;
use crypto::InitializationVector;

pub struct AesGcmEncryptor {
    key: Vec<u8>,
    iv: InitializationVector,
}

impl AesGcmEncryptor {
    pub fn new(packet_number: PacketNumber) -> AesGcmEncryptor {
        unimplemented!()
    }
}

impl AeadEncryptor for AesGcmEncryptor {
    fn encrypt(&mut self, associated_data: &[u8], plain_text: &[u8]) -> Vec<u8> {
        let cipher = Cipher::aes_128_gcm();

        let mut tag = vec![0u8; 12];

        // Using openssl here as it is the only crate which currently supports variable tag widths
        encrypt_aead(cipher,
                     &self.key,
                     Some(self.iv.bytes()),
                     associated_data,
                     plain_text,
                     &mut tag)
                .expect("there should be no error when performing encryption using aes_128_gcm")
    }
}

