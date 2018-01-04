use errors::*;
use packets::PacketNumber;
use protocol::Version;
use std::net::SocketAddr;
use frames::Frame;

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
    Regular { frames: Vec<Frame> },
}

impl PacketContent {
    pub fn frames(&self) -> Option<&[Frame]> {
        match *self {
            PacketContent::Regular { frames: ref frames } => {
                Some(frames.as_slice())
            }
            _ => None
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Packet {
    pub packet_number: PacketNumber,
    pub content: PacketContent,
}

