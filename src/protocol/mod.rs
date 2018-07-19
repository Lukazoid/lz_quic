mod readable;
pub use self::readable::Readable;

mod writable;
pub use self::writable::Writable;

#[cfg(test)]
pub fn test_write_read_with_context<T>(
    value: &T,
    context: &<T as Readable>::Context,
) -> ::errors::Result<()>
where
    T: Readable + Writable + PartialEq + ::std::fmt::Debug,
{
    let bytes = value.bytes()?;

    let read: T = Readable::from_bytes_with_context(&bytes[..], context)?;

    assert_eq!(&read, value);

    Ok(())
}

#[cfg(test)]
pub fn test_write_read<T>(value: &T) -> ::errors::Result<()>
where
    T: Readable + Writable + PartialEq + ::std::fmt::Debug,
    <T as Readable>::Context: Default,
{
    test_write_read_with_context(value, &Default::default())
}

mod error_code;
pub use self::error_code::ErrorCode;

mod var_int;
pub use self::var_int::VarInt;

mod version;
pub use self::version::Version;

mod connection_id;
pub use self::connection_id::ConnectionId;

mod server_id;
pub use self::server_id::ServerId;

mod role;
pub use self::role::Role;

mod stream_type;
pub use self::stream_type::StreamType;

mod stream_id;
pub use self::stream_id::StreamId;

mod encryption_level;
pub use self::encryption_level::EncryptionLevel;

mod flow_control;
pub use self::flow_control::FlowControl;

mod transport_parameters;
pub use self::transport_parameters::{ClientHelloMessageParameters,
                                     ClientSpecificTransportParameters,
                                     EncryptedExtensionsMessageParameters, MessageParameters,
                                     RoleSpecificTransportParameters,
                                     ServerSpecificTransportParameters, TransportParameters};

mod stream_offset;
pub use self::stream_offset::StreamOffset;
