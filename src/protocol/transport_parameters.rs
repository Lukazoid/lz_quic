use conv::TryFrom;
use errors::*;
use protocol::{Readable, Role, Version, Writable};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
enum TransportParameterId {
    InitialMaxStreamData,
    InitialMaxData,
    InitialMaxBidiStreams,
    IdleTimeout,
    PreferredAddress,
    MaxPacketSize,
    StatelessResetToken,
    AckDelayExponent,
    InitialMaxUniStreams,
    DisabelMigration,
}

impl From<TransportParameterId> for u16 {
    fn from(value: TransportParameterId) -> Self {
        use self::TransportParameterId::*;
        match value {
            InitialMaxStreamData => 0,
            InitialMaxData => 1,
            InitialMaxBidiStreams => 2,
            IdleTimeout => 3,
            PreferredAddress => 4,
            MaxPacketSize => 5,
            StatelessResetToken => 6,
            AckDelayExponent => 7,
            InitialMaxUniStreams => 8,
            DisabelMigration => 9,
        }
    }
}

impl TryFrom<u16> for TransportParameterId {
    type Err = Error;

    fn try_from(value: u16) -> Result<Self> {
        use self::TransportParameterId::*;
        let transport_parameter_id = match value {
            0 => InitialMaxStreamData,
            1 => InitialMaxData,
            2 => InitialMaxBidiStreams,
            3 => IdleTimeout,
            4 => PreferredAddress,
            5 => MaxPacketSize,
            6 => StatelessResetToken,
            7 => AckDelayExponent,
            8 => InitialMaxUniStreams,
            9 => DisabelMigration,
            _ => bail!(ErrorKind::InvalidTransportParameterId(value)),
        };

        Ok(transport_parameter_id)
    }
}

impl Readable for TransportParameterId {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _context: &Self::Context) -> Result<Self> {
        trace!("reading transport parameter id");

        let id = u16::read(reader)?;
        let transport_parameter_id = TransportParameterId::try_from(id)?;

        debug!("read transport parameter id {:?}", transport_parameter_id);

        Ok(transport_parameter_id)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct TransportParameter {
    pub id: TransportParameterId,
    pub value: Vec<u8>,
}

impl Readable for TransportParameter {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _context: &Self::Context) -> Result<Self> {
        trace!("reading transport parameter");

        let id = TransportParameterId::read(reader)?;

        let length = u16::read(reader)?;

        let value = Readable::read(&mut reader.take(length as u64))?;

        let transport_parameter = Self { id, value };

        debug!("read transport parameter {:?}", transport_parameter);

        Ok(transport_parameter)
    }
}

impl Readable for TransportParameters {
    type Context = Role;

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading transport parameters");

        let message_parameters = MessageParameters::read_with_context(reader, context)?;

        let parameters_len = u16::read(reader)?;

        let parameters: SmallVec<[_; 10]> =
            TransportParameter::collect(&mut reader.take(parameters_len.into()))?;

        let mut parameters_by_id = HashMap::with_capacity(parameters.len());
        for TransportParameter { id, value } in parameters {
            if parameters_by_id.insert(id, value).is_some() {
                bail!(ErrorKind::DuplicateTransportParameter(id.into()));
            };
        }

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
