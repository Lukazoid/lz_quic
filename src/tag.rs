use errors::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, Write};
use std::cmp::Ordering;
use std::str;
use hex::ToHex;
use writable::Writable;
use readable::Readable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, Hash)]
pub enum Tag {
    ClientHello,
    ServerHello,
    ServerNameIndication,
    SourceAddressToken,
    ProofDemand,
    CommonCertificateSets,
    CachedCertificates,
    Version,
    Fnv1aHash,
    Rejection,
    ServerConfiguration,
    ServerNonce,
    ServerConfigurationTimeToLive,
    CertificateChain,
    ProofOfAuthenticity,
    ServerConfigurationId,
    KeyExchangeAlgorithm,
    Curve25519,
    P256,
    AuthenticatedEncryptionAlgorithm,
    AesGcm,
    Salsa20Poly1305,
    PublicValue,
    Orbit,
    ServerConfigurationExpiry,
    ClientNonce,
    ClientEncryptedTagValues,
    ChannelIdKey,
    ChannelIdSignature,
    X509,
    Custom([u8; 4]),
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let bytes: &[u8; 4] = self.into();

        // TODO LH It would be better to check bytes is an ASCII string instead of a utf8 string
        if let Ok(string) = str::from_utf8(bytes) {
            f.write_str(string)
        } else {
            write!(f, "0x{}", bytes.to_hex())
        }

    }
}

impl<'a> From<&'a Tag> for [u8; 4] {
    fn from(value: &'a Tag) -> Self {
        (*value).into()
    }
}

impl<'a> From<&'a Tag> for u32 {
    fn from(value: &'a Tag) -> Self {
        (*value).into()
    }
}


macro_rules! build_lookups {
    ($($tag:tt => $value:expr,)*) => {
                
        impl From<Tag> for u32 {
            fn from(value: Tag) -> Self {
                match value {
                $(
                    Tag::$tag => LittleEndian::read_u32($value),
                )*
                    Tag::Custom(ref value) => LittleEndian::read_u32(value),
                }
            }
        }
        
        impl<'a> From<&'a Tag> for &'a [u8; 4] {
            fn from(value: &'a Tag) -> Self {
                match value {
                $(
                    &Tag::$tag => $value,
                )*
                    &Tag::Custom(ref value) => value,
                }
            }
        }

        impl From<Tag> for [u8; 4] {
            fn from(value: Tag) -> Self {
                match value {
                $(
                    Tag::$tag => {
                        // Have to clone the static byte array
                        $value.clone()
                    },
                )*
                    Tag::Custom(value) => value,
                }
            }
        }

        impl From<[u8; 4]> for Tag {
            fn from(value: [u8; 4]) -> Self {
                match &value {
                $(
                    $value => Tag::$tag,
                )*
                    _ => Tag::Custom(value),
                }
            }
        }

        impl<'a> From<&'a [u8; 4]> for Tag {
            fn from(value: &'a [u8; 4]) -> Self {
                match value {
                $(
                    $value => Tag::$tag,
                )*
                    bytes @ _ => Tag::Custom(bytes.clone()),
                }
            }
        }
    }
}

build_lookups!{
    ClientHello => b"CHLO",
    ServerHello => b"SHLO",
    ServerNameIndication => b"SNI\0",
    SourceAddressToken => b"STK\0",
    ProofDemand => b"PDMD",
    CommonCertificateSets => b"CCS\0",
    CachedCertificates => b"CCRT",
    Version => b"VER\0",
    Fnv1aHash => b"XLCT",
    Rejection => b"REJ\0",
    ServerConfiguration => b"SCFG",
    ServerNonce => b"SNO\0",
    ServerConfigurationTimeToLive => b"STTL",
    CertificateChain => b"ff54",
    ProofOfAuthenticity => b"PROF",
    ServerConfigurationId => b"SCID",
    KeyExchangeAlgorithm => b"KEXS",
    Curve25519 => b"C255",
    P256 => b"P256",
    AuthenticatedEncryptionAlgorithm => b"AEAD",
    AesGcm => b"AESG",
    Salsa20Poly1305 => b"S20P",
    PublicValue  => b"PUBS",
    Orbit => b"ORBT",
    ServerConfigurationExpiry => b"EXPY",
    ClientNonce => b"NONC",
    ClientEncryptedTagValues => b"CETV",
    ChannelIdKey => b"CIDK",
    ChannelIdSignature => b"CIDS",
    X509 => b"X509",
}

impl Writable for Tag {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        Ok(u32::from(self).write(writer)?)
    }
}

impl Readable for Tag {
    fn read<R: Read>(reader: &mut R) -> Result<Tag> {
        let mut bytes = [0; 4];
        let tag = reader.read_exact(&mut bytes)
            .chain_err(|| ErrorKind::UnableToReadBytes)
            .map(|_| Self::from(&bytes))?;

        Ok(tag)
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Tag) -> Option<Ordering> {
        u32::from(self).partial_cmp(&u32::from(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    macro_rules! to_hex_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (Tag, u32) = $value;
                    
                    assert_eq!(expected, u32::from(input));
                }
            )*
        }
    }

    to_hex_tests!{
        client_hello_to_hex: (Tag::ClientHello, 0x4f4c4843),
        server_name_identification_to_hex: (Tag::ServerNameIndication, 0x494e53),
    }

    macro_rules! read_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (&[u8], Tag) = $value;
                    
                    let mut cursor = Cursor::new(input);
                    
                    assert_eq!(expected, Tag::read(&mut cursor).unwrap());
                }
            )*
        }
    }

    read_tests! {
        read_client_hello: (&[0x43, 0x48, 0x4c, 0x4f], Tag::ClientHello),
        read_server_name_indication: (&[0x53, 0x4e, 0x49, 0x00], Tag::ServerNameIndication),
    }

    macro_rules! display_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (Tag, &'static str) = $value;
                    
                    assert_eq!(expected, input.to_string());
                }
            )*
        }
    }
    display_tests! {
        display_chlo: (Tag::ClientHello, "CHLO"),
        display_custom: (Tag::Custom([0xff, 0x54, 0x52, 0x43]), "0xff545243"),
    }

}