use std::net::SocketAddr;
use std::io::{Result as IoResult, Write};
use tokio_core::net::UdpCodec;
use byteorder::{WriteBytesExt, LittleEndian};
use frames::quic_frame::QuicFrame;
use packets::quic_packet::QuicPacket;

#[derive(Default)]
pub struct QuicCodec {
}

impl UdpCodec for QuicCodec {
    type In = QuicPacket;
    type Out = QuicPacket;

    fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {
        unimplemented!()
    }

    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
        msg.write(buf).unwrap()
    }
}