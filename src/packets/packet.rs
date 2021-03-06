use bytes::Bytes;
use errors::*;
use frames::{Frame, InitialPacketFrame};
use packets::PacketNumber;
use protocol::Version;
use std::net::SocketAddr;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PacketContent {
    VersionNegotiation {
        supported_versions: Vec<Version>,
    },
    PublicReset {
        nonce_proof: u64,
        rejected_packet_number: PacketNumber,
        client_address: Option<SocketAddr>,
    },
    Regular {
        frames: Vec<Frame>,
    },
    Initial {
        token: Bytes,
        frames: Vec<InitialPacketFrame>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Packet {
    pub packet_number: PacketNumber,
    pub content: PacketContent,
}
