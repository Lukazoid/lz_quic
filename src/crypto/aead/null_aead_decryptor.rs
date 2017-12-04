use errors::*;
use crypto::aead::AeadDecryptor;
use lz_fnv::{Fnv1a, FnvHasher};
use std::mem;
use protocol::Readable;
use std::io::Cursor;
use packets::PacketNumber;
use extprim::u128::u128;

#[derive(Debug, Clone, Default)]
pub struct NullAeadDecryptor;

impl AeadDecryptor for NullAeadDecryptor {
    fn decrypt(
        &self,
        associated_data: &[u8],
        cipher_text: &[u8],
        packet_number: PacketNumber,
    ) -> Result<Vec<u8>> {
        trace!("decrypting data");

        let hash_length = mem::size_of::<u64>() + mem::size_of::<u32>();
        let cipher_text_length = cipher_text.len();

        if cipher_text_length < hash_length {
            bail!(ErrorKind::CipherTextTooShort(
                cipher_text_length,
                hash_length
            ));
        }

        let mut hasher = Fnv1a::<u128>::default();

        let plain_text = &cipher_text[hash_length..];

        hasher.write(associated_data);
        hasher.write(plain_text);
        let test_hash = hasher.finish();

        let test_low = test_hash.low64();
        let test_high = test_hash.high64();

        let mut cipher_cursor = Cursor::new(cipher_text);

        let actual_low = u64::read(&mut cipher_cursor)?;
        let actual_high = u32::read(&mut cipher_cursor)?;

        if (test_high as u32) != actual_high || test_low != actual_low {
            bail!(ErrorKind::FailedToAuthenticateReceivedData);
        }

        debug!("decrypted data");

        Ok(plain_text.to_vec())
    }
}
