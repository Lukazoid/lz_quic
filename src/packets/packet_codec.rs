use bytes::Bytes;
use chrono::UTC;
use conv::ValueFrom;
use packets::{IncomingPacket, OutgoingPacket, PacketHeader, PacketHeaderReadContext};
use protocol::{Readable, Writable};
use std::io::{Cursor, Result as IoResult};
use std::net::SocketAddr;
use tokio_core::net::UdpCodec;

#[derive(Debug, Clone, Default)]
pub struct PacketCodec;

impl UdpCodec for PacketCodec {
    type In = IncomingPacket;
    type Out = OutgoingPacket;

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        trace!("decoding incoming packet");

        let mut buf_cursor = Cursor::new(buf);

        // TODO LH Actually determine if the short header should have a packet number
        let packet_header = PacketHeader::read_with_context(
            &mut buf_cursor,
            &PacketHeaderReadContext {
                has_connection_id: true,
            },
        )?;

        // The data is everything after the header in the datagram

        let data_start_index = usize::value_from(buf_cursor.position())
            .expect("the buf cursor should not exceed the value which can be stored by a usize");

        let data = Bytes::from(&buf[data_start_index..]);

        let incoming_packet = IncomingPacket {
            source_address: *src,
            packet_header: packet_header,
            data: data,
            received_at: UTC::now(),
        };

        debug!("decoded incoming packet {:?}", incoming_packet);

        Ok(incoming_packet)
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        trace!("encoding outgoing packet {:?}", msg);

        msg.packet_header
            .write(buf)
            .and(msg.data.write(buf))
            .expect("there should be no error writing the public header to an in-memory buffer");

        debug!("encoded outgoing packet {:?}", msg);

        msg.destination_address
    }
}
