use errors::*;

pub trait AeadDecryptor {
    fn decrypt(&mut self, associated_data: &[u8], cipher_text: &[u8]) -> Result<Vec<u8>>;
}