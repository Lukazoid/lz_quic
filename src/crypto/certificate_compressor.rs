use errors::*;
use crypto::certificate::Certificate;
use crypto::certificate_set::CertificateSet;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};
use std::collections::{HashSet, HashMap};
use flate2::{Compress, Compression, Decompress, Flush, Status};
use std::io::{Cursor, Read, Write};
use itertools::Itertools;
use writable::Writable;
use readable::Readable;
use options_slice_ext::OptionsSliceExt;
use crypto::common_certificate_sets;

lazy_static! {
    pub static ref CERTIFICATE_COMPRESSOR: CertificateCompressor = CertificateCompressor::new(common_certificate_sets::build_common_certificate_sets());
}

#[derive(Debug, Clone)]
pub struct CertificateCompressor {
    common_certificate_sets: HashMap<u64, CertificateSet>,
}

#[derive(Debug)]
enum CompressedCertificateEntry {
    EndOfList,
    Compressed,
    Cached {
        hash: u64,
    },
    Common {
        set_hash: u64,
        index: u32,
    },
}

impl Writable for CompressedCertificateEntry {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            CompressedCertificateEntry::EndOfList => {
                0u8.write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteCompressedCertificateEntryType)?
            }
            CompressedCertificateEntry::Compressed => {
                1u8.write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteCompressedCertificateEntryType)?
            }
            CompressedCertificateEntry::Cached { hash } => {
                2u8.write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteCompressedCertificateEntryType)?;

                hash.write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteCachedCertificateHash)?;
            }
            CompressedCertificateEntry::Common { set_hash, index } => {
                3u8.write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteCompressedCertificateEntryType)?;

                set_hash.write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteCommonCertificateSetHash)?;
                index.write(writer).chain_err(|| ErrorKind::UnableToWriteCommonCertificateIndex)?;
            }
        };

        Ok(())
    }
}

impl Readable for CompressedCertificateEntry {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let entry_type = u8::read(reader)
            .chain_err(|| ErrorKind::UnableToReadCompressedCertificateEntryType)?;

        let intermediate_entry = match entry_type {
            0 => CompressedCertificateEntry::EndOfList,
            1 => CompressedCertificateEntry::Compressed,
            2 => {
                let hash = u64::read(reader)
                    .chain_err(|| ErrorKind::UnableToReadCachedCertificateHash)?;

                CompressedCertificateEntry::Cached { hash: hash }
            }
            3 => {
                let set_hash = u64::read(reader)
                    .chain_err(|| ErrorKind::UnableToReadCommonCertificateSetHash)?;
                let index = u32::read(reader)
                    .chain_err(|| ErrorKind::UnableToReadCommonCertificateIndex)?;

                CompressedCertificateEntry::Common {
                    set_hash: set_hash,
                    index: index,
                }
            }
            entry_type @ _ => bail!(ErrorKind::InvalidCompressedCertificateEntryType(entry_type)),
        };

        Ok(intermediate_entry)
    }
}


static COMMON_SUBSTRINGS: &'static [u8] =
    &[0x04, 0x02, 0x30, 0x00, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x1d, 0x25, 0x04, 0x16, 0x30, 0x14,
      0x06, 0x08, 0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01, 0x06, 0x08, 0x2b, 0x06, 0x01,
      0x05, 0x05, 0x07, 0x03, 0x02, 0x30, 0x5f, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x86, 0xf8,
      0x42, 0x04, 0x01, 0x06, 0x06, 0x0b, 0x60, 0x86, 0x48, 0x01, 0x86, 0xfd, 0x6d, 0x01, 0x07,
      0x17, 0x01, 0x30, 0x33, 0x20, 0x45, 0x78, 0x74, 0x65, 0x6e, 0x64, 0x65, 0x64, 0x20, 0x56,
      0x61, 0x6c, 0x69, 0x64, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x53, 0x20, 0x4c, 0x69, 0x6d,
      0x69, 0x74, 0x65, 0x64, 0x31, 0x34, 0x20, 0x53, 0x53, 0x4c, 0x20, 0x43, 0x41, 0x30, 0x1e,
      0x17, 0x0d, 0x31, 0x32, 0x20, 0x53, 0x65, 0x63, 0x75, 0x72, 0x65, 0x20, 0x53, 0x65, 0x72,
      0x76, 0x65, 0x72, 0x20, 0x43, 0x41, 0x30, 0x2d, 0x61, 0x69, 0x61, 0x2e, 0x76, 0x65, 0x72,
      0x69, 0x73, 0x69, 0x67, 0x6e, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x45, 0x2d, 0x63, 0x72, 0x6c,
      0x2e, 0x76, 0x65, 0x72, 0x69, 0x73, 0x69, 0x67, 0x6e, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x45,
      0x2e, 0x63, 0x65, 0x72, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01,
      0x01, 0x05, 0x05, 0x00, 0x03, 0x82, 0x01, 0x01, 0x00, 0x4a, 0x2e, 0x63, 0x6f, 0x6d, 0x2f,
      0x72, 0x65, 0x73, 0x6f, 0x75, 0x72, 0x63, 0x65, 0x73, 0x2f, 0x63, 0x70, 0x73, 0x20, 0x28,
      0x63, 0x29, 0x30, 0x30, 0x09, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x04, 0x02, 0x30, 0x00, 0x30,
      0x1d, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x05, 0x05,
      0x00, 0x03, 0x82, 0x01, 0x01, 0x00, 0x7b, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x1d, 0x0e, 0x30,
      0x82, 0x01, 0x22, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01,
      0x01, 0x05, 0x00, 0x03, 0x82, 0x01, 0x0f, 0x00, 0x30, 0x82, 0x01, 0x0a, 0x02, 0x82, 0x01,
      0x01, 0x00, 0xd2, 0x6f, 0x64, 0x6f, 0x63, 0x61, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x43, 0x2e,
      0x63, 0x72, 0x6c, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x1d, 0x0e, 0x04, 0x16, 0x04, 0x14, 0xb4,
      0x2e, 0x67, 0x6c, 0x6f, 0x62, 0x61, 0x6c, 0x73, 0x69, 0x67, 0x6e, 0x2e, 0x63, 0x6f, 0x6d,
      0x2f, 0x72, 0x30, 0x0b, 0x06, 0x03, 0x55, 0x1d, 0x0f, 0x04, 0x04, 0x03, 0x02, 0x01, 0x30,
      0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x05, 0x05, 0x00, 0x30,
      0x81, 0xca, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02, 0x55, 0x53,
      0x31, 0x10, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x04, 0x08, 0x13, 0x07, 0x41, 0x72, 0x69, 0x7a,
      0x6f, 0x6e, 0x61, 0x31, 0x13, 0x30, 0x11, 0x06, 0x03, 0x55, 0x04, 0x07, 0x13, 0x0a, 0x53,
      0x63, 0x6f, 0x74, 0x74, 0x73, 0x64, 0x61, 0x6c, 0x65, 0x31, 0x1a, 0x30, 0x18, 0x06, 0x03,
      0x55, 0x04, 0x0a, 0x13, 0x11, 0x47, 0x6f, 0x44, 0x61, 0x64, 0x64, 0x79, 0x2e, 0x63, 0x6f,
      0x6d, 0x2c, 0x20, 0x49, 0x6e, 0x63, 0x2e, 0x31, 0x33, 0x30, 0x31, 0x06, 0x03, 0x55, 0x04,
      0x0b, 0x13, 0x2a, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x63, 0x65, 0x72, 0x74, 0x69,
      0x66, 0x69, 0x63, 0x61, 0x74, 0x65, 0x73, 0x2e, 0x67, 0x6f, 0x64, 0x61, 0x64, 0x64, 0x79,
      0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x72, 0x65, 0x70, 0x6f, 0x73, 0x69, 0x74, 0x6f, 0x72, 0x79,
      0x31, 0x30, 0x30, 0x2e, 0x06, 0x03, 0x55, 0x04, 0x03, 0x13, 0x27, 0x47, 0x6f, 0x20, 0x44,
      0x61, 0x64, 0x64, 0x79, 0x20, 0x53, 0x65, 0x63, 0x75, 0x72, 0x65, 0x20, 0x43, 0x65, 0x72,
      0x74, 0x69, 0x66, 0x69, 0x63, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x20, 0x41, 0x75, 0x74, 0x68,
      0x6f, 0x72, 0x69, 0x74, 0x79, 0x31, 0x11, 0x30, 0x0f, 0x06, 0x03, 0x55, 0x04, 0x05, 0x13,
      0x08, 0x30, 0x37, 0x39, 0x36, 0x39, 0x32, 0x38, 0x37, 0x30, 0x1e, 0x17, 0x0d, 0x31, 0x31,
      0x30, 0x0e, 0x06, 0x03, 0x55, 0x1d, 0x0f, 0x01, 0x01, 0xff, 0x04, 0x04, 0x03, 0x02, 0x05,
      0xa0, 0x30, 0x0c, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x01, 0x01, 0xff, 0x04, 0x02, 0x30, 0x00,
      0x30, 0x1d, 0x30, 0x0f, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x01, 0x01, 0xff, 0x04, 0x05, 0x30,
      0x03, 0x01, 0x01, 0x00, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x1d, 0x25, 0x04, 0x16, 0x30, 0x14,
      0x06, 0x08, 0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x01, 0x06, 0x08, 0x2b, 0x06, 0x01,
      0x05, 0x05, 0x07, 0x03, 0x02, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x1d, 0x0f, 0x01, 0x01, 0xff,
      0x04, 0x04, 0x03, 0x02, 0x05, 0xa0, 0x30, 0x33, 0x06, 0x03, 0x55, 0x1d, 0x1f, 0x04, 0x2c,
      0x30, 0x2a, 0x30, 0x28, 0xa0, 0x26, 0xa0, 0x24, 0x86, 0x22, 0x68, 0x74, 0x74, 0x70, 0x3a,
      0x2f, 0x2f, 0x63, 0x72, 0x6c, 0x2e, 0x67, 0x6f, 0x64, 0x61, 0x64, 0x64, 0x79, 0x2e, 0x63,
      0x6f, 0x6d, 0x2f, 0x67, 0x64, 0x73, 0x31, 0x2d, 0x32, 0x30, 0x2a, 0x30, 0x28, 0x06, 0x08,
      0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x02, 0x01, 0x16, 0x1c, 0x68, 0x74, 0x74, 0x70, 0x73,
      0x3a, 0x2f, 0x2f, 0x77, 0x77, 0x77, 0x2e, 0x76, 0x65, 0x72, 0x69, 0x73, 0x69, 0x67, 0x6e,
      0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x63, 0x70, 0x73, 0x30, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30,
      0x30, 0x5a, 0x17, 0x0d, 0x31, 0x33, 0x30, 0x35, 0x30, 0x39, 0x06, 0x08, 0x2b, 0x06, 0x01,
      0x05, 0x05, 0x07, 0x30, 0x02, 0x86, 0x2d, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x73,
      0x30, 0x39, 0x30, 0x37, 0x06, 0x08, 0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x02, 0x30, 0x44,
      0x06, 0x03, 0x55, 0x1d, 0x20, 0x04, 0x3d, 0x30, 0x3b, 0x30, 0x39, 0x06, 0x0b, 0x60, 0x86,
      0x48, 0x01, 0x86, 0xf8, 0x45, 0x01, 0x07, 0x17, 0x06, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03,
      0x55, 0x04, 0x06, 0x13, 0x02, 0x47, 0x42, 0x31, 0x1b, 0x53, 0x31, 0x17, 0x30, 0x15, 0x06,
      0x03, 0x55, 0x04, 0x0a, 0x13, 0x0e, 0x56, 0x65, 0x72, 0x69, 0x53, 0x69, 0x67, 0x6e, 0x2c,
      0x20, 0x49, 0x6e, 0x63, 0x2e, 0x31, 0x1f, 0x30, 0x1d, 0x06, 0x03, 0x55, 0x04, 0x0b, 0x13,
      0x16, 0x56, 0x65, 0x72, 0x69, 0x53, 0x69, 0x67, 0x6e, 0x20, 0x54, 0x72, 0x75, 0x73, 0x74,
      0x20, 0x4e, 0x65, 0x74, 0x77, 0x6f, 0x72, 0x6b, 0x31, 0x3b, 0x30, 0x39, 0x06, 0x03, 0x55,
      0x04, 0x0b, 0x13, 0x32, 0x54, 0x65, 0x72, 0x6d, 0x73, 0x20, 0x6f, 0x66, 0x20, 0x75, 0x73,
      0x65, 0x20, 0x61, 0x74, 0x20, 0x68, 0x74, 0x74, 0x70, 0x73, 0x3a, 0x2f, 0x2f, 0x77, 0x77,
      0x77, 0x2e, 0x76, 0x65, 0x72, 0x69, 0x73, 0x69, 0x67, 0x6e, 0x2e, 0x63, 0x6f, 0x6d, 0x2f,
      0x72, 0x70, 0x61, 0x20, 0x28, 0x63, 0x29, 0x30, 0x31, 0x10, 0x30, 0x0e, 0x06, 0x03, 0x55,
      0x04, 0x07, 0x13, 0x07, 0x53, 0x31, 0x13, 0x30, 0x11, 0x06, 0x03, 0x55, 0x04, 0x0b, 0x13,
      0x0a, 0x47, 0x31, 0x13, 0x30, 0x11, 0x06, 0x0b, 0x2b, 0x06, 0x01, 0x04, 0x01, 0x82, 0x37,
      0x3c, 0x02, 0x01, 0x03, 0x13, 0x02, 0x55, 0x31, 0x16, 0x30, 0x14, 0x06, 0x03, 0x55, 0x04,
      0x03, 0x14, 0x31, 0x19, 0x30, 0x17, 0x06, 0x03, 0x55, 0x04, 0x03, 0x13, 0x31, 0x1d, 0x30,
      0x1b, 0x06, 0x03, 0x55, 0x04, 0x0f, 0x13, 0x14, 0x50, 0x72, 0x69, 0x76, 0x61, 0x74, 0x65,
      0x20, 0x4f, 0x72, 0x67, 0x61, 0x6e, 0x69, 0x7a, 0x61, 0x74, 0x69, 0x6f, 0x6e, 0x31, 0x12,
      0x31, 0x21, 0x30, 0x1f, 0x06, 0x03, 0x55, 0x04, 0x0b, 0x13, 0x18, 0x44, 0x6f, 0x6d, 0x61,
      0x69, 0x6e, 0x20, 0x43, 0x6f, 0x6e, 0x74, 0x72, 0x6f, 0x6c, 0x20, 0x56, 0x61, 0x6c, 0x69,
      0x64, 0x61, 0x74, 0x65, 0x64, 0x31, 0x14, 0x31, 0x31, 0x30, 0x2f, 0x06, 0x03, 0x55, 0x04,
      0x0b, 0x13, 0x28, 0x53, 0x65, 0x65, 0x20, 0x77, 0x77, 0x77, 0x2e, 0x72, 0x3a, 0x2f, 0x2f,
      0x73, 0x65, 0x63, 0x75, 0x72, 0x65, 0x2e, 0x67, 0x47, 0x6c, 0x6f, 0x62, 0x61, 0x6c, 0x53,
      0x69, 0x67, 0x6e, 0x31, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x43, 0x41, 0x2e, 0x63, 0x72,
      0x6c, 0x56, 0x65, 0x72, 0x69, 0x53, 0x69, 0x67, 0x6e, 0x20, 0x43, 0x6c, 0x61, 0x73, 0x73,
      0x20, 0x33, 0x20, 0x45, 0x63, 0x72, 0x6c, 0x2e, 0x67, 0x65, 0x6f, 0x74, 0x72, 0x75, 0x73,
      0x74, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x63, 0x72, 0x6c, 0x73, 0x2f, 0x73, 0x64, 0x31, 0x1a,
      0x30, 0x18, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x45,
      0x56, 0x49, 0x6e, 0x74, 0x6c, 0x2d, 0x63, 0x63, 0x72, 0x74, 0x2e, 0x67, 0x77, 0x77, 0x77,
      0x2e, 0x67, 0x69, 0x63, 0x65, 0x72, 0x74, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x31, 0x6f, 0x63,
      0x73, 0x70, 0x2e, 0x76, 0x65, 0x72, 0x69, 0x73, 0x69, 0x67, 0x6e, 0x2e, 0x63, 0x6f, 0x6d,
      0x30, 0x39, 0x72, 0x61, 0x70, 0x69, 0x64, 0x73, 0x73, 0x6c, 0x2e, 0x63, 0x6f, 0x73, 0x2e,
      0x67, 0x6f, 0x64, 0x61, 0x64, 0x64, 0x79, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x72, 0x65, 0x70,
      0x6f, 0x73, 0x69, 0x74, 0x6f, 0x72, 0x79, 0x2f, 0x30, 0x81, 0x80, 0x06, 0x08, 0x2b, 0x06,
      0x01, 0x05, 0x05, 0x07, 0x01, 0x01, 0x04, 0x74, 0x30, 0x72, 0x30, 0x24, 0x06, 0x08, 0x2b,
      0x06, 0x01, 0x05, 0x05, 0x07, 0x30, 0x01, 0x86, 0x18, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f,
      0x2f, 0x6f, 0x63, 0x73, 0x70, 0x2e, 0x67, 0x6f, 0x64, 0x61, 0x64, 0x64, 0x79, 0x2e, 0x63,
      0x6f, 0x6d, 0x2f, 0x30, 0x4a, 0x06, 0x08, 0x2b, 0x06, 0x01, 0x05, 0x05, 0x07, 0x30, 0x02,
      0x86, 0x3e, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x63, 0x65, 0x72, 0x74, 0x69, 0x66,
      0x69, 0x63, 0x61, 0x74, 0x65, 0x73, 0x2e, 0x67, 0x6f, 0x64, 0x61, 0x64, 0x64, 0x79, 0x2e,
      0x63, 0x6f, 0x6d, 0x2f, 0x72, 0x65, 0x70, 0x6f, 0x73, 0x69, 0x74, 0x6f, 0x72, 0x79, 0x2f,
      0x67, 0x64, 0x5f, 0x69, 0x6e, 0x74, 0x65, 0x72, 0x6d, 0x65, 0x64, 0x69, 0x61, 0x74, 0x65,
      0x2e, 0x63, 0x72, 0x74, 0x30, 0x1f, 0x06, 0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30, 0x16,
      0x80, 0x14, 0xfd, 0xac, 0x61, 0x32, 0x93, 0x6c, 0x45, 0xd6, 0xe2, 0xee, 0x85, 0x5f, 0x9a,
      0xba, 0xe7, 0x76, 0x99, 0x68, 0xcc, 0xe7, 0x30, 0x27, 0x86, 0x29, 0x68, 0x74, 0x74, 0x70,
      0x3a, 0x2f, 0x2f, 0x63, 0x86, 0x30, 0x68, 0x74, 0x74, 0x70, 0x3a, 0x2f, 0x2f, 0x73];

impl CertificateCompressor {
    pub fn new<I: IntoIterator<Item = CertificateSet>>(common_certificate_sets: I) -> Self {
        Self {
            common_certificate_sets: common_certificate_sets.into_iter()
                .map(|certificate_set| (certificate_set.hash(), certificate_set))
                .collect(),
        }
    }

    fn match_common_certificate(&self,
                                certificate: &Certificate,
                                known_common_certificate_set_hashes: &HashSet<u64>)
                                -> Option<CompressedCertificateEntry> {

        let known_common_certificate_sets = self.common_certificate_sets
            .values()
            .map(|c| (c, c.hash()))
            .filter(|c| known_common_certificate_set_hashes.contains(&c.1));

        for known_common_certificate_set in known_common_certificate_sets {
            if let Some(index) = known_common_certificate_set.0.index_of(certificate) {
                return Some(CompressedCertificateEntry::Common {
                    set_hash: known_common_certificate_set.1,
                    index: index as u32,
                });
            }
        }

        None
    }

    fn to_compressed_entry(&self,
                           certificate: &Certificate,
                           known_common_certificate_set_hashes: &HashSet<u64>,
                           cached_certificate_hashes: &HashSet<u64>)
                           -> CompressedCertificateEntry {
        match_cached_certificate(certificate, cached_certificate_hashes)
            .or_else(|| {
                self.match_common_certificate(certificate, known_common_certificate_set_hashes)
            })
            .unwrap_or(CompressedCertificateEntry::Compressed)
    }

    fn get_known_certificate(&self,
                             compressed_certificate_entry: &CompressedCertificateEntry,
                             cached_certificates: &HashMap<u64, Certificate>)
                             -> Option<Result<Certificate>> {
        match *compressed_certificate_entry {
            CompressedCertificateEntry::Cached { hash } => {
                let cached_certificate = cached_certificates.get(&hash)
                    .cloned()
                    .ok_or_else(|| {
                        Error::from(ErrorKind::UnableToFindCachedCertificateWithHash(hash))
                    });

                Some(cached_certificate)
            }
            CompressedCertificateEntry::Common { set_hash, index } => {
                let index = index as usize;

                let common_certificate = self.common_certificate_sets.get(&set_hash)
                        .ok_or_else(|| Error::from(ErrorKind::UnableToFindCommonCertificateSetWithHash(set_hash)))
                        .and_then(|common_certificate_set|
                        {
                            common_certificate_set.certificate(index)
                                .cloned()
                                .ok_or_else(||Error::from(ErrorKind::UnableToFindCommonCertificateWithIndexInSet(index, set_hash)))
                        });

                Some(common_certificate)
            }
            CompressedCertificateEntry::Compressed => None,
            CompressedCertificateEntry::EndOfList => {
                panic!("end of list entries should not be available at this point")
            }
        }
    }

    pub fn decompress_certificate_chain<R: Read>(&self,
                                                 cached_certificates: &HashMap<u64, Certificate>,
                                                 reader: &mut R)
                                                 -> Result<Vec<Certificate>> {

        let compressed_certificate_entries = deserialize_entries(reader)?;

        // TODO LH A better way to detect we are at the end of the reader
        let uncompressed_length = u32::read(reader)
            .chain_err(|| ErrorKind::UnableToReadCompressedCertificatesUncompressedLength)
            .unwrap_or(0) as usize;

        // Check no clients attempt to allocate a too large buffer
        if uncompressed_length > 128 * 1024 {
            bail!(ErrorKind::CompressedCertificatesUncompressedLengthIsTooLarge(uncompressed_length));
        }

        let mut all_decompressed_certificates =
            Vec::with_capacity(compressed_certificate_entries.len());

        for compressed_certificate_entry in compressed_certificate_entries {
            if let Some(result) =
                   self.get_known_certificate(&compressed_certificate_entry, cached_certificates) {
                all_decompressed_certificates.push(Some(result?));
            } else {
                all_decompressed_certificates.push(None);
            }
        }

        if uncompressed_length > 0 {
            let decompressed_certificates = {
                let known_certificates: Vec<_> =
                    all_decompressed_certificates.iter().filter_map(|x| x.as_ref()).collect();

                decompress_certificates(known_certificates.as_slice(), uncompressed_length, reader)?
            };

            all_decompressed_certificates.as_mut_slice().replace_nones(decompressed_certificates)?;
        }

        let mut certificates = Vec::with_capacity(all_decompressed_certificates.len());

        for decompressed_certificate in all_decompressed_certificates {
            if let Some(certificate) = decompressed_certificate {
                certificates.push(certificate);
            } else {
                bail!(ErrorKind::NotEnoughCompressedCertificates);
            }
        }

        Ok(certificates)
    }

    pub fn compress_certificate_chain<'a, C: IntoIterator<Item = &'a Certificate>, W: Write>
        (&self,
         certificates: C,
         known_common_certificate_set_hashes: &HashSet<u64>,
         cached_certificate_hashes: &HashSet<u64>,
         writer: &mut W)
         -> Result<()> {

        let certificate_compression_entries: Vec<_> = certificates.into_iter()
            .map(|certificate| {
                let compressed_entry = self.to_compressed_entry(certificate,
                                         known_common_certificate_set_hashes,
                                         cached_certificate_hashes);

                (certificate, compressed_entry)
            })
            .collect();

        let uncompressed_length: usize = certificate_compression_entries.iter()
            .filter_map(|&(ref certificate, ref compressed_entry)| match *compressed_entry {
                CompressedCertificateEntry::Compressed => Some(certificate),
                _ => None,
            })
            .map(|certificate| {
                // 4 bytes for the uncompressed length + the number of certificate bytes
                4 + certificate.bytes().len()
            })
            .sum();

        serialize_entries(certificate_compression_entries.iter()
                              .map(|&(_, ref compressed_entry)| compressed_entry),
                          writer)
            ?;

        if uncompressed_length > 0 {
            (uncompressed_length as u32)
                .write(writer)
                .chain_err(|| ErrorKind::UnableToWriteCompressedCertificatesUncompressedLength(uncompressed_length))?;

            let certificate_compressions_by_known = certificate_compression_entries.iter()
                .group_by(|&&(_, ref compressed_entry)| match *compressed_entry {
                    CompressedCertificateEntry::Cached { .. } |
                    CompressedCertificateEntry::Common { .. } => true,
                    CompressedCertificateEntry::Compressed => false,
                    CompressedCertificateEntry::EndOfList => {
                        panic!("end of list entries should not be generated, only read/written")
                    }
                });

            let certificates_by_known: HashMap<bool, Vec<&Certificate>> =
                certificate_compressions_by_known.into_iter()
                    .map(|(key, group)| {
                        (key, group.into_iter().map(|&(certificate, _)| certificate).collect())
                    })
                    .collect();

            // Use all of the known certificates to create the preset dictionary
            let known_certificates = certificates_by_known.get(&true)
                .map(|certificates| certificates.as_slice())
                .unwrap_or(&[]);


            // Only compress the unknown certificates
            let certificates_to_compress = certificates_by_known.get(&false)
                .map(|certificates| certificates.as_slice())
                .unwrap_or(&[]);

            compress_certificates(known_certificates, certificates_to_compress, writer)
                .chain_err(|| ErrorKind::UnableToWriteCompressedCertificates)?;
        }

        Ok(())
    }
}

fn match_cached_certificate(certificate: &Certificate,
                            cached_certificate_hashes: &HashSet<u64>)
                            -> Option<CompressedCertificateEntry> {

    let mut hasher = FnvHasher::default();
    certificate.hash(&mut hasher);
    let certificate_hash = hasher.finish();

    if cached_certificate_hashes.contains(&certificate_hash) {
        Some(CompressedCertificateEntry::Cached { hash: certificate_hash })
    } else {
        None
    }
}

fn zlib_dictionary_for_entries(certificates: &[&Certificate]) -> Vec<u8> {
    let total_size = certificates.iter()
        .map(|c| c.bytes().len())
        .sum::<usize>() + COMMON_SUBSTRINGS.len();

    let mut zlib_dictionary = Vec::with_capacity(total_size);

    // The common and cached certificates should be in reverse order
    let bytes = certificates.iter()
        .rev()
        .flat_map(|c| c.bytes());

    zlib_dictionary.extend(bytes);
    zlib_dictionary.extend(COMMON_SUBSTRINGS);

    zlib_dictionary
}

fn serialize_entries<'a, I: IntoIterator<Item = &'a CompressedCertificateEntry>, W: Write>
    (compressed_certificate_entries: I,
     writer: &mut W)
     -> Result<()> {
    for compressed_entry in compressed_certificate_entries {
        compressed_entry.write(writer)
            .chain_err(|| ErrorKind::UnableToWriteCompressedCertificateEntry)?;
    }

    CompressedCertificateEntry::EndOfList.write(writer)
        .chain_err(|| ErrorKind::UnableToWriteCompressedCertificateEntry)?;

    Ok(())
}

fn deserialize_entries<R: Read>(reader: &mut R) -> Result<Vec<CompressedCertificateEntry>> {
    let mut compressed_certificate_entries = Vec::new();
    loop {
        let compressed_certificate_entry = CompressedCertificateEntry::read(reader)
            .chain_err(|| ErrorKind::UnableToReadCompressedCertificateEntry)?;
        if let CompressedCertificateEntry::EndOfList = compressed_certificate_entry {
            break;
        }
        compressed_certificate_entries.push(compressed_certificate_entry);
    }

    Ok(compressed_certificate_entries)
}

fn decompress_certificates<R: Read>(known_certificates: &[&Certificate],
                                    uncompressed_length: usize,
                                    reader: &mut R)
                                    -> Result<Vec<Certificate>> {

    let mut decompressed = Vec::with_capacity(uncompressed_length);

    let mut decompress = Decompress::new(true);

    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed)
        .chain_err(|| ErrorKind::UnableToReadCompressedCertificates)?;

    let decompress_result = decompress.decompress_vec(&compressed, &mut decompressed, Flush::Finish)
        .chain_err(|| ErrorKind::UnableToDecompressCompressedCertificates)?;

    if let Status::NeedDictionary { .. } = decompress_result {
        assert_eq!(decompressed.len(), 0);

        let zlib_dictionary = zlib_dictionary_for_entries(known_certificates);
        decompress.set_dictionary(&zlib_dictionary);

        let processed_in = decompress.total_in() as usize;

        decompress.decompress_vec(&compressed[processed_in..],
                            &mut decompressed,
                            Flush::Finish)
            .chain_err(|| ErrorKind::UnableToDecompressCompressedCertificates)?;
    }

    let decompressed_length = decompressed.len();

    let mut decompressed_reader = Cursor::new(decompressed);

    let mut certificates = Vec::new();
    while (decompressed_reader.position() as usize) < decompressed_length {
        let uncompressed_length = u32::read(&mut decompressed_reader)
            .chain_err(|| {
                ErrorKind::UnableToReadCompressedCertificateUncompressedLength
            })? as usize;

        let mut certificate_bytes = Vec::with_capacity(uncompressed_length);
        certificate_bytes.resize(uncompressed_length, 0);

        decompressed_reader.read_exact(&mut certificate_bytes)
            .chain_err(|| ErrorKind::UnableToReadCertificateBytes)?;

        certificates.push(Certificate::new(certificate_bytes));
    }

    Ok(certificates)
}

fn compress_chunk<W: Write>(compress: &mut Compress,
                            mut chunk: &[u8],
                            buffer: &mut Vec<u8>,
                            writer: &mut W)
                            -> Result<()> {
    // Loop while there are still bytes to be compressed
    while !chunk.is_empty() {

        let before = compress.total_in();
        let compress_result = compress.compress_vec(chunk, buffer, Flush::None);
        let processed_in = (compress.total_in() - before) as usize;

        match compress_result {
            Status::Ok => {
                // This is fine as it means progress is being made
            }
            Status::StreamEnd => {
                assert!(processed_in > 0,
                        "we should only hit the end of the stream if some bytes were actually \
                         processed")
            }
            _ => panic!("an unexpected error occurred while compressing"),
        };

        // Move past the bytes which have been processed
        chunk = &chunk[processed_in..];

        writer.write_all(buffer)
            .chain_err(|| ErrorKind::UnableToWriteCompressedChunk)?;

        buffer.clear();
    }

    Ok(())
}

fn compress_certificates<W: Write>(known_certificates: &[&Certificate],
                                   certificates: &[&Certificate],
                                   writer: &mut W)
                                   -> Result<()> {

    let zlib_dictionary = zlib_dictionary_for_entries(known_certificates);

    let mut compress = Compress::new(Compression::Default, true);
    compress.set_dictionary(&zlib_dictionary);

    // Vec so we do not allocate a large buffer on the stack
    let mut buffer = Vec::with_capacity(32 * 1024);

    for certificate in certificates {

        let certificate_bytes = certificate.bytes();

        let mut length_bytes = [0u8; 4];

        let uncompressed_length = certificate_bytes.len();
        (uncompressed_length as u32)
            .write(&mut &mut length_bytes[..])
            .and_then(|_| compress_chunk(&mut compress, &length_bytes, &mut buffer, writer))
            .chain_err(|| {
                ErrorKind::UnableToWriteCompressedCertificateUncompressedLength(uncompressed_length)
            })?;

        compress_chunk(&mut compress, certificate_bytes, &mut buffer, writer)
            .chain_err(|| ErrorKind::UnableToWriteCertificateBytes)?;
    }

    // Loop while there are still output bytes to be written
    loop {
        compress.compress_vec(&[], &mut buffer, Flush::Finish);
        if buffer.len() == 0 {
            break;
        }

        writer.write_all(&buffer).chain_err(|| ErrorKind::UnableToWriteCompressedChunk)?;
        buffer.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::certificate_set::CertificateSet;
    use crypto::certificate::Certificate;
    use std::collections::HashSet;
    use std::io::Cursor;

    #[test]
    fn compress_certificate_chain_does_not_compress_common_certificates() {
        // Arrange
        let common_certificate = Certificate::new(vec![1, 2, 3]);
        let common_certificate_set = CertificateSet::from(vec![common_certificate.clone()]);

        let mut known_common_certificate_set_hashes = HashSet::new();
        known_common_certificate_set_hashes.insert(common_certificate_set.hash());

        let certificate_compressor = CertificateCompressor::new(vec![common_certificate_set]);

        // Act
        let mut compressed_bytes = Vec::new();
        let compress_result =
            certificate_compressor.compress_certificate_chain(&[common_certificate],
                                                              &known_common_certificate_set_hashes,
                                                              &HashSet::new(),
                                                              &mut compressed_bytes);

        // Assert
        assert!(compress_result.is_ok());
        assert_eq!(compressed_bytes.len(),
                   14,
                   "1 byte for common entry type + 12 bytes for the common entry + 1 byte for \
                    end of list");
    }

    #[test]
    fn decompress_of_compressed_certificate_chain_decompresses_common_certificates() {
        // Arrange
        let common_certificate = Certificate::new(vec![1, 2, 3]);
        let common_certificate_set = CertificateSet::from(vec![common_certificate.clone()]);

        let mut known_common_certificate_set_hashes = HashSet::new();
        known_common_certificate_set_hashes.insert(common_certificate_set.hash());

        let certificate_compressor = CertificateCompressor::new(vec![common_certificate_set]);

        // Act
        let mut compressed_bytes = Vec::new();
        let input = &[common_certificate];
        let compress_result = certificate_compressor.compress_certificate_chain(input,
                                        &known_common_certificate_set_hashes,
                                        &HashSet::new(),
                                        &mut compressed_bytes);

        assert!(compress_result.is_ok());

        let decompress_result = certificate_compressor.decompress_certificate_chain(&HashMap::new(),
                                        &mut Cursor::new(compressed_bytes));

        // Assert
        assert_eq!(decompress_result.unwrap(), input);
    }

    #[test]
    fn decompress_of_compressed_certificate_chain_decompresses_unknown_certificates() {
        // Arrange
        let unknown_certificate = Certificate::new(vec![1, 2, 3]);

        let certificate_compressor = CertificateCompressor::new(Vec::new());

        // Act
        let mut compressed_bytes = Vec::new();
        let input = &[unknown_certificate];
        let compress_result = certificate_compressor.compress_certificate_chain(input,
                                        &HashSet::new(),
                                        &HashSet::new(),
                                        &mut compressed_bytes);

        assert!(compress_result.is_ok());

        let decompress_result = certificate_compressor.decompress_certificate_chain(&HashMap::new(),
                                        &mut Cursor::new(compressed_bytes));

        // Assert
        assert_eq!(decompress_result.unwrap(), input);
    }

    #[test]
    fn compress_certificate_chain_compresses_unknown_certificates() {
        // Arrange
        let unknown_certificate = Certificate::new(vec![1, 2, 3]);

        let certificate_compressor = CertificateCompressor::new(Vec::new());

        // Act
        let mut compressed_bytes = Vec::new();
        let compress_result =
            certificate_compressor.compress_certificate_chain(&[unknown_certificate],
                                                              &HashSet::new(),
                                                              &HashSet::new(),
                                                              &mut compressed_bytes);

        // Assert
        assert!(compress_result.is_ok());
    }
}