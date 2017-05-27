use errors::*;
use conv::TryFrom;
use handshake::{ServerConfigurationId, Tag, TagValueMap};
use crypto::key_exchange::KeyExchangeAlgorithm;

#[derive(Debug, Clone)]
pub struct ServerConfiguration {
    pub server_configuration_id: ServerConfigurationId,
    pub key_exchange_algorithms: Vec<KeyExchangeAlgorithm>,
}

impl ServerConfiguration {
    pub fn from_tag_value_map(tag_value_map: &TagValueMap) -> Result<Self> {
        let server_configuration_id = tag_value_map
            .get_required_value(Tag::ServerConfigurationId)?;
        let key_exchange_algorithms = tag_value_map
            .get_required_values(Tag::KeyExchangeAlgorithm)?;

        Ok(ServerConfiguration {
               server_configuration_id: server_configuration_id,
               key_exchange_algorithms: key_exchange_algorithms,
           })
    }

    pub fn to_tag_value_map(&self) -> TagValueMap {
        let mut tag_value_map = TagValueMap::default();

        tag_value_map.set_value(Tag::ServerConfigurationId, &self.server_configuration_id);

        tag_value_map.set_value(Tag::KeyExchangeAlgorithm, &self.key_exchange_algorithms);

        tag_value_map
    }
}
