use tokio_core::net::UdpCodec;
use packets::{IncomingPacket, OutgoingPacket, PacketHeader};
use std::io::{Cursor, Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::net::SocketAddr;
use protocol::{Readable, Writable};
use chrono::UTC;
use bytes::Bytes;

#[derive(Debug, Clone, Default)]
pub struct PacketCodec;

impl UdpCodec for PacketCodec {
    type In = IncomingPacket;
    type Out = OutgoingPacket;

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        trace!("decoding incoming packet");

        let mut buf_cursor = Cursor::new(buf);
        let packet_header = PacketHeader::read(&mut buf_cursor)?;

        // The data is everything after the header in the datagram
        let data = Bytes::from(&buf[buf_cursor.position() as usize..]);

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
