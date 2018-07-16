use bytes::Bytes;
use chrono::UTC;
use conv::ValueFrom;
use packets::{IncomingPacket, OutgoingPacket, PacketHeader, PacketHeaderReadContext};
use protocol::{Readable, Writable};
use smallvec::SmallVec;
use std::io::{Cursor, Result as IoResult};
use std::net::SocketAddr;
use tokio_core::net::UdpCodec;

#[derive(Debug, Clone, Default)]
pub struct PacketCodec;

impl UdpCodec for PacketCodec {
    type In = SmallVec<[IncomingPacket; 2]>;
    type Out = OutgoingPacket;

    fn decode(&mut self, src: &SocketAddr, mut buf: &[u8]) -> IoResult<Self::In> {
        let received_at = UTC::now();

        let mut incoming_packets = SmallVec::new();

        while !buf.is_empty() {
            trace!("decoding incoming packet");

            let mut buf_cursor = Cursor::new(buf);

            let packet_header = PacketHeader::read_with_context(
                &mut buf_cursor,
                &PacketHeaderReadContext {
                    has_connection_id: true,
                },
            )?;

            let data_start_index = usize::value_from(buf_cursor.position()).expect(
                "the buf cursor should not exceed the value which can be stored by a usize",
            );

            let data_end_index = packet_header
                .payload_length()
                .map(|l| usize::value_from(u64::from(l)).unwrap())
                .unwrap_or(buf.len());

            let data = Bytes::from(&buf[data_start_index..data_end_index]);

            let incoming_packet = IncomingPacket {
                source_address: *src,
                packet_header,
                data,
                received_at,
            };

            debug!("decoded incoming packet {:?}", incoming_packet);

            incoming_packets.push(incoming_packet);

            buf = &buf[data_end_index..];
        }

        Ok(incoming_packets)
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
