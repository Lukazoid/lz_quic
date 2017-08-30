use errors::*;
use protocol::ConnectionId;
use packets::{PacketCodec, Packet};
use futures::stream::{BoxStream, Stream};
use futures::sink::BoxSink;
use tokio_core::net::{UdpFramed, UdpSocket};

pub struct PacketDispatcher {
    // packet_stores: Hashmap<ConnectionId, InboundPacketStore>
    framed: UdpFramed<PacketCodec>,
}

impl PacketDispatcher {
    pub fn new(udp_socket: UdpSocket) -> Self {
        let framed = udp_socket.framed(PacketCodec::default());
        
        Self {
            framed: framed
        }
    }

    pub fn incoming_stream(&self, connection_id: ConnectionId) -> BoxStream<Packet, Error> {

        unimplemented!()
    }

    pub fn outgoing_sink(&self, connection_id: ConnectionId) -> BoxSink<Packet, Error> {

        unimplemented!()
    }
}