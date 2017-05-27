use errors::*;
use std::io::{Read, Write};
use handshake::{Tag, TagValueMap, ClientHelloMessage, RejectionMessage, ServerConfiguration};
use protocol::{Readable, Writable};

#[derive(Debug, Clone)]
pub enum HandshakeMessage {
    Rejection(RejectionMessage),
    ClientHello(ClientHelloMessage),
    ServerConfiguration(ServerConfiguration),
}

impl HandshakeMessage {
    fn write_message<W: Write>(writer: &mut W, tag: Tag, tag_value_map: &TagValueMap) -> Result<()> {
        tag.write(writer)
            .chain_err(|| ErrorKind::UnableToWriteCryptoMessageTag(tag))?;

        (tag_value_map.len() as u16)
            .write(writer)
            .chain_err(|| ErrorKind::UnableToWriteTagValueMapLength)?;

        // Two bytes of padding
        let padding = [0; 2];
        writer
            .write_all(&padding)
            .chain_err(|| ErrorKind::UnableToWritePadding(padding.len()))?;

        tag_value_map.write(writer)?;

        Ok(())
    }

    pub fn write_rejection<W: Write>(writer: &mut W, rejection_message: &RejectionMessage) -> Result<()>{
        Self::write_message(writer, Tag::Rejection, &rejection_message.to_tag_value_map())
    }
    
    pub fn write_client_hello<W: Write>(writer: &mut W, client_hello_message: &ClientHelloMessage) -> Result<()>{
        Self::write_message(writer, Tag::ClientHello, &client_hello_message.to_tag_value_map())
    }

    pub fn write_server_configuration<W: Write>(writer: &mut W, server_configuration: &ServerConfiguration) -> Result<()>{
        Self::write_message(writer, Tag::ServerConfiguration, &server_configuration.to_tag_value_map())
    }
}

impl Writable for HandshakeMessage {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            HandshakeMessage::Rejection(ref rejection_message) => {
                Self::write_rejection(writer, rejection_message)
            }
            HandshakeMessage::ClientHello(ref client_hello_message) => {
                Self::write_client_hello(writer, client_hello_message)
            }
            HandshakeMessage::ServerConfiguration(ref server_configuration) => {
                Self::write_server_configuration(writer, server_configuration)
            }
        }
    }
}

fn read_quic_tag_value_map<R: Read>(reader: &mut R) -> Result<TagValueMap> {
    let tag_value_count = u16::read(reader)
        .chain_err(|| ErrorKind::UnableToReadTagValueMapLength)?;

    // Ignore the two bytes of padding
    let mut padding = [0; 2];
    reader
        .read_exact(&mut padding)
        .chain_err(|| ErrorKind::UnableToReadPadding(padding.len()))?;

    TagValueMap::read(reader, tag_value_count as usize)
}

impl Readable for HandshakeMessage {
    fn read<R: Read>(reader: &mut R) -> Result<HandshakeMessage> {
        let tag = Tag::read(reader)
            .chain_err(|| ErrorKind::UnableToReadCryptoMessageTag)?;

        match tag {
            Tag::Rejection => {
                let tag_value_map = read_quic_tag_value_map(reader)?;

                RejectionMessage::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::UnableToReadCryptoRejectionMessage)
                    .map(HandshakeMessage::Rejection)
            }
            Tag::ClientHello => {
                let tag_value_map = read_quic_tag_value_map(reader)?;
                ClientHelloMessage::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::UnableToReadCryptoClientHelloMessage)
                    .map(HandshakeMessage::ClientHello)
            }
            Tag::ServerConfiguration => {
                let tag_value_map = read_quic_tag_value_map(reader)?;
                ServerConfiguration::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::UnableToReadCryptoServerConfigurationMessage)
                    .map(HandshakeMessage::ServerConfiguration)
            }
            tag @ _ => bail!(ErrorKind::InvalidHandshakeMessage(tag)),
        }
    }
}

