use errors::*;
use protocol::{Readable, Role, Version, Writable};
use std::collections::HashSet;
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransportParameters {
    pub message_parameters: MessageParameters,

    pub initial_max_stream_data: u32,
    pub initial_max_data: u32,
    pub idle_timeout_seconds: u16,
    pub initial_max_bidi_streams: Option<u16>,
    pub initial_max_uni_streams: Option<u16>,
    pub max_packet_size: Option<u16>,
    pub ack_delay_exponent: Option<u8>,
    pub disable_migration: Option<bool>,
    pub stateless_reset_token: Option<[u8; 16]>,

    // TODO LH What type is preferred_address?
    pub preferred_address: Option<()>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MessageParameters {
    ClientHello {
        initial_version: Version,
    },
    EncryptedExtensions {
        negotiated_version: Version,
        supported_versions: HashSet<Version>,
    },
}

impl MessageParameters {
    pub fn from_role(&self) -> Role {
        match self {
            MessageParameters::ClientHello { .. } => Role::Client,
            MessageParameters::EncryptedExtensions { .. } => Role::Server,
        }
    }
}

impl Readable for MessageParameters {
    type Context = Role;

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading message parameters");

        let message_parameters = match context {
            Role::Server => {
                // we are the server so we will read the clients hello
                let initial_version = Version::read(reader)?;

                MessageParameters::ClientHello { initial_version }
            }
            Role::Client => {
                // we are the client so we will read the servers encrypted extensions
                let negotiated_version = Version::read(reader)?;

                let supported_version_bytes = u32::read(reader)?;

                let supported_versions: HashSet<Version> =
                    Version::collect(&mut reader.take(supported_version_bytes.into()))?;

                MessageParameters::EncryptedExtensions {
                    negotiated_version,
                    supported_versions,
                }
            }
        };

        debug!("read message parameters {:?}", message_parameters);

        Ok(message_parameters)
    }
}

impl Writable for MessageParameters {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing message parameters {:?}", self);

        match self {
            MessageParameters::ClientHello { initial_version } => {
                initial_version.write(writer)?;
            }
            MessageParameters::EncryptedExtensions {
                negotiated_version,
                supported_versions,
            } => {
                negotiated_version.write(writer)?;
                (supported_versions.len() as u32).write(writer)?;
                supported_versions.write(writer)?;
            }
        }

        debug!("written message parameters {:?}", self);

        Ok(())
    }
}

#[repr(u16)]
enum TransportParameterId {
    InitialMaxStreamData = 0,
    InitialMaxData = 1,
    InitialMaxBidiStreams = 2,
    IdleTimeout = 3,
    PreferredAddress = 4,
    MaxPacketSize = 5,
    StatelessResetToken = 6,
    AckDelayExponent = 7,
    IntiialMaxUniStreams = 8,
    DisabelMigration = 9,
}

impl Readable for TransportParameters {
    type Context = Role;

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading transport parameters");

        let message_parameters = MessageParameters::read_with_context(reader, context)?;
        unimplemented!()

        // let transport_parameters = Self { role: context };

        // debug!("read transport parameters {:?}", transport_parameters);

        // Ok(transport_parameters)
    }
}

impl Writable for TransportParameters {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing transport parameters {:?}", self);

        self.message_parameters.write(writer)?;

        debug!("written transport parameters {:?}", self);
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::{MessageParameters, TransportParameters};
    use protocol::{self, Role, Version};

    #[test]
    fn write_read_client_hello() {
        let transport_parameters = TransportParameters {
            message_parameters: MessageParameters::ClientHello {
                initial_version: Version::DRAFT_IETF_08,
            },

            initial_max_stream_data: 8192,
            initial_max_data: 65536,
            idle_timeout_seconds: 120,
            initial_max_bidi_streams: Some(8),
            initial_max_uni_streams: Some(8),
            max_packet_size: Some(1024),
            ack_delay_exponent: Some(162),
            disable_migration: Some(false),
            stateless_reset_token: Some(*b"sixteen bytes!!!"),
            preferred_address: Some(()),
        };

        // the read context is from the server perspective;
        protocol::test_write_read_with_context(&transport_parameters, &Role::Server).unwrap();
    }
}
