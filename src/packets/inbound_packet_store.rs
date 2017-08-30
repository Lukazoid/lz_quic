use packets::{InboundPacket, PacketNumber};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct InboundPacketStore {
    pending_packets: BTreeMap<PacketNumber, InboundPacket>
}

impl InboundPacketStore {
    pub fn packet(&mut self, packet_number: PacketNumber) -> Option<InboundPacket> {
        self.pending_packets.remove(&packet_number)
    }
}