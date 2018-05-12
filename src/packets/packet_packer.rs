use errors::*;
use packets::{OutgoingPacket, Packet};

#[derive(Debug)]
pub struct PacketPacker;

impl PacketPacker {
    pub fn pack_packet(&mut self) -> Result<OutgoingPacket> {
        unimplemented!()
    }
}
