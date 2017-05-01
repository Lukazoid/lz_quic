use errors::*;
use packets::public_header::PublicHeader;
use packets::packet_number::PacketNumber;
use std::io::{Read, Write};
use version::Version;
use std::net::SocketAddr;
use frames::frame::Frame;

#[derive(Debug)]
pub enum PacketContent {
    VersionNegotiation { supported_versions: Vec<Version> },
    PublicReset {
        nonce_proof: u64,
        rejected_packet_number: PacketNumber,
        client_address: Option<SocketAddr>,
    },
    Regular { frames: Vec<Frame> },
}

#[derive(Debug)]
pub struct Packet {
    pub packet_number: PacketNumber,
    pub content: PacketContent,
}

