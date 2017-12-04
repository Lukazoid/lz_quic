use tokio_core::net::UdpCodec;
use packets::{InboundPacket, OutboundPacket, PublicHeader};
use std::io::{Cursor, Error as IoError, ErrorKind as IoErrorKind, Result as IoResult};
use std::net::SocketAddr;
use protocol::{Readable, Writable};
use chrono::UTC;

#[derive(Debug, Clone, Default)]
pub struct PacketCodec;

impl UdpCodec for PacketCodec {
    type In = InboundPacket;
    type Out = OutboundPacket;

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        trace!("decoding inbound packet");

        let mut buf_cursor = Cursor::new(buf);
        let public_header = PublicHeader::read(&mut buf_cursor)
            .map_err(|e| IoError::new(IoErrorKind::InvalidData, e.to_string()))?;

        // The data is everything after the header in the datagram
        let data = buf[buf_cursor.position() as usize..].to_vec();

        let inbound_packet = InboundPacket {
            source_address: *src,
            public_header: public_header,
            data: data,
            received_at: UTC::now(),
        };

        debug!("decoded inbound packet {:?}", inbound_packet);

        Ok(inbound_packet)
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        trace!("encoding outbound packet {:?}", msg);

        msg.public_header
            .write(buf)
            .and(msg.data.write(buf))
            .expect("there should be no error writing the public header to an in-memory buffer");

        debug!("encoded outbound packet {:?}", msg);

        msg.destination_address
    }
}
