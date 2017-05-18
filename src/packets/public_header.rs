use errors::*;
use protocol::{ConnectionId, Version};
use crypto::DiversificationNonce;
use packets::{PartialPacketNumber, PartialPacketNumberLength};
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug)]
pub struct PublicHeader {
    pub reset_flag: bool,
    pub connection_id: Option<ConnectionId>,
    pub version: Option<Version>,
    pub diversification_nonce: Option<DiversificationNonce>,
    pub partial_packet_number: PartialPacketNumber,
}

bitflags!(
    flags PublicHeaderBitFlags : u8 {
        const PUBLIC_FLAG_VERSION       = 0x01,
        const PUBLIC_FLAG_RESET         = 0x02,
        const HAS_DIVERSIFICATION_NONCE = 0x04,
        const HAS_CONNECTION_ID         = 0x08,
        const TWO_BYTE_PACKET_NUMBER    = 0x10,
        const FOUR_BYTE_PACKET_NUMBER   = 0x20,
        const SIX_BYTE_PACKET_NUMBER    = 0x30,
    }
);
impl Readable for PublicHeader {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let flags = u8::read(reader).chain_err(|| ErrorKind::UnableToReadPublicPacketHeaderFlags)?;
        let flags = PublicHeaderBitFlags::from_bits_truncate(flags);

        let reset_flag = flags.intersects(PUBLIC_FLAG_VERSION);

        let connection_id = if flags.intersects(HAS_CONNECTION_ID) {
            Some(ConnectionId::read(reader)?)
        } else {
            None
        };

        let version = if flags.intersects(PUBLIC_FLAG_VERSION) {
            Some(Version::read(reader)?)
        } else {
            None
        };

        let diversification_nonce = if flags.intersects(HAS_DIVERSIFICATION_NONCE) {
            Some(DiversificationNonce::read(reader)?)
        } else {
            None
        };

        let partial_packet_number_length = if flags.intersects(SIX_BYTE_PACKET_NUMBER) {
            PartialPacketNumberLength::SixBytes
        } else if flags.intersects(FOUR_BYTE_PACKET_NUMBER) {
            PartialPacketNumberLength::FourBytes
        } else if flags.intersects(TWO_BYTE_PACKET_NUMBER) {
            PartialPacketNumberLength::TwoBytes
        }else{
            PartialPacketNumberLength::OneByte
        };

        let partial_packet_number = PartialPacketNumber::read(reader, partial_packet_number_length)?;

        Ok(Self {
            reset_flag: reset_flag,
            connection_id: connection_id,
            version: version,
            diversification_nonce: diversification_nonce,
            partial_packet_number: partial_packet_number
        })
    }
}

impl Writable for PublicHeader {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut flags = PublicHeaderBitFlags::empty();
        if self.version.is_some() {
            flags |= PUBLIC_FLAG_VERSION;
        }
        if self.reset_flag {
            flags |= PUBLIC_FLAG_RESET;
        }
        if self.diversification_nonce.is_some() {
            flags |= HAS_DIVERSIFICATION_NONCE;
        }
        if self.connection_id.is_some() {
            flags |= HAS_CONNECTION_ID;
        }
        
        flags |= match self.partial_packet_number {
            PartialPacketNumber::OneByte(_) => PublicHeaderBitFlags::empty(),
            PartialPacketNumber::TwoBytes(_) => TWO_BYTE_PACKET_NUMBER,
            PartialPacketNumber::FourBytes(_) => FOUR_BYTE_PACKET_NUMBER,
            PartialPacketNumber::SixBytes(_) => SIX_BYTE_PACKET_NUMBER,
        };

        flags.bits().write(writer).chain_err(|| ErrorKind::UnableToWritePublicPacketHeaderFlags)?;

        if let Some(connection_id) = self.connection_id {
            connection_id.write(writer)?;
        }

        if let Some(version) = self.version {
            version.write(writer)?;
        }

        if let Some(ref diversification_nonce) = self.diversification_nonce {
            diversification_nonce.write(writer)?;
        }

        self.partial_packet_number.write(writer)?;

        Ok(())
    }
}