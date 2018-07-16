use errors::*;
use frames::Frame;
use packets::{OutgoingPacket, Packet};

#[derive(Debug)]
pub struct PacketPacker {
    frames: Vec<Frame>,
}

impl PacketPacker {
    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn pack_packet(&mut self) -> Result<OutgoingPacket> {
        unimplemented!()
    }
}
