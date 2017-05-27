use errors::*;
use handshake::{ServerConfiguration, HandshakeMessage, Tag, TagValueMap};
use conv::TryFrom;

#[derive(Debug, Clone)]
pub struct RejectionMessage {
    server_configuration: Option<ServerConfiguration>,
    source_address_token: Option<Vec<u8>>,
    server_nonce: Option<Vec<u8>>,
    seconds_to_live: u64,
}

impl RejectionMessage {
    pub fn from_tag_value_map(tag_value_map: &TagValueMap) -> Result<Self> {
        let server_configuration = if let Some(server_configuration_handshake_message) =
                                          tag_value_map.get_optional_value(Tag::ServerConfiguration)? {
            if let HandshakeMessage::ServerConfiguration(server_configuration) =
                   server_configuration_handshake_message {
                Some(server_configuration)
            } else {
                bail!(ErrorKind::InvalidTagValue(Tag::ServerConfiguration));
            }
        } else {
            None
        };

        let source_address_token = tag_value_map.get_optional_value(Tag::SourceAddressToken)?;
        let server_nonce = tag_value_map.get_optional_value(Tag::ServerNonce)?;
        let seconds_to_live = tag_value_map.get_required_value(Tag::ServerConfigurationTimeToLive)?;

        Ok(Self {
            server_configuration: server_configuration,
            source_address_token: source_address_token,
            server_nonce: server_nonce,
            seconds_to_live: seconds_to_live,
        })
    }

    pub fn to_tag_value_map(&self) -> TagValueMap {
        let mut tag_value_map = TagValueMap::default();

        if let Some(ref server_configuration) = self.server_configuration {
            let server_configuration_message =
                HandshakeMessage::ServerConfiguration(server_configuration.clone());

            tag_value_map.set_value(Tag::ServerConfiguration,
                                         &server_configuration_message);
        }

        if let Some(ref source_address_token) = self.source_address_token {
            tag_value_map.set_value(Tag::SourceAddressToken, source_address_token);
        }

        if let Some(ref server_nonce) = self.server_nonce {
            tag_value_map.set_value(Tag::ServerNonce, server_nonce);
        }

        tag_value_map.set_value(Tag::ServerConfigurationTimeToLive,
                                     &self.seconds_to_live);

        tag_value_map
    }
}