use errors::*;
use packets::version_negotiation_packet::VersionNegotiationPacket;
use packets::public_reset_packet::PublicResetPacket;
use packets::regular_packet::RegularPacket;
use std::io::Write;
use std::net::SocketAddr;
use byteorder::WriteBytesExt;
use writable::Writable;

#[derive(Debug, Clone)]
pub enum QuicPacket {
    VersionNegotiation(VersionNegotiationPacket),
    PublicReset(PublicResetPacket),
    Regular(RegularPacket),
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

impl QuicPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<SocketAddr> {
        match self {
            &QuicPacket::VersionNegotiation(ref version_negotiation_packet) => {
                let flags = PUBLIC_FLAG_VERSION | HAS_CONNECTION_ID;
                writer.write_u8(flags.bits())
                    .chain_err(|| ErrorKind::UnableToWriteVersionNegotiationPacket)?;

                version_negotiation_packet.connection_id
                    .write(writer)
                    .chain_err(|| ErrorKind::UnableToWriteVersionNegotiationPacket)?;

                for version in version_negotiation_packet.supported_versions.iter() {
                    version.write(writer)
                        .chain_err(|| ErrorKind::UnableToWriteQuicVersion(*version))
                        .chain_err(|| ErrorKind::UnableToWriteVersionNegotiationPacket)?;
                }
            }
            &QuicPacket::PublicReset(ref public_reset_packet) => {
                let flags = PUBLIC_FLAG_RESET | HAS_CONNECTION_ID;
                writer.write_u8(flags.bits())
                    .chain_err(|| ErrorKind::UnableToWritePublicResetPacket)?;

                public_reset_packet.connection_id
                    .write(writer)
                    .chain_err(|| {
                        ErrorKind::UnableToWriteQuicConnectionId(public_reset_packet.connection_id)
                    })
                    .chain_err(|| ErrorKind::UnableToWritePublicResetPacket)?;


                // TODO LH Write the Quic Tag and the Tag value map

            }
            &QuicPacket::Regular(ref regular_packet) => {
                for frame in regular_packet.frames.iter() {
                    frame.write(writer)
                        .chain_err(|| ErrorKind::UnableToWriteRegularPacket)?;
                }
            }
        }

        unimplemented!()
    }
}