use crypto::aead::AeadEncryptor;
use lz_fnv::Fnv128a;
use std::mem;
use protocol::Writable;

#[derive(Debug, Clone, Default)]
pub struct NullAeadEncryptor {}

impl AeadEncryptor for NullAeadEncryptor {
    fn encrypt(&mut self, associated_data: &[u8], plain_text: &[u8]) -> Vec<u8> {
        let mut hasher = Fnv128a::default();

        hasher.write(associated_data);
        hasher.write(plain_text);

        let hash = hasher.finish();

        let low = hash.low64();
        let high = hash.high64();

        let hash_length = mem::size_of::<u64>() + mem::size_of::<u32>();
        
        let mut result = Vec::with_capacity(plain_text.len() + hash_length);

        low.write_to_vec(&mut result);
        (high as u32).write_to_vec(&mut result);

        result.extend_from_slice(plain_text);

        result
    }
}

