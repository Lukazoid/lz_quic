use errors::*;
use handshake::{ServerConfiguration, HandshakeMessage, Tag, TagValueMap, SourceAddressToken, ServerNonce};
use conv::TryFrom;
use crypto::PublicKey;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServerHelloMessage {
    pub server_configuration: Option<ServerConfiguration>,
    pub source_address_token: Option<SourceAddressToken>,
    pub server_nonce: Option<ServerNonce>,
    pub seconds_to_live: u64,
    pub public_key: PublicKey,
    pub compressed_certificate_chain: Option<Vec<u8>>,
}

impl ServerHelloMessage {
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
        let public_key = tag_value_map.get_required_value(Tag::PublicValue)?;
        let compressed_certificate_chain: Option<Vec<u8>> = tag_value_map.get_optional_value(Tag::CertificateChain)?;

        Ok(Self {
            server_configuration: server_configuration,
            source_address_token: source_address_token,
            server_nonce: server_nonce,
            seconds_to_live: seconds_to_live,
            public_key: public_key,
            compressed_certificate_chain: compressed_certificate_chain,
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

        tag_value_map.set_value(Tag::PublicValue, self.public_key.bytes());

        if let Some(ref compressed_certificate_chain) = self.compressed_certificate_chain {
            tag_value_map.set_value(Tag::CertificateChain, compressed_certificate_chain);
        }
    

        tag_value_map
    }
}