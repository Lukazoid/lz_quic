use bytes::BytesMut;
use conv::{TryFrom, ValueFrom};
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
    pub disable_migration: bool,
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
                let len = u32::value_from(supported_versions.len()).expect("the number of supported versions should not exceed the value which can be stored in a u32");
                (len * 4).write(writer)?;
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
    DisableMigration,
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
            DisableMigration => 9,
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
            9 => DisableMigration,
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

impl Writable for TransportParameterId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing transport parameter id {:?}", self);

        u16::from(*self).write(writer)?;

        debug!("written transport parameter id {:?}", self);

        Ok(())
    }
}

type TransportParameterValue = SmallVec<[u8; 4]>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct TransportParameter {
    pub id: TransportParameterId,
    pub value: TransportParameterValue,
}

impl Readable for TransportParameter {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _context: &Self::Context) -> Result<Self> {
        trace!("reading transport parameter");

        let id = TransportParameterId::read(reader)?;

        let length = u16::read(reader)?;

        let value = Readable::read(&mut reader.take(length.into()))?;

        let transport_parameter = Self { id, value };

        debug!("read transport parameter {:?}", transport_parameter);

        Ok(transport_parameter)
    }
}

impl Writable for TransportParameter {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing transport parameter {:?}", self);

        self.id.write(writer)?;

        let length = u16::value_from(self.value.len()).unwrap();

        length.write(writer)?;

        self.value.write(writer)?;

        debug!("written transport parameter {:?}", self);

        Ok(())
    }
}

fn try_get_parameter_value<F, T>(
    hash_map: &HashMap<TransportParameterId, TransportParameterValue>,
    id: TransportParameterId,
    read_value: F,
) -> Result<Option<T>>
where
    F: FnOnce(&[u8]) -> Result<T>,
{
    match hash_map.get(&id) {
        Some(small_vec) => {
            let value = read_value(small_vec)?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

fn get_parameter_value<F, T>(
    hash_map: &HashMap<TransportParameterId, TransportParameterValue>,
    id: TransportParameterId,
    read_value: F,
) -> Result<T>
where
    F: FnOnce(&[u8]) -> Result<T>,
{
    let small_vec = hash_map
        .get(&id)
        .ok_or_else(|| Error::from(ErrorKind::TransportParameterMustBeSpecified(id.into())))?;

    let bytes = &small_vec[..];

    read_value(bytes)
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

        let initial_max_stream_data = get_parameter_value(
            &parameters_by_id,
            TransportParameterId::InitialMaxStreamData,
            u32::from_bytes,
        )?;

        let initial_max_data = get_parameter_value(
            &parameters_by_id,
            TransportParameterId::InitialMaxData,
            u32::from_bytes,
        )?;

        let idle_timeout_seconds = get_parameter_value(
            &parameters_by_id,
            TransportParameterId::IdleTimeout,
            u16::from_bytes,
        )?;

        let initial_max_bidi_streams = try_get_parameter_value(
            &parameters_by_id,
            TransportParameterId::InitialMaxBidiStreams,
            u16::from_bytes,
        )?;

        let initial_max_uni_streams = try_get_parameter_value(
            &parameters_by_id,
            TransportParameterId::InitialMaxUniStreams,
            u16::from_bytes,
        )?;

        let max_packet_size = try_get_parameter_value(
            &parameters_by_id,
            TransportParameterId::MaxPacketSize,
            u16::from_bytes,
        )?;

        let ack_delay_exponent = try_get_parameter_value(
            &parameters_by_id,
            TransportParameterId::AckDelayExponent,
            u8::from_bytes,
        )?;

        let disable_migration = try_get_parameter_value(
            &parameters_by_id,
            TransportParameterId::DisableMigration,
            |_| Ok(true),
        )?.unwrap_or(false);

        let stateless_reset_token;
        let preferred_address;
        if matches!(context, Role::Client) {
            stateless_reset_token = try_get_parameter_value(
                &parameters_by_id,
                TransportParameterId::StatelessResetToken,
                <[u8; 16]>::from_bytes,
            )?;
            preferred_address = try_get_parameter_value(
                &parameters_by_id,
                TransportParameterId::PreferredAddress,
                |_| Ok(()),
            )?;
        } else {
            stateless_reset_token = None;
            preferred_address = None;
        }

        let transport_parameters = Self {
            message_parameters,
            initial_max_stream_data,
            initial_max_data,
            idle_timeout_seconds,
            initial_max_bidi_streams,
            initial_max_uni_streams,
            max_packet_size,
            ack_delay_exponent,
            disable_migration,
            stateless_reset_token,
            preferred_address,
        };

        debug!("read transport parameters {:?}", transport_parameters);

        Ok(transport_parameters)
    }
}

impl Writable for TransportParameters {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing transport parameters {:?}", self);

        self.message_parameters.write(writer)?;

        let mut transport_parameters: SmallVec<[_; 10]> = SmallVec::new();
        transport_parameters.push(TransportParameter {
            id: TransportParameterId::InitialMaxStreamData,
            value: self.initial_max_stream_data.bytes_small()?,
        });
        transport_parameters.push(TransportParameter {
            id: TransportParameterId::InitialMaxData,
            value: self.initial_max_data.bytes_small()?,
        });
        transport_parameters.push(TransportParameter {
            id: TransportParameterId::IdleTimeout,
            value: self.idle_timeout_seconds.bytes_small()?,
        });
        if let Some(value) = self.initial_max_bidi_streams {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::InitialMaxBidiStreams,
                value: value.bytes_small()?,
            });
        }
        if let Some(value) = self.initial_max_uni_streams {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::InitialMaxUniStreams,
                value: value.bytes_small()?,
            });
        }
        if let Some(value) = self.max_packet_size {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::MaxPacketSize,
                value: value.bytes_small()?,
            });
        }
        if let Some(value) = self.ack_delay_exponent {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::AckDelayExponent,
                value: value.bytes_small()?,
            });
        }
        if self.disable_migration {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::DisableMigration,
                value: SmallVec::new(),
            });
        }
        if let Some(value) = self.stateless_reset_token {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::StatelessResetToken,
                value: value.bytes_small()?,
            });
        }
        if self.preferred_address.is_some() {
            transport_parameters.push(TransportParameter {
                id: TransportParameterId::PreferredAddress,
                value: SmallVec::new(),
            });
        }

        let total_len: usize = transport_parameters
            .iter()
            .map(|p| 2 + 2 + p.value.len())
            .sum();

        u16::value_from(total_len).unwrap().write(writer)?;

        transport_parameters.write(writer)?;

        debug!("written transport parameters {:?}", self);

        Ok(())
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
            disable_migration: false,
            stateless_reset_token: None,
            preferred_address: None,
        };

        // the read context is from the server perspective;
        protocol::test_write_read_with_context(&transport_parameters, &Role::Server).unwrap();
    }

    #[test]
    fn write_read_server_encrypted_extensions() {
        let transport_parameters = TransportParameters {
            message_parameters: MessageParameters::EncryptedExtensions {
                negotiated_version: Version::DRAFT_IETF_08,
                supported_versions: hashset!(Version::DRAFT_IETF_08),
            },

            initial_max_stream_data: 8192,
            initial_max_data: 65536,
            idle_timeout_seconds: 120,
            initial_max_bidi_streams: Some(8),
            initial_max_uni_streams: Some(8),
            max_packet_size: Some(1024),
            ack_delay_exponent: Some(162),
            disable_migration: false,
            stateless_reset_token: None,
            preferred_address: Some(()),
        };

        // the read context is from the server perspective;
        protocol::test_write_read_with_context(&transport_parameters, &Role::Client).unwrap();
    }
}
