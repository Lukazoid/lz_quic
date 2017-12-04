use errors::*;
use crypto::aead::AeadEncryptor;
use lz_fnv::{Fnv1a, FnvHasher};
use std::mem;
use protocol::Writable;
use packets::PacketNumber;
use extprim::u128::u128;

#[derive(Debug, Clone, Default)]
pub struct NullAeadEncryptor;

impl AeadEncryptor for NullAeadEncryptor {
    fn encrypt(
        &self,
        associated_data: &[u8],
        plain_text: &[u8],
        packet_number: PacketNumber,
    ) -> Result<Vec<u8>> {
        trace!("encrypting data");
        let mut hasher = Fnv1a::<u128>::default();

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
        debug!("encrypted data");
        
        Ok(result)
    }
}
