use errors::*;
use packets::outbound_packet::OutboundPacket;

#[derive(Debug)]
pub struct PacketPacker {}

impl PacketPacker {
    pub fn pack_packet(&self) -> Result<OutboundPacket> {
        unimplemented!()
    }
}
