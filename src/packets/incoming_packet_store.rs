use packets::{IncomingPacket, PacketNumber};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct IncomingPacketStore {
    pending_packets: BTreeMap<PacketNumber, IncomingPacket>
}

impl IncomingPacketStore {
    pub fn packet(&mut self, packet_number: PacketNumber) -> Option<IncomingPacket> {
        self.pending_packets.remove(&packet_number)
    }
}