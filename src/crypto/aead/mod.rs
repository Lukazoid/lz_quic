mod aead_decryptor;
pub use self::aead_decryptor::AeadDecryptor;

mod aead_encryptor;
pub use self::aead_encryptor::AeadEncryptor;

mod null_aead_decryptor;
pub use self::null_aead_decryptor::NullAeadDecryptor;

mod null_aead_encryptor;
pub use self::null_aead_encryptor::NullAeadEncryptor;

use packets::PacketNumber;
use crypto::InitializationVector;
use protocol::Writable;

pub fn make_nonce(iv: &InitializationVector, packet_number: PacketNumber) -> Vec<u8> {
    let mut result = Vec::new();

    result.extend_from_slice(iv.bytes());
    packet_number.write_to_vec(&mut result);

    result
}

mod aes_gcm_encryptor;
pub use self::aes_gcm_encryptor::AesGcmEncryptor;

mod aes_gcm_decryptor;
pub use self::aes_gcm_decryptor::AesGcmDecryptor;