use errors::*;
use quic_tag_value_map::QuicTagValueMap;
use quic_tag::QuicTag;
use std::convert::TryFrom;
use crypto::server_configuration_id::ServerConfigurationId;
use crypto::key_exchange_algorithm::KeyExchangeAlgorithm;

#[derive(Debug, Clone)]
pub struct ServerConfiguration {
    pub server_configuration_id: ServerConfigurationId,
    pub key_exchange_algorithms: Vec<KeyExchangeAlgorithm>,
}

impl<'a> TryFrom<&'a QuicTagValueMap> for ServerConfiguration {
    type Error = Error;

    fn try_from(value: &'a QuicTagValueMap) -> Result<Self> {
        let server_configuration_id = value.get_required_value(QuicTag::ServerConfigurationId)?;
        let key_exchange_algorithms = value.get_required_values(QuicTag::KeyExchangeAlgorithm)?;

        Ok(ServerConfiguration {
            server_configuration_id: server_configuration_id,
            key_exchange_algorithms: key_exchange_algorithms,
        })
    }
}

impl<'a> From<&'a ServerConfiguration> for QuicTagValueMap {
    fn from(value: &'a ServerConfiguration) -> Self {
        let mut quic_tag_value_map = QuicTagValueMap::default();

        quic_tag_value_map.set_value(QuicTag::ServerConfigurationId,
                                     &value.server_configuration_id);

        quic_tag_value_map.set_value(QuicTag::KeyExchangeAlgorithm,
                                     &value.key_exchange_algorithms);

        quic_tag_value_map
    }
}