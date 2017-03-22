use errors::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use std::io::{Read, Write};
use std::cmp::Ordering;
use std::str;
use hex::ToHex;
use writable::Writable;
use readable::Readable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, Hash)]
pub enum QuicTag {
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

impl Display for QuicTag {
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

impl<'a> From<&'a QuicTag> for [u8; 4] {
    fn from(value: &'a QuicTag) -> Self {
        (*value).into()
    }
}

impl<'a> From<&'a QuicTag> for u32 {
    fn from(value: &'a QuicTag) -> Self {
        (*value).into()
    }
}


macro_rules! build_lookups {
    ($($tag:tt => $value:expr,)*) => {
                
        impl From<QuicTag> for u32 {
            fn from(value: QuicTag) -> Self {
                match value {
                $(
                    QuicTag::$tag => LittleEndian::read_u32($value),
                )*
                    QuicTag::Custom(ref value) => LittleEndian::read_u32(value),
                }
            }
        }
        
        impl<'a> From<&'a QuicTag> for &'a [u8; 4] {
            fn from(value: &'a QuicTag) -> Self {
                match value {
                $(
                    &QuicTag::$tag => $value,
                )*
                    &QuicTag::Custom(ref value) => value,
                }
            }
        }

        impl From<QuicTag> for [u8; 4] {
            fn from(value: QuicTag) -> Self {
                match value {
                $(
                    QuicTag::$tag => {
                        // Have to clone the static byte array
                        $value.clone()
                    },
                )*
                    QuicTag::Custom(value) => value,
                }
            }
        }

        impl From<[u8; 4]> for QuicTag {
            fn from(value: [u8; 4]) -> Self {
                match &value {
                $(
                    $value => QuicTag::$tag,
                )*
                    _ => QuicTag::Custom(value),
                }
            }
        }

        impl<'a> From<&'a [u8; 4]> for QuicTag {
            fn from(value: &'a [u8; 4]) -> Self {
                match value {
                $(
                    $value => QuicTag::$tag,
                )*
                    bytes @ _ => QuicTag::Custom(bytes.clone()),
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

impl Writable for QuicTag {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        Ok(writer.write_u32::<LittleEndian>(u32::from(self))?)
    }
}

impl Readable for QuicTag {
    fn read<R: Read>(reader: &mut R) -> Result<QuicTag> {
        let mut bytes = [0; 4];
        let quic_tag = reader.read_exact(&mut bytes)
            .map(|_| Self::from(&bytes))?;

        Ok(quic_tag)
    }
}

impl PartialOrd for QuicTag {
    fn partial_cmp(&self, other: &QuicTag) -> Option<Ordering> {
        u32::from(self).partial_cmp(&u32::from(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use test::Bencher;

    #[bench]
    fn bench_three_byte_read(b: &mut Bencher) {
        let input = [0x53, 0x4e, 0x49, 0x00];
        b.iter(|| {
            let mut cursor = Cursor::new(&input);

            QuicTag::read(&mut cursor).unwrap();
        });
    }

    #[bench]
    fn bench_four_byte_read(b: &mut Bencher) {
        let input = [0x43, 0x48, 0x4c, 0x4f];
        b.iter(|| {
            let mut cursor = Cursor::new(&input);

            QuicTag::read(&mut cursor).unwrap();
        });
    }

    macro_rules! to_hex_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (QuicTag, u32) = $value;
                    
                    assert_eq!(expected, u32::from(input));
                }
            )*
        }
    }

    to_hex_tests!{
        client_hello_to_hex: (QuicTag::ClientHello, 0x4f4c4843),
        server_name_identification_to_hex: (QuicTag::ServerNameIndication, 0x494e53),
    }

    macro_rules! read_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (&[u8], QuicTag) = $value;
                    
                    let mut cursor = Cursor::new(input);
                    
                    assert_eq!(expected, QuicTag::read(&mut cursor).unwrap());
                }
            )*
        }
    }

    read_tests! {
        read_client_hello: (&[0x43, 0x48, 0x4c, 0x4f], QuicTag::ClientHello),
        read_server_name_indication: (&[0x53, 0x4e, 0x49, 0x00], QuicTag::ServerNameIndication),
    }

    macro_rules! display_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (QuicTag, &'static str) = $value;
                    
                    assert_eq!(expected, input.to_string());
                }
            )*
        }
    }
    display_tests! {
        display_chlo: (QuicTag::ClientHello, "CHLO"),
        display_custom: (QuicTag::Custom([0xff, 0x54, 0x52, 0x43]), "0xff545243"),
    }

}