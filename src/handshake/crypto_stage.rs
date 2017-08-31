use errors::*;
use crypto::aead::{AeadDecryptor, AeadEncryptor, AesGcmDecryptor, AesGcmEncryptor,
                   NullAeadDecryptor, NullAeadEncryptor};
use crypto::key_derivation::DerivedKeys;
use packets::PacketNumber;
use protocol::EncryptionLevel;
use std::mem;

#[derive(Debug)]
struct AeadPair {
    encryptor: AesGcmEncryptor,
    decryptor: AesGcmDecryptor,
}

impl AeadPair {
    pub fn new(derived_keys: DerivedKeys) -> Self {
        AeadPair {
            encryptor: AesGcmEncryptor::new(derived_keys.local_key, derived_keys.local_iv),
            decryptor: AesGcmDecryptor::new(derived_keys.remote_key, derived_keys.remote_iv),
        }
    }
}

#[derive(Debug)]
pub enum CryptoStage {
    Unencrypted,
    NonForwardSecure { aead: AeadPair },
    ForwardSecure {
        non_forward_secure: AeadPair,
        forward_secure: AeadPair,
    },
}

impl CryptoStage {
    
    pub fn upgrade_to_non_forward_secure(&mut self, derived_keys: DerivedKeys) -> Result<()> {
        // temporarily take ownership of self
        match mem::replace(self, CryptoStage::Unencrypted) {
            CryptoStage::Unencrypted => {
                *self = CryptoStage::NonForwardSecure{ aead: AeadPair::new(derived_keys) };
                
                Ok(())
            },
            original @ CryptoStage::NonForwardSecure { .. } | 
                original @ CryptoStage::ForwardSecure { .. } => {
                *self = original;
                
                bail!(ErrorKind::UnableToUpgradeCryptoAsItIsAlreadyAtNonForwardSecureStage);
            },
        }
    }

    pub fn upgrade_to_forward_secure(&mut self, derived_keys: DerivedKeys) -> Result<()> {
        // temporarily take ownership of self
        match mem::replace(self, CryptoStage::Unencrypted) {
            original @ CryptoStage::Unencrypted => {
                *self = original;

                bail!(ErrorKind::UnableToUpgradeCryptoFromUnencryptedToForwardSecureStage);
            },
            CryptoStage::NonForwardSecure { aead: aead } => {
                *self = CryptoStage::ForwardSecure{
                    non_forward_secure: aead,
                    forward_secure: AeadPair::new(derived_keys)
                };
                
                Ok(())
            },
            original @ CryptoStage::ForwardSecure { .. } => {
                *self = original;

                bail!(ErrorKind::UnableToUpgradeCryptoAsItIsAlreadyAtForwardSecureStage);
            }
        }
    }

    pub fn encryption_level(&self) -> EncryptionLevel {
        match *self {
            CryptoStage::Unencrypted => EncryptionLevel::Unencrypted,
            CryptoStage::NonForwardSecure { .. } => EncryptionLevel::NonForwardSecure,
            CryptoStage::ForwardSecure { .. } => EncryptionLevel::ForwardSecure,
        }
    }
    pub fn decrypt(
        &self,
        associated_data: &[u8],
        raw: &[u8],
        packet_number: PacketNumber,
    ) -> Result<(EncryptionLevel, Vec<u8>)> {
        match *self {
            CryptoStage::Unencrypted => {
                let decryptor = NullAeadDecryptor::default();
                let decrypted = decryptor.decrypt(associated_data, raw, packet_number)?;

                Ok((EncryptionLevel::Unencrypted, decrypted))
            }
            CryptoStage::NonForwardSecure { aead: ref aead, .. } => {
                let decrypted = aead.decryptor.decrypt(associated_data, raw, packet_number)?;

                Ok((EncryptionLevel::NonForwardSecure, decrypted))
            }
            CryptoStage::ForwardSecure {
                forward_secure: ref aead,
                ..
            } => {
                let decrypted = aead.decryptor.decrypt(associated_data, raw, packet_number)?;

                Ok((EncryptionLevel::ForwardSecure, decrypted))
            }
        }
    }

    fn unencrypted_encryptor(&self) -> NullAeadEncryptor {
        NullAeadEncryptor::default()
    }

    fn non_forward_secure_encryptor(&self) -> Option<&AesGcmEncryptor> {
        match *self {
            CryptoStage::NonForwardSecure { aead: ref aead, .. } |
            CryptoStage::ForwardSecure { non_forward_secure: ref aead, .. } => Some(&aead.encryptor),
            _ => None,
        }
    }

    fn forward_secure_encryptor(&self) -> Option<&AesGcmEncryptor> {
        match *self {
            CryptoStage::ForwardSecure {
                forward_secure: ref aead,
                ..
            } => Some(&aead.encryptor),
            _ => None,
        }
    }

    pub fn encrypt_at_level(
        &self,
        associated_data: &[u8],
        raw: &[u8],
        packet_number: PacketNumber,
        encryption_level: EncryptionLevel,
    ) -> Result<Vec<u8>> {
        match encryption_level {
            EncryptionLevel::Unencrypted => {
                let encryptor = self.unencrypted_encryptor();

                encryptor.encrypt(associated_data, raw, packet_number)
            }
            EncryptionLevel::NonForwardSecure => {
                let encryptor = self.non_forward_secure_encryptor()
                    .ok_or_else(|| Error::from(ErrorKind::NoNonForwardSecureAead))?;

                encryptor.encrypt(associated_data, raw, packet_number)
            }
            EncryptionLevel::ForwardSecure => {
                let encryptor = self.forward_secure_encryptor()
                    .ok_or_else(|| Error::from(ErrorKind::NoForwardSecureAead))?;

                encryptor.encrypt(associated_data, raw, packet_number)
            }
        }
    }

    pub fn encrypt(
        &self,
        associated_data: &[u8],
        raw: &[u8],
        packet_number: PacketNumber,
    ) -> Result<(EncryptionLevel, Vec<u8>)> {
        match *self {
            CryptoStage::Unencrypted => {
                let encryptor = NullAeadEncryptor::default();
                let encrypted = encryptor.encrypt(associated_data, raw, packet_number)?;

                Ok((EncryptionLevel::Unencrypted, encrypted))
            }
            CryptoStage::NonForwardSecure { aead: ref aead, .. } => {
                let encrypted = aead.encryptor.encrypt(associated_data, raw, packet_number)?;

                Ok((EncryptionLevel::NonForwardSecure, encrypted))
            }
            CryptoStage::ForwardSecure {
                forward_secure: ref aead,
                ..
            } => {
                let encrypted = aead.encryptor.encrypt(associated_data, raw, packet_number)?;

                Ok((EncryptionLevel::ForwardSecure, encrypted))
            }
        }
    }
}
