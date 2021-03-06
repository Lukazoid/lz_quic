use conv::{TryFrom, ValueFrom};
use errors::*;
use protocol::{Readable, Version, Writable};
use smallvec::SmallVec;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::io::{Read, Write};

pub trait MessageParameters: Debug + Readable + Writable {}

pub trait RoleSpecificTransportParameters: Debug {
    fn from_transport_parameters(
        transport_parameters: &HashMap<TransportParameterId, TransportParameterValue>,
    ) -> Result<Self>
    where
        Self: Sized;

    fn add_transport_parameters(
        &self,
        transport_parameters: &mut HashMap<TransportParameterId, TransportParameterValue>,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServerSpecificTransportParameters {
    pub stateless_reset_token: Option<[u8; 16]>,

    // TODO LH What type is preferred_address?
    pub preferred_address: Option<()>,
}

impl RoleSpecificTransportParameters for ServerSpecificTransportParameters {
    fn from_transport_parameters(
        transport_parameters: &HashMap<TransportParameterId, TransportParameterValue>,
    ) -> Result<Self> {
        let stateless_reset_token = try_get_parameter_value(
            &transport_parameters,
            TransportParameterId::StatelessResetToken,
            <[u8; 16]>::from_bytes,
        )?;
        let preferred_address = try_get_parameter_value(
            &transport_parameters,
            TransportParameterId::PreferredAddress,
            |_| Ok(()),
        )?;

        Ok(Self {
            stateless_reset_token,
            preferred_address,
        })
    }

    fn add_transport_parameters(
        &self,
        transport_parameters: &mut HashMap<TransportParameterId, TransportParameterValue>,
    ) -> Result<()> {
        if let Some(value) = self.stateless_reset_token {
            transport_parameters.insert(
                TransportParameterId::StatelessResetToken,
                value.bytes_small()?,
            );
        }
        if self.preferred_address.is_some() {
            transport_parameters.insert(TransportParameterId::PreferredAddress, SmallVec::new());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientSpecificTransportParameters;

impl RoleSpecificTransportParameters for ClientSpecificTransportParameters {
    fn from_transport_parameters(
        _transport_parameters: &HashMap<TransportParameterId, TransportParameterValue>,
    ) -> Result<Self> {
        Ok(Self {})
    }

    fn add_transport_parameters(
        &self,
        _transport_parameters: &mut HashMap<TransportParameterId, TransportParameterValue>,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransportParameters<M, R> {
    pub message_parameters: M,

    pub initial_max_stream_data: u32,
    pub initial_max_data: u32,
    pub idle_timeout: u16,
    pub initial_max_bidi_streams: Option<u16>,
    pub initial_max_uni_streams: Option<u16>,
    pub max_packet_size: Option<u16>,
    pub ack_delay_exponent: Option<u8>,
    pub disable_migration: bool,

    pub role_specific_transport_parameters: R,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientHelloMessageParameters {
    pub initial_version: Version,
}

impl MessageParameters for ClientHelloMessageParameters {}

impl Readable for ClientHelloMessageParameters {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading client hello message parameters");

        let initial_version = Version::read(reader)?;

        let client_hello_message_parameters = ClientHelloMessageParameters { initial_version };

        debug!(
            "read client hello message parameters {:?}",
            client_hello_message_parameters
        );

        Ok(client_hello_message_parameters)
    }
}

impl Writable for ClientHelloMessageParameters {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing client hello message parameters {:?}", self);

        self.initial_version.write(writer)?;

        debug!("written client hello message parameters {:?}", self);

        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EncryptedExtensionsMessageParameters {
    pub negotiated_version: Version,
    pub supported_versions: HashSet<Version>,
}

impl MessageParameters for EncryptedExtensionsMessageParameters {}

impl Readable for EncryptedExtensionsMessageParameters {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading encrypted extensions message parameters");

        let negotiated_version = Version::read(reader)?;

        let supported_version_bytes = u32::read(reader)?;

        let supported_versions: HashSet<Version> =
            Version::collect(&mut reader.take(supported_version_bytes.into()))?;

        let encrypted_extensions_message_parameters = EncryptedExtensionsMessageParameters {
            negotiated_version,
            supported_versions,
        };

        debug!(
            "read encrypted extensions message parameters {:?}",
            encrypted_extensions_message_parameters
        );

        Ok(encrypted_extensions_message_parameters)
    }
}

impl Writable for EncryptedExtensionsMessageParameters {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing encrypted extensions message parameters {:?}", self);

        self.negotiated_version.write(writer)?;
        let len = u32::value_from(self.supported_versions.len()).expect("the number of supported versions should not exceed the value which can be stored in a u32");
        (len * 4).write(writer)?;
        self.supported_versions.write(writer)?;

        debug!("written encrypted extensions message parameters {:?}", self);

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TransportParameterId {
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

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl<M: MessageParameters, RS: RoleSpecificTransportParameters> Readable
    for TransportParameters<M, RS>
where
    <M as Readable>::Context: Default,
{
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading transport parameters");

        let message_parameters = M::read(reader)?;

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

        let idle_timeout = get_parameter_value(
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

        let role_specific_transport_parameters = RS::from_transport_parameters(&parameters_by_id)?;

        let transport_parameters = Self {
            message_parameters,
            initial_max_stream_data,
            initial_max_data,
            idle_timeout,
            initial_max_bidi_streams,
            initial_max_uni_streams,
            max_packet_size,
            ack_delay_exponent,
            disable_migration,
            role_specific_transport_parameters,
        };

        debug!("read transport parameters {:?}", transport_parameters);

        Ok(transport_parameters)
    }
}

impl<M: MessageParameters, R: RoleSpecificTransportParameters> Writable
    for TransportParameters<M, R>
where
    <M as Readable>::Context: Default,
{
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing transport parameters {:?}", self);

        self.message_parameters.write(writer)?;

        let mut transport_parameters = HashMap::new();
        transport_parameters.insert(
            TransportParameterId::InitialMaxStreamData,
            self.initial_max_stream_data.bytes_small()?,
        );
        transport_parameters.insert(
            TransportParameterId::InitialMaxData,
            self.initial_max_data.bytes_small()?,
        );
        transport_parameters.insert(
            TransportParameterId::IdleTimeout,
            self.idle_timeout.bytes_small()?,
        );
        if let Some(value) = self.initial_max_bidi_streams {
            transport_parameters.insert(
                TransportParameterId::InitialMaxBidiStreams,
                value.bytes_small()?,
            );
        }
        if let Some(value) = self.initial_max_uni_streams {
            transport_parameters.insert(
                TransportParameterId::InitialMaxUniStreams,
                value.bytes_small()?,
            );
        }
        if let Some(value) = self.max_packet_size {
            transport_parameters.insert(TransportParameterId::MaxPacketSize, value.bytes_small()?);
        }
        if let Some(value) = self.ack_delay_exponent {
            transport_parameters
                .insert(TransportParameterId::AckDelayExponent, value.bytes_small()?);
        }
        if self.disable_migration {
            transport_parameters.insert(TransportParameterId::DisableMigration, SmallVec::new());
        }

        self.role_specific_transport_parameters
            .add_transport_parameters(&mut transport_parameters)?;

        let total_len: usize = transport_parameters
            .values()
            .map(|value| 2 + 2 + value.len())
            .sum();

        u16::value_from(total_len).unwrap().write(writer)?;

        let transport_parameters: Vec<_> = transport_parameters
            .into_iter()
            .map(|(id, value)| TransportParameter { id, value })
            .collect();

        transport_parameters.write(writer)?;

        debug!("written transport parameters {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{ClientHelloMessageParameters, ClientSpecificTransportParameters,
                EncryptedExtensionsMessageParameters, ServerSpecificTransportParameters,
                TransportParameters};
    use protocol::{self, Version};

    #[test]
    fn write_read_client_hello() {
        let transport_parameters = TransportParameters {
            message_parameters: ClientHelloMessageParameters {
                initial_version: Version::DRAFT_IETF_08,
            },

            initial_max_stream_data: 8192,
            initial_max_data: 65536,
            idle_timeout: 120,
            initial_max_bidi_streams: Some(8),
            initial_max_uni_streams: Some(8),
            max_packet_size: Some(1024),
            ack_delay_exponent: Some(162),
            disable_migration: false,
            role_specific_transport_parameters: ClientSpecificTransportParameters,
        };

        protocol::test_write_read(&transport_parameters).unwrap();
    }

    #[test]
    fn write_read_server_encrypted_extensions() {
        let transport_parameters = TransportParameters {
            message_parameters: EncryptedExtensionsMessageParameters {
                negotiated_version: Version::DRAFT_IETF_08,
                supported_versions: hashset![Version::DRAFT_IETF_08],
            },

            initial_max_stream_data: 8192,
            initial_max_data: 65536,
            idle_timeout: 120,
            initial_max_bidi_streams: Some(8),
            initial_max_uni_streams: Some(8),
            max_packet_size: Some(1024),
            ack_delay_exponent: Some(162),
            disable_migration: false,
            role_specific_transport_parameters: ServerSpecificTransportParameters {
                stateless_reset_token: None,
                preferred_address: Some(()),
            },
        };

        protocol::test_write_read(&transport_parameters).unwrap();
    }
}
