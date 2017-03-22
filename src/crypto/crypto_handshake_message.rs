use errors::*;
use std::io::{Read, Write};
use quic_tag::QuicTag;
use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use quic_tag_value_map::QuicTagValueMap;
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

        let (quic_tag, quic_tag_value_map) = match *self {
            CryptoHandshakeMessage::Rejection(ref rejection_message) => {
                (QuicTag::Rejection, QuicTagValueMap::from(rejection_message))
            }
            CryptoHandshakeMessage::ClientHello(ref client_hello_message) => {
                (QuicTag::ClientHello, QuicTagValueMap::from(client_hello_message))
            }
            CryptoHandshakeMessage::ServerConfiguration(ref server_configuration) => {
                (QuicTag::ServerConfiguration, QuicTagValueMap::from(server_configuration))
            }
        };

        quic_tag.write(writer)
            .chain_err(|| ErrorKind::UnableToWriteCryptoMessageQuicTag(quic_tag))?;

        writer.write_u16::<LittleEndian>(quic_tag_value_map.len() as u16)
            .chain_err(|| ErrorKind::UnableToWriteQuicTagValueMapLength)?;

        // Two bytes of padding
        let padding = [0; 2];
        writer.write_all(&padding)
            .chain_err(|| ErrorKind::UnableToWritePadding(padding.len()))?;

        quic_tag_value_map.write(writer)?;

        Ok(())
    }
}

fn read_quic_tag_value_map<R: Read>(reader: &mut R) -> Result<QuicTagValueMap> {
    let tag_value_count = reader.read_u16::<LittleEndian>()
        .chain_err(|| ErrorKind::UnableToReadQuicTagValueMapLength)?;

    // Ignore the two bytes of padding
    let mut padding = [0; 2];
    reader.read_exact(&mut padding)
        .chain_err(|| ErrorKind::UnableToReadPadding(padding.len()))?;

    QuicTagValueMap::read(reader, tag_value_count as usize)
}

impl Readable for CryptoHandshakeMessage {
    fn read<R: Read>(reader: &mut R) -> Result<CryptoHandshakeMessage> {
        let quic_tag = QuicTag::read(reader)
            .chain_err(|| ErrorKind::UnableToReadCryptoMessageQuicTag)?;

        match quic_tag {
            QuicTag::Rejection => {
                (&read_quic_tag_value_map(reader)?)
                    .try_into()
                    .chain_err(|| ErrorKind::UnableToReadCryptoRejectionMessage)
                    .map(CryptoHandshakeMessage::Rejection)
            }
            QuicTag::ClientHello => {
                (&read_quic_tag_value_map(reader)?)
                    .try_into()
                    .chain_err(|| ErrorKind::UnableToReadCryptoClientHelloMessage)
                    .map(CryptoHandshakeMessage::ClientHello)
            }
            QuicTag::ServerConfiguration => {
                (&read_quic_tag_value_map(reader)?)
                    .try_into()
                    .chain_err(|| ErrorKind::UnableToReadCryptoServerConfigurationMessage)
                    .map(CryptoHandshakeMessage::ServerConfiguration)
            }
            quic_tag @ _ => bail!(ErrorKind::InvalidCryptoHandshakeMessage(quic_tag)),
        }
    }
}