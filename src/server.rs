use errors::*;
use tokio_core::net::{UdpSocket, UdpCodec};
use tokio_core::reactor::Handle;
use std::collections::HashMap;
use std::sync::RwLock;
use protocol::ConnectionId;
use session::Session;
use std::net::SocketAddr;
use futures::{Future, Stream};
use futures::future::{self, Empty};
use futures::stream::{self, BoxStream};
use std::error::Error as StdError;

#[derive(Debug)]
pub struct Server {
    udp_socket: UdpSocket,
}

impl Server {
    pub fn bind(addr: SocketAddr, handle: &Handle) -> Result<Self> {
        let udp_socket = UdpSocket::bind(&addr, handle)
            .chain_err(|| ErrorKind::FailedToBindToUdpSocket(addr))?;
            
        Ok(Self { udp_socket: udp_socket })
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.udp_socket.local_addr()
            .chain_err(||ErrorKind::FailedToGetLocalAddress)
    }

    // pub fn incoming(self) -> BoxStream<Session, Error> {
    //     stream::empty::<(), Error>()
    //         .map(move |_|{
    //             let connection_id = ConnectionId::generate(&mut ::rand::thread_rng());
    //             Session::new_server(connection_id)
    //         })
    //         .boxed()
    // }
}


// struct PacketHeaderCodec{ }

// impl UdpCodec for PacketHeaderCodec {
//     type In = IncomingPacket;
//     type Out = OutgoingPacket;

//     fn decode(&mut self, src: &SocketAddr, buf: &[u8]) -> IoResult<Self::In> {

//         Ok(IncomingPacket{})
//         // let packet_result = Packet::from_bytes(buf);

//         // if let Ok(ref packet) = packet_result {

//         // }

//         // // TODO LH Once https://github.com/brson/error-chain/issues/134 is resolved include the cause directly
//         // packet_result.map_err(|e| IoError::new(IoErrorKind::Other, e.to_string()))
//     }

//     fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
//         msg.packet.write(buf).expect("there should be no error writing to a vector");

//         msg.remote
//     }

        
// }