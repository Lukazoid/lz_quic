use errors::*;
use protocol::ConnectionId;
use packets::{InboundPacketStore, Packet, PacketCodec};
use tokio_core::net::{UdpFramed, UdpSocket};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::net::SocketAddr;
use std::collections::HashMap;
use futures::stream::Stream;
use futures::sink::Sink;

struct DebuggableFramed(UdpFramed<PacketCodec>);

impl Debug for DebuggableFramed {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "UdpFramed<PacketCodec>")
    }
}

#[derive(Debug)]
pub struct PacketDispatcher {
    inbound_packet_stores: HashMap<ConnectionId, InboundPacketStore>,
    framed: DebuggableFramed,
}

impl PacketDispatcher {
    pub fn new(udp_socket: UdpSocket) -> Self {
        let framed = udp_socket.framed(PacketCodec::default());

        Self {
            inbound_packet_stores: HashMap::new(),
            framed: DebuggableFramed(framed),
        }
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.framed
            .0
            .get_ref()
            .local_addr()
            .chain_err(|| ErrorKind::FailedToGetLocalAddress)
    }

    // pub fn incoming_stream(
    //     &self,
    //     connection_id: ConnectionId,
    // ) -> Box<Stream<Item = Packet, Error = Error> + Send> {

    //     unimplemented!()
    // }

    // pub fn outgoing_sink(&self, connection_id: ConnectionId) -> Box<Sink<Packet, Error> + Send> {

    //     unimplemented!()
    // }
}
