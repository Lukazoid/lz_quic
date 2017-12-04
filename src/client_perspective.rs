use {Perspective, DataStream};
use tokio_core::net::UdpSocket;

#[derive(Debug)]
pub struct ClientPerspective {}

impl ClientPerspective {
    pub(crate) fn new(udp_socket: UdpSocket) -> Self {
        Self {}
    }
}

impl Perspective for ClientPerspective {
    fn open_stream(&self) -> DataStream<Self>{
        unimplemented!()
    }
}
