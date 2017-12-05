use errors::*;
use std::io::{Read, Write};
use handshake::{ClientHelloMessage, RejectionMessage, ServerConfiguration, ServerHelloMessage,
                Tag, TagValueMap};
use protocol::{Readable, Writable};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HandshakeMessage {
    Rejection(RejectionMessage),
    ClientHello(ClientHelloMessage),
    ServerConfiguration(ServerConfiguration),
    ServerHello(ServerHelloMessage),
}

impl HandshakeMessage {
    fn write_message<W: Write>(
        writer: &mut W,
        tag: Tag,
        tag_value_map: &TagValueMap,
    ) -> Result<()> {
        tag.write(writer)
            .chain_err(|| ErrorKind::FailedToWriteCryptoMessageTag(tag))?;

        (tag_value_map.len() as u16)
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteTagValueMapLength)?;

        // Two bytes of padding
        let padding = [0; 2];
        writer
            .write_all(&padding)
            .chain_err(|| ErrorKind::FailedToWritePadding(padding.len()))?;

        tag_value_map.write(writer)?;

        Ok(())
    }
    
    pub fn tag(&self) -> Tag {
        match *self {
            HandshakeMessage::Rejection(_) => {
                Tag::Rejection
            }
            HandshakeMessage::ClientHello(_) => {
                Tag::ClientHello
            }
            HandshakeMessage::ServerConfiguration(_) => {
                Tag::ServerConfiguration
            }
            HandshakeMessage::ServerHello(_) => {
                Tag::ServerHello
            }
        }
    }

    pub fn write_rejection<W: Write>(
        writer: &mut W,
        rejection_message: &RejectionMessage,
    ) -> Result<()> {
        Self::write_message(
            writer,
            Tag::Rejection,
            &rejection_message.to_tag_value_map(),
        )
    }

    pub fn write_client_hello<W: Write>(
        writer: &mut W,
        client_hello_message: &ClientHelloMessage,
    ) -> Result<()> {
        Self::write_message(
            writer,
            Tag::ClientHello,
            &client_hello_message.to_tag_value_map(),
        )
    }

    pub fn write_server_configuration<W: Write>(
        writer: &mut W,
        server_configuration: &ServerConfiguration,
    ) -> Result<()> {
        Self::write_message(
            writer,
            Tag::ServerConfiguration,
            &server_configuration.to_tag_value_map(),
        )
    }

    pub fn write_server_hello<W: Write>(
        writer: &mut W,
        server_hello_message: &ServerHelloMessage,
    ) -> Result<()> {
        Self::write_message(
            writer,
            Tag::ServerHello,
            &server_hello_message.to_tag_value_map(),
        )
    }
}

impl Writable for HandshakeMessage {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing handshake message {:?}", self);

        match *self {
            HandshakeMessage::Rejection(ref rejection_message) => {
                Self::write_rejection(writer, rejection_message)?;
            }
            HandshakeMessage::ClientHello(ref client_hello_message) => {
                Self::write_client_hello(writer, client_hello_message)?;
            }
            HandshakeMessage::ServerConfiguration(ref server_configuration) => {
                Self::write_server_configuration(writer, server_configuration)?;
            }
            HandshakeMessage::ServerHello(ref server_hello_message) => {
                Self::write_server_hello(writer, server_hello_message)?;
            }
        }

        debug!("written handshake message {:?}", self);

        Ok(())
    }
}

fn read_quic_tag_value_map<R: Read>(reader: &mut R) -> Result<TagValueMap> {
    trace!("reading how many tag values there are");

    let tag_value_count = u16::read(reader)
        .chain_err(|| ErrorKind::FailedToReadTagValueMapLength)?;

    trace!("determined there are {} tag values", tag_value_count);

    // Ignore the two bytes of padding
    let mut padding = [0; 2];
    reader
        .read_exact(&mut padding)
        .chain_err(|| ErrorKind::FailedToReadPadding(padding.len()))?;

    TagValueMap::read(reader, tag_value_count as usize)
}

impl Readable for HandshakeMessage {
    fn read<R: Read>(reader: &mut R) -> Result<HandshakeMessage> {
        trace!("reading handshake message");

        let tag = Tag::read(reader)
            .chain_err(|| ErrorKind::FailedToReadCryptoMessageTag)?;

        let handshake_message = match tag {
            Tag::Rejection => {
                let tag_value_map = read_quic_tag_value_map(reader)?;

                RejectionMessage::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::FailedToReadCryptoRejectionMessage)
                    .map(HandshakeMessage::Rejection)?
            }
            Tag::ClientHello => {
                let tag_value_map = read_quic_tag_value_map(reader)?;

                ClientHelloMessage::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::FailedToReadCryptoClientHelloMessage)
                    .map(HandshakeMessage::ClientHello)?
            }
            Tag::ServerConfiguration => {
                let tag_value_map = read_quic_tag_value_map(reader)?;

                ServerConfiguration::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::FailedToReadCryptoServerConfigurationMessage)
                    .map(HandshakeMessage::ServerConfiguration)?
            }
            Tag::ServerHello => {
                let tag_value_map = read_quic_tag_value_map(reader)?;

                ServerHelloMessage::from_tag_value_map(&tag_value_map)
                    .chain_err(|| ErrorKind::FailedToReadCryptoServerHelloMessage)
                    .map(HandshakeMessage::ServerHello)?
            }
            tag @ _ => bail!(ErrorKind::InvalidHandshakeMessage(tag)),
        };

        debug!("read handshake message {:?}", handshake_message);

        Ok(handshake_message)
    }
}
