use conv::ValueFrom;
use errors::*;
use packets::{LongHeader, LongHeaderPacketType, PacketNumber, PartialPacketNumber, ShortHeader,
              VersionNegotiationPacket};
use protocol::{ConnectionId, Readable, VarInt, Version, Writable};
use std::io::{Read, Write};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PacketHeader {
    Long(LongHeader),
    Short(ShortHeader),
    VersionNegotiation(VersionNegotiationPacket),
}

impl PacketHeader {
    pub fn destination_connection_id(&self) -> Option<ConnectionId> {
        match self {
            PacketHeader::Long(long_header) => long_header.destination_connection_id,
            PacketHeader::Short(short_header) => short_header.destination_connection_id,
            PacketHeader::VersionNegotiation(version_negotiation) => {
                version_negotiation.destination_connection_id
            }
        }
    }

    pub fn source_connection_id(&self) -> Option<ConnectionId> {
        match self {
            PacketHeader::Long(long_header) => long_header.source_connection_id,
            PacketHeader::Short(_) => None,
            PacketHeader::VersionNegotiation(version_negotiation) => {
                version_negotiation.source_connection_id
            }
        }
    }

    pub fn payload_length(&self) -> Option<VarInt> {
        match self {
            PacketHeader::Long(long_header) => Some(long_header.payload_length),
            PacketHeader::Short(_) => None,
            PacketHeader::VersionNegotiation(_) => Some(0u32.into()),
        }
    }
}

bitflags!(
    flags PacketHeaderBitFlags : u8 {
        const KEY_PHASE                             = 0x40,

        const LONG_HEADER                           = 0x80,
        const LONG_PACKET_TYPE_INITIAL              = 0x7F,
        const LONG_PACKET_TYPE_RETRY                = 0x7E,
        const LONG_PACKET_TYPE_HANDSHAKE            = 0x7D,
        const LONG_PACKET_TYPE_ZERO_RTT_PROTECTED   = 0x7C,
    }
);

fn read_connection_id<R: Read>(reader: &mut R, length_flags: u8) -> Result<Option<ConnectionId>> {
    if length_flags == 0 {
        Ok(None)
    } else {
        // Non-zero encoded lengths are increased by 3 to get the full length of the connection ID
        let length = length_flags + 3;

        let connection_id = ConnectionId::read(&mut reader.take(length.into()))?;

        Ok(Some(connection_id))
    }
}

#[derive(Debug)]
pub struct PacketHeaderReadContext {
    pub has_connection_id: bool,
}

impl Readable for PacketHeader {
    type Context = PacketHeaderReadContext;
    fn read_with_context<R: Read>(reader: &mut R, context: &Self::Context) -> Result<Self> {
        trace!("reading packet header");

        trace!("reading packet header flags");
        let flags = u8::read(reader).chain_err(|| ErrorKind::FailedToReadPacketHeaderFlags)?;
        let flags = PacketHeaderBitFlags::from_bits_truncate(flags);
        debug!("read packet header flags {:?}", flags);

        let packet_header = if flags.intersects(LONG_HEADER) {
            let version = Version::read(reader)?;

            let dcil_scil = u8::read(reader)?;
            let destination_connection_id_flags = (dcil_scil >> 4) & 0xf;
            let source_connection_id_flags = dcil_scil & 0xf;

            let destination_connection_id =
                read_connection_id(reader, destination_connection_id_flags)?;
            let source_connection_id = read_connection_id(reader, source_connection_id_flags)?;

            if version.is_version_negotiation() {
                let supported_versions = Version::collect(reader)?;

                PacketHeader::VersionNegotiation(VersionNegotiationPacket {
                    destination_connection_id,
                    source_connection_id,
                    supported_versions,
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
                let payload_length = VarInt::read(reader)?;

                let partial_packet_number = PartialPacketNumber::read(reader)?;

                PacketHeader::Long(LongHeader {
                    packet_type,
                    version,
                    destination_connection_id,
                    source_connection_id,
                    payload_length,
                    partial_packet_number,
                })
            }
        } else {
            let key_phase = flags.intersects(KEY_PHASE);

            let destination_connection_id = if context.has_connection_id {
                Some(ConnectionId::read(reader)?)
            } else {
                None
            };

            let partial_packet_number = PartialPacketNumber::read(reader)?;
            PacketHeader::Short(ShortHeader {
                key_phase,
                destination_connection_id,
                partial_packet_number,
            })
        };

        debug!("read packet header {:?}", packet_header);

        Ok(packet_header)
    }
}

impl Writable for PacketHeader {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing packet header {:?}", self);

        match self {
            PacketHeader::VersionNegotiation(version_negotiation) => {
                let flags = LONG_HEADER;

                flags
                    .bits()
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePacketHeaderFlags)?;

                Version::NEGOTIATION.write(writer)?;

                // TODO LH compact the connection id lengths
                let mut dcil_scil = 0u8;
                if version_negotiation.destination_connection_id.is_some() {
                    dcil_scil |= 0xf << 4;
                }
                if version_negotiation.source_connection_id.is_some() {
                    dcil_scil |= 0xf;
                }
                dcil_scil.write(writer)?;

                version_negotiation.destination_connection_id.write(writer)?;
                version_negotiation.source_connection_id.write(writer)?;

                version_negotiation.supported_versions.write(writer)?;
            }
            PacketHeader::Long(long_header) => {
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

                long_header.version.write(writer)?;

                // TODO LH compact the connection id lengths
                let mut dcil_scil = 0u8;
                if long_header.destination_connection_id.is_some() {
                    dcil_scil |= 0xf << 4;
                }
                if long_header.source_connection_id.is_some() {
                    dcil_scil |= 0xf;
                }
                dcil_scil.write(writer)?;

                long_header.destination_connection_id.write(writer)?;
                long_header.source_connection_id.write(writer)?;
                long_header.payload_length.write(writer)?;
                long_header.partial_packet_number.write(writer)?;
            }
            PacketHeader::Short(short_header) => {
                let mut flags = PacketHeaderBitFlags::empty();
                if short_header.key_phase {
                    flags |= KEY_PHASE;
                }

                (flags.bits() | 0x30)
                    .write(writer)
                    .chain_err(|| ErrorKind::FailedToWritePacketHeaderFlags)?;

                short_header.destination_connection_id.write(writer)?;

                short_header.partial_packet_number.write(writer)?;
            }
        }

        debug!("written packet header {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PacketHeader;
    use packets::{LongHeader, LongHeaderPacketType, PacketHeaderReadContext, ShortHeader,
                  VersionNegotiationPacket};
    use protocol::{self, ConnectionId, Version};

    #[test]
    pub fn read_write_version_negotiation_packet_header() {
        let version_negotiation_packet = VersionNegotiationPacket {
            destination_connection_id: Some(ConnectionId::generate().unwrap()),
            source_connection_id: Some(ConnectionId::generate().unwrap()),
            supported_versions: vec![Version::DRAFT_IETF_08],
        };
        let packet_header = PacketHeader::VersionNegotiation(version_negotiation_packet);

        protocol::test_write_read_with_context(
            &packet_header,
            &PacketHeaderReadContext {
                has_connection_id: true,
            },
        ).unwrap();
    }

    #[test]
    pub fn read_write_long_packet_header() {
        let long_header = LongHeader {
            packet_type: LongHeaderPacketType::Handshake,
            destination_connection_id: Some(ConnectionId::generate().unwrap()),
            source_connection_id: Some(ConnectionId::generate().unwrap()),
            version: Version::DRAFT_IETF_08,
            partial_packet_number: 5u8.into(),
            payload_length: 654234u32.into(),
        };
        let packet_header = PacketHeader::Long(long_header);

        protocol::test_write_read_with_context(
            &packet_header,
            &PacketHeaderReadContext {
                has_connection_id: true,
            },
        ).unwrap();
    }

    #[test]
    pub fn read_write_short_packet_header() {
        let short_header = ShortHeader {
            destination_connection_id: Some(ConnectionId::generate().unwrap()),
            partial_packet_number: 3421u16.into(),
            key_phase: true,
        };
        let packet_header = PacketHeader::Short(short_header);

        protocol::test_write_read_with_context(
            &packet_header,
            &PacketHeaderReadContext {
                has_connection_id: true,
            },
        ).unwrap();
    }
}
