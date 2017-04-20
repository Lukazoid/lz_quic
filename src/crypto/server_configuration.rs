use errors::*;
use tag_value_map::TagValueMap;
use tag::Tag;
use std::convert::TryFrom;
use crypto::server_configuration_id::ServerConfigurationId;
use crypto::key_exchange_algorithm::KeyExchangeAlgorithm;

#[derive(Debug, Clone)]
pub struct ServerConfiguration {
    pub server_configuration_id: ServerConfigurationId,
    pub key_exchange_algorithms: Vec<KeyExchangeAlgorithm>,
}

impl<'a> TryFrom<&'a TagValueMap> for ServerConfiguration {
    type Error = Error;

    fn try_from(value: &'a TagValueMap) -> Result<Self> {
        let server_configuration_id = value.get_required_value(Tag::ServerConfigurationId)?;
        let key_exchange_algorithms = value.get_required_values(Tag::KeyExchangeAlgorithm)?;

        Ok(ServerConfiguration {
            server_configuration_id: server_configuration_id,
            key_exchange_algorithms: key_exchange_algorithms,
        })
    }
}

impl<'a> From<&'a ServerConfiguration> for TagValueMap {
    fn from(value: &'a ServerConfiguration) -> Self {
        let mut tag_value_map = TagValueMap::default();

        tag_value_map.set_value(Tag::ServerConfigurationId,
                                     &value.server_configuration_id);

        tag_value_map.set_value(Tag::KeyExchangeAlgorithm,
                                     &value.key_exchange_algorithms);

        tag_value_map
    }
}