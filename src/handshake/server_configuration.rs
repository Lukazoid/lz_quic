use errors::*;
use conv::TryFrom;
use handshake::{ServerConfigurationId, Tag, TagValueMap};
use crypto::key_exchange::{Curve25519KeyExchange, KeyExchange, KeyExchangeAlgorithm,
                           P256KeyExchange, KEY_EXCHANGE_ALGORITHM_COUNT};
use crypto::aead::{AEAD_ALGORITHM_COUNT, AeadAlgorithm};
use crypto::{PublicKey, SharedKey};
use std::io::{Read, Write};
use protocol::{Readable, Version, Writable};
use time::{self, Timespec};
use smallvec::SmallVec;
use primitives::U24;
use std::borrow::Borrow;

struct PublicKeyEntry<T:Borrow<PublicKey>>(T);

impl Readable for PublicKeyEntry<PublicKey> {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let length = U24::read(reader)?;
        let length: u32 = length.into();

        let mut bytes = Vec::with_capacity(length as usize);
        reader.read_to_end(&mut bytes)
            .chain_err(|| ErrorKind::FailedToReadPublicKeyBytes)?;
            
        Ok(PublicKeyEntry(bytes.as_slice().into()))
    }
}

impl<T:Borrow<PublicKey>> Writable for PublicKeyEntry<T> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let bytes = self.0.borrow().bytes();

        let length = bytes.len() as u32;
        let length = U24::try_from(length)
            .chain_err(|| ErrorKind::PublicKeyBytesTooLongForU24)?;

        length.write(writer)?;
        writer.write_all(bytes)
            .chain_err(||ErrorKind::FailedToWritePublicKeyBytes)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerConfiguration {
    pub server_configuration_id: ServerConfigurationId,
    pub key_exchange_algorithms: SmallVec<[KeyExchangeAlgorithm; KEY_EXCHANGE_ALGORITHM_COUNT]>,
    pub aead_algorithms: SmallVec<[AeadAlgorithm; AEAD_ALGORITHM_COUNT]>,
    pub public_keys: SmallVec<[PublicKey; AEAD_ALGORITHM_COUNT]>,
    pub orbit: u64,
    /// The expiry time in seconds.
    pub expiry_time: u64,
    pub versions: SmallVec<[Version;8]>,
    pub shared_key: SharedKey,
}

fn calculate_shared_key(
    key_exchange_algorithm: KeyExchangeAlgorithm,
    public_key: &PublicKey,
) -> Result<SharedKey> {
    let shared_key = match key_exchange_algorithm {
        KeyExchangeAlgorithm::Curve25519 => {
            let key_exchange = Curve25519KeyExchange::new()?;
            key_exchange.calculate_shared_key(public_key)?
        }
        KeyExchangeAlgorithm::P256 => {
            let key_exchange = P256KeyExchange::new()?;
            key_exchange.calculate_shared_key(public_key)?
        }
        _ => {
            bail!(ErrorKind::ASupportedKeyExchangeAlgorithmMustBeSpecified);
        }
    };

    Ok(shared_key)
}

impl ServerConfiguration {
    pub fn is_expired(&self) -> bool {
        let now = time::now_utc().to_timespec().sec;
        assert!(now > 0, "we cannot possibly be before the unix epoch");

        (now as u64) >= self.expiry_time
    }

    pub fn from_tag_value_map(tag_value_map: &TagValueMap) -> Result<Self> {
        let server_configuration_id = tag_value_map
            .get_required_value(Tag::ServerConfigurationId)?;
        let key_exchange_algorithms: SmallVec<[KeyExchangeAlgorithm; KEY_EXCHANGE_ALGORITHM_COUNT]> = tag_value_map
            .get_required_values(Tag::KeyExchangeAlgorithm)?;
        let aead_algorithms = tag_value_map
            .get_required_values(Tag::AuthenticatedEncryptionAlgorithm)?;
        let public_keys: SmallVec<[PublicKeyEntry<PublicKey>; AEAD_ALGORITHM_COUNT]> = tag_value_map
            .get_required_values(Tag::PublicValue)?;

        if public_keys.len() == 0 {
            bail!(ErrorKind::APublicKeyMustBeSpecified);
        }

        if key_exchange_algorithms.len() != public_keys.len() {
            bail!(ErrorKind::KeyExchangeAlgorithmAndPublicKeyCountsMustMatch);
        }

        let public_keys: SmallVec<[PublicKey; AEAD_ALGORITHM_COUNT]>= public_keys.into_iter().map(|x|x.0).collect();

        let shared_key = {
            // Need to find the first key exchange algorithm we support
            let (public_key_index, key_exchange_algorithm, public_key): (usize, KeyExchangeAlgorithm, &PublicKey) = 
            key_exchange_algorithms.iter()
                .map(|x| *x)
                .zip(public_keys.iter())
                .enumerate()
                .filter_map(|(index, (key_exchange_algorithm, public_key))| {
                    if key_exchange_algorithm.is_supported() {
                        Some((index, key_exchange_algorithm, public_key))
                    } else {
                        None
                    }
                })
                .next()
                .ok_or_else(|| Error::from(ErrorKind::ASupportedKeyExchangeAlgorithmMustBeSpecified))?;

            calculate_shared_key(key_exchange_algorithm, public_key)?
        };

        let orbit = tag_value_map.get_required_value(Tag::Orbit)?;

        let expiry_time = tag_value_map
            .get_required_value(Tag::ServerConfigurationExpiry)?;

        let versions = tag_value_map.get_required_values(Tag::Version)?;

        Ok(ServerConfiguration {
            server_configuration_id: server_configuration_id,
            key_exchange_algorithms: key_exchange_algorithms,
            aead_algorithms: aead_algorithms,
            public_keys: public_keys,
            orbit: orbit,
            expiry_time: expiry_time,
            versions: versions,
            shared_key: shared_key,
        })
    }

    pub fn to_tag_value_map(&self) -> TagValueMap {
        let mut tag_value_map = TagValueMap::default();

        tag_value_map.set_value(Tag::ServerConfigurationId, &self.server_configuration_id);
        tag_value_map.set_value(Tag::KeyExchangeAlgorithm, &self.key_exchange_algorithms);
        tag_value_map.set_value(Tag::AuthenticatedEncryptionAlgorithm, &self.aead_algorithms);

        let public_key_entries: SmallVec<[_; AEAD_ALGORITHM_COUNT]> = self.public_keys.iter()
            .map(|pk|PublicKeyEntry(pk))
            .collect();

        tag_value_map.set_value(Tag::PublicValue, &public_key_entries);
        tag_value_map.set_value(Tag::Orbit, &self.orbit);
        tag_value_map.set_value(Tag::ServerConfigurationExpiry, &self.expiry_time);
        tag_value_map.set_value(Tag::Version, &self.versions);

        tag_value_map
    }
}
