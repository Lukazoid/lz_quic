use errors::*;
use std::fmt::{Display, Formatter, Result as FmtResult};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{Read, Write};
use std::cmp::Ordering;
use std::str;
use hex::ToHex;
use protocol::{Readable, Writable};

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
        let bytes = self.bytes();

        // TODO LH It would be better to check bytes is an ASCII string instead of a utf8 string
        if let Ok(string) = str::from_utf8(bytes) {
            f.write_str(string)
        } else {
            write!(f, "0x{}", bytes.to_hex())
        }

    }
}

macro_rules! build_lookups {
    ($($tag:tt => $value:expr,)*) => {
        impl Tag {
            pub fn bytes(&self) -> &[u8;4] {
                match *self {
                $(
                    Tag::$tag => $value,
                )*
                    Tag::Custom(ref value) => value,
                }
            }

            /// Provided so that references to static byte arrays may be used to create a `Tag`.
            pub fn from_bytes(bytes: &[u8; 4]) -> Tag {
                match bytes {
                $(
                    $value => Tag::$tag,
                )*
                    _ => Tag::Custom(bytes.clone()),
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
    CertificateChain => b"\xff\x54\x52\x43",
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

impl Tag {
    fn as_u32(&self) -> u32 {
        let bytes = self.bytes();
        LittleEndian::read_u32(bytes)
    }
}

impl Writable for Tag {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing tag {:?}", self);
        self.as_u32().write(writer)?;
        debug!("written tag {:?}", self);

        Ok(())
    }
}

impl Readable for Tag {
    fn read<R: Read>(reader: &mut R) -> Result<Tag> {
        trace!("reading tag");
        let mut bytes = [0; 4];
        let tag = reader.read_exact(&mut bytes)
            .chain_err(|| ErrorKind::FailedToReadBytes)
            .map(|_| Self::from(bytes))?;
        debug!("read tag {:?}", tag);

        Ok(tag)
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Tag) -> Option<Ordering> {
        self.as_u32().partial_cmp(&other.as_u32())
    }
}

#[cfg(all(feature = "unstable", test))]
mod bench {
    use super::*;
    use test::Bencher;

    #[bench]
    fn tag_from_bytes(b: &mut Bencher) {
        b.iter(|| {
            let _ = Tag::from_bytes(b"KEXS");
        });
    }

    #[bench]
    fn bytes_from_tag(b: &mut Bencher) {
        b.iter(|| {
            let _ = Tag::KeyExchangeAlgorithm.bytes();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    macro_rules! write_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (Tag, &[u8; 4]) = $value;
                    
                    let mut vec = Vec::new();
                    input.write_to_vec(&mut vec);

                    assert_eq!(expected, vec.as_slice());
                }
            )*
        }
    }

    write_tests!{
        write_client_hello: (Tag::ClientHello, &[0x43, 0x48, 0x4c, 0x4f]),
        write_server_name_indication: (Tag::ServerNameIndication, &[0x53, 0x4e, 0x49, 0x00]),
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