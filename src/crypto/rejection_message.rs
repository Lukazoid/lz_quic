use errors::*;
use crypto::crypto_handshake_message::CryptoHandshakeMessage;
use crypto::server_configuration::ServerConfiguration;
use conv::TryFrom;
use tag_value_map::TagValueMap;
use tag::Tag;

#[derive(Debug, Clone)]
pub struct RejectionMessage {
    server_configuration: Option<ServerConfiguration>,
    source_address_token: Option<Vec<u8>>,
    server_nonce: Option<Vec<u8>>,
    seconds_to_live: u64,
}

impl<'a> TryFrom<&'a TagValueMap> for RejectionMessage {
    type Err = Error;

    fn try_from(value: &'a TagValueMap) -> Result<Self> {

        let server_configuration = if let Some(server_configuration_handshake_message) =
                                          value.get_optional_value(Tag::ServerConfiguration)? {
            if let CryptoHandshakeMessage::ServerConfiguration(server_configuration) =
                   server_configuration_handshake_message {
                Some(server_configuration)
            } else {
                bail!(ErrorKind::InvalidTagValue(Tag::ServerConfiguration));
            }
        } else {
            None
        };

        let source_address_token = value.get_optional_value(Tag::SourceAddressToken)?;
        let server_nonce = value.get_optional_value(Tag::ServerNonce)?;
        let seconds_to_live = value.get_required_value(Tag::ServerConfigurationTimeToLive)?;

        Ok(Self {
            server_configuration: server_configuration,
            source_address_token: source_address_token,
            server_nonce: server_nonce,
            seconds_to_live: seconds_to_live,
        })
    }
}

impl<'a> From<&'a RejectionMessage> for TagValueMap {
    fn from(value: &'a RejectionMessage) -> Self {
        let mut tag_value_map = TagValueMap::default();

        if let Some(ref server_configuration) = value.server_configuration {
            let server_configuration_message =
                CryptoHandshakeMessage::ServerConfiguration(server_configuration.clone());

            tag_value_map.set_value(Tag::ServerConfiguration,
                                         &server_configuration_message);
        }

        if let Some(ref source_address_token) = value.source_address_token {
            tag_value_map.set_value(Tag::SourceAddressToken, source_address_token);
        }

        if let Some(ref server_nonce) = value.server_nonce {
            tag_value_map.set_value(Tag::ServerNonce, server_nonce);
        }

        tag_value_map.set_value(Tag::ServerConfigurationTimeToLive,
                                     &value.seconds_to_live);

        tag_value_map
    }
}