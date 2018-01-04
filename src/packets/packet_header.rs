use errors::*;
use protocol::{ConnectionId, Version};
use packets::{LongHeader, LongHeaderPacketType, PacketNumber, PartialPacketNumber,
              PartialPacketNumberLength, ShortHeader, VersionNegotiationPacket};
use protocol::{Readable, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PacketHeader {
    Long(LongHeader),
    Short(ShortHeader),
    VersionNegotiation(VersionNegotiationPacket),
}

bitflags!(
    flags PacketHeaderBitFlags : u8 {
        const KEY_PHASE                             = 0x20,
        const OMIT_CONNECTION_ID                    = 0x40,

        const SHORT_PACKET_TYPE_ONE_BYTE            = 0x1F,
        const SHORT_PACKET_TYPE_TWO_BYTES           = 0x1E,
        const SHORT_PACKET_TYPE_FOUR_BYTES          = 0x1D,

        const LONG_HEADER                           = 0x80,
        const LONG_PACKET_TYPE_INITIAL              = 0x7F,
        const LONG_PACKET_TYPE_RETRY                = 0x7E,
        const LONG_PACKET_TYPE_HANDSHAKE            = 0x7D,
        const LONG_PACKET_TYPE_ZERO_RTT_PROTECTED   = 0x7C,
    }
);

impl Readable for PacketHeader {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading packet header");

        trace!("reading packet header flags");
        let flags = u8::read(reader).chain_err(|| ErrorKind::FailedToReadPacketHeaderFlags)?;
        let flags = PacketHeaderBitFlags::from_bits_truncate(flags);
        debug!("read packet header flags {:?}", flags);

        let packet_header = if flags.intersects(LONG_HEADER) {
            let connection_id = ConnectionId::read(reader)?;
            let version = Version::read(reader)?;
            if version.is_version_negotiation() {
                let supported_versions = Version::collect(reader)?;

                PacketHeader::VersionNegotiation(VersionNegotiationPacket {
                    connection_id: connection_id,
                    supported_versions: supported_versions,
                })
            } else {
                let packet_type_flags =
                    PacketHeaderBitFlags::from_bits_truncate(flags.bits() & 0x7F);
                let packet_type = match packet_type_flags {
                    LONG_PACKET_TYPE_INITIAL => LongHeaderPacketType::Initial,
                    LONG_PACKET_TYPE_RETRY => LongHeaderPacketType::Retry,
                    LONG_PACKET_TYPE_HANDSHAKE => LongHeaderPacketType::Handshake,
                    LONG_PACKET_TYPE_ZERO_RTT_PROTECTED => LongHeaderPacketType::ZeroRttProtected,
                    _ => bail!(ErrorKind::InvalidLongHeaderPacketType(
                        packet_type_flags.bits()
                    )),
                };
                let packet_number = PacketNumber::read(reader)?;

                PacketHeader::Long(LongHeader {
                    packet_type: packet_type,
                    connection_id: connection_id,
                    version: version,
                    packet_number: packet_number,
                })
            }
        } else {
            let omit_connection_id = flags.intersects(OMIT_CONNECTION_ID);
            let key_phase = flags.intersects(KEY_PHASE);
            let packet_type_flags = PacketHeaderBitFlags::from_bits_truncate(flags.bits() & 0x1F);

            let connection_id = if omit_connection_id {
                None
            } else {
                Some(ConnectionId::read(reader)?)
            };

            let partial_packet_number_length = match packet_type_flags {
                SHORT_PACKET_TYPE_ONE_BYTE => PartialPacketNumberLength::OneByte,
                SHORT_PACKET_TYPE_TWO_BYTES => PartialPacketNumberLength::TwoBytes,
                SHORT_PACKET_TYPE_FOUR_BYTES => PartialPacketNumberLength::FourBytes,
                _ => bail!(ErrorKind::InvalidShortHeaderPacketType(
                    packet_type_flags.bits()
                )),
            };

            let partial_packet_number =
                PartialPacketNumber::read(reader, partial_packet_number_length)?;
            PacketHeader::Short(ShortHeader {
                key_phase: key_phase,
                connection_id: connection_id,
                partial_packet_number: partial_packet_number,
            })
        };

        debug!("read packet header {:?}", packet_header);

        Ok(packet_header)
    }
}

impl Writable for PacketHeader {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing packet header {:?}", self);

        match *self {
            PacketHeader::VersionNegotiation(ref version_negotiation) => {
                let flags = LONG_HEADER;

                flags
                    .bits()
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePacketHeaderFlags)?;

                version_negotiation.connection_id.write(writer)?;
                Version::NEGOTIATION.write(writer)?;
                version_negotiation.supported_versions.write(writer)?;
            }
            PacketHeader::Long(ref long_header) => {
                let mut flags = LONG_HEADER;
                flags |= match long_header.packet_type {
                    LongHeaderPacketType::Initial => LONG_PACKET_TYPE_INITIAL,
                    LongHeaderPacketType::Retry => LONG_PACKET_TYPE_RETRY,
                    LongHeaderPacketType::Handshake => LONG_PACKET_TYPE_HANDSHAKE,
                    LongHeaderPacketType::ZeroRttProtected => LONG_PACKET_TYPE_ZERO_RTT_PROTECTED,
                };

                flags
                    .bits()
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePacketHeaderFlags)?;

                long_header.connection_id.write(writer)?;
                long_header.version.write(writer)?;
                long_header.packet_number.write(writer)?;
            }
            PacketHeader::Short(ref short_header) => {
                let mut flags = PacketHeaderBitFlags::empty();
                if short_header.connection_id.is_none() {
                    flags |= OMIT_CONNECTION_ID;
                }
                if short_header.key_phase {
                    flags |= KEY_PHASE;
                }

                flags |= match short_header.partial_packet_number {
                    PartialPacketNumber::OneByte(_) => SHORT_PACKET_TYPE_ONE_BYTE,
                    PartialPacketNumber::TwoBytes(_) => SHORT_PACKET_TYPE_TWO_BYTES,
                    PartialPacketNumber::FourBytes(_) => SHORT_PACKET_TYPE_FOUR_BYTES,
                };

                flags
                    .bits()
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePacketHeaderFlags)?;

                if let Some(connection_id) = short_header.connection_id {
                    connection_id.write(writer)?;
                }

                short_header.partial_packet_number.write(writer)?;
            }
        }

        debug!("written packet header {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use packets::{LongHeader, LongHeaderPacketType, PacketNumber, PartialPacketNumber,
                  ShortHeader, VersionNegotiationPacket};
    use protocol::{ConnectionId, Readable, Version, Writable};
    use rand;
    use super::PacketHeader;

    #[test]
    pub fn read_write_version_negotiation_packet_header() {
        let version_negotiation_packet = VersionNegotiationPacket {
            connection_id: ConnectionId::generate(&mut rand::thread_rng()),
            supported_versions: vec![Version::DRAFT_IETF_08],
        };
        let packet_header = PacketHeader::VersionNegotiation(version_negotiation_packet);

        let mut bytes = Vec::new();

        packet_header.write_to_vec(&mut bytes);

        let read_packet_header = PacketHeader::from_bytes(&bytes[..]).unwrap();

        assert_eq!(packet_header, read_packet_header);
    }

    #[test]
    pub fn read_write_long_packet_header() {
        let long_header = LongHeader {
            packet_type: LongHeaderPacketType::Handshake,
            connection_id: ConnectionId::generate(&mut rand::thread_rng()),
            version: Version::DRAFT_IETF_08,
            packet_number: PacketNumber::from(5u64),
        };
        let packet_header = PacketHeader::Long(long_header);

        let mut bytes = Vec::new();

        packet_header.write_to_vec(&mut bytes);

        let read_packet_header = PacketHeader::from_bytes(&bytes[..]).unwrap();

        assert_eq!(packet_header, read_packet_header);
    }

    #[test]
    pub fn read_write_short_packet_header() {
        let short_header = ShortHeader {
            connection_id: Some(ConnectionId::generate(&mut rand::thread_rng())),
            partial_packet_number: PartialPacketNumber::TwoBytes(3421),
            key_phase: true,
        };
        let packet_header = PacketHeader::Short(short_header);

        let mut bytes = Vec::new();

        packet_header.write_to_vec(&mut bytes);

        let read_packet_header = PacketHeader::from_bytes(&bytes[..]).unwrap();

        assert_eq!(packet_header, read_packet_header);
    }
}
