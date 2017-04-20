use errors::*;
use crypto::crypto_handshake_message::CryptoHandshakeMessage;
use crypto::server_configuration::ServerConfiguration;
use std::convert::TryFrom;
use quic_tag_value_map::QuicTagValueMap;
use quic_tag::QuicTag;

#[derive(Debug, Clone)]
pub struct RejectionMessage {
    server_configuration: Option<ServerConfiguration>,
    source_address_token: Option<Vec<u8>>,
    server_nonce: Option<Vec<u8>>,
    seconds_to_live: u64,
}

impl<'a> TryFrom<&'a QuicTagValueMap> for RejectionMessage {
    type Error = Error;

    fn try_from(value: &'a QuicTagValueMap) -> Result<Self> {

        let server_configuration = if let Some(server_configuration_handshake_message) =
                                          value.get_optional_value(QuicTag::ServerConfiguration)? {
            if let CryptoHandshakeMessage::ServerConfiguration(server_configuration) =
                   server_configuration_handshake_message {
                Some(server_configuration)
            } else {
                bail!(ErrorKind::InvalidQuicTagValue(QuicTag::ServerConfiguration));
            }
        } else {
            None
        };

        let source_address_token = value.get_optional_value(QuicTag::SourceAddressToken)?;
        let server_nonce = value.get_optional_value(QuicTag::ServerNonce)?;
        let seconds_to_live = value.get_required_value(QuicTag::ServerConfigurationTimeToLive)?;

        Ok(Self {
            server_configuration: server_configuration,
            source_address_token: source_address_token,
            server_nonce: server_nonce,
            seconds_to_live: seconds_to_live,
        })
    }
}

impl<'a> From<&'a RejectionMessage> for QuicTagValueMap {
    fn from(value: &'a RejectionMessage) -> Self {
        let mut quic_tag_value_map = QuicTagValueMap::default();

        if let Some(ref server_configuration) = value.server_configuration {
            let server_configuration_message =
                CryptoHandshakeMessage::ServerConfiguration(server_configuration.clone());

            quic_tag_value_map.set_value(QuicTag::ServerConfiguration,
                                         &server_configuration_message);
        }

        if let Some(ref source_address_token) = value.source_address_token {
            quic_tag_value_map.set_value(QuicTag::SourceAddressToken, source_address_token);
        }

        if let Some(ref server_nonce) = value.server_nonce {
            quic_tag_value_map.set_value(QuicTag::ServerNonce, server_nonce);
        }

        quic_tag_value_map.set_value(QuicTag::ServerConfigurationTimeToLive,
                                     &value.seconds_to_live);

        quic_tag_value_map
    }
}