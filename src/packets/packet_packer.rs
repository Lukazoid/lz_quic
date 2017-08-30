use errors::*;
use packets::{Packet, OutboundPacket};

#[derive(Debug)]
pub struct PacketPacker;

impl PacketPacker {
    pub fn pack_packet(&self, packet: Packet) -> Result<OutboundPacket> {


        unimplemented!()
    }
}
