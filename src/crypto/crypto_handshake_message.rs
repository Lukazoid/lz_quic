use errors::*;
use std::io::{Read, Write};
use tag::Tag;
use tag_value_map::TagValueMap;
use std::convert::TryInto;
use crypto::client_hello_message::ClientHelloMessage;
use crypto::rejection_message::RejectionMessage;
use crypto::server_configuration::ServerConfiguration;
use writable::Writable;
use readable::Readable;

#[derive(Debug, Clone)]
pub enum CryptoHandshakeMessage {
    Rejection(RejectionMessage),
    ClientHello(ClientHelloMessage),
    ServerConfiguration(ServerConfiguration),
}

impl Writable for CryptoHandshakeMessage {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {

        let (tag, tag_value_map) = match *self {
            CryptoHandshakeMessage::Rejection(ref rejection_message) => {
                (Tag::Rejection, TagValueMap::from(rejection_message))
            }
            CryptoHandshakeMessage::ClientHello(ref client_hello_message) => {
                (Tag::ClientHello, TagValueMap::from(client_hello_message))
            }
            CryptoHandshakeMessage::ServerConfiguration(ref server_configuration) => {
                (Tag::ServerConfiguration, TagValueMap::from(server_configuration))
            }
        };

        tag.write(writer)
            .chain_err(|| ErrorKind::UnableToWriteCryptoMessageTag(tag))?;

        (tag_value_map.len() as u16)
            .write(writer)
            .chain_err(|| ErrorKind::UnableToWriteTagValueMapLength)?;

        // Two bytes of padding
        let padding = [0; 2];
        writer.write_all(&padding)
            .chain_err(|| ErrorKind::UnableToWritePadding(padding.len()))?;

        tag_value_map.write(writer)?;

        Ok(())
    }
}

fn read_quic_tag_value_map<R: Read>(reader: &mut R) -> Result<TagValueMap> {
    let tag_value_count = u16::read(reader)
        .chain_err(|| ErrorKind::UnableToReadTagValueMapLength)?;

    // Ignore the two bytes of padding
    let mut padding = [0; 2];
    reader.read_exact(&mut padding)
        .chain_err(|| ErrorKind::UnableToReadPadding(padding.len()))?;

    TagValueMap::read(reader, tag_value_count as usize)
}

impl Readable for CryptoHandshakeMessage {
    fn read<R: Read>(reader: &mut R) -> Result<CryptoHandshakeMessage> {
        let tag = Tag::read(reader)
            .chain_err(|| ErrorKind::UnableToReadCryptoMessageTag)?;

        match tag {
            Tag::Rejection => {
                (&read_quic_tag_value_map(reader)?)
                    .try_into()
                    .chain_err(|| ErrorKind::UnableToReadCryptoRejectionMessage)
                    .map(CryptoHandshakeMessage::Rejection)
            }
            Tag::ClientHello => {
                (&read_quic_tag_value_map(reader)?)
                    .try_into()
                    .chain_err(|| ErrorKind::UnableToReadCryptoClientHelloMessage)
                    .map(CryptoHandshakeMessage::ClientHello)
            }
            Tag::ServerConfiguration => {
                (&read_quic_tag_value_map(reader)?)
                    .try_into()
                    .chain_err(|| ErrorKind::UnableToReadCryptoServerConfigurationMessage)
                    .map(CryptoHandshakeMessage::ServerConfiguration)
            }
            tag @ _ => bail!(ErrorKind::InvalidCryptoHandshakeMessage(tag)),
        }
    }
}