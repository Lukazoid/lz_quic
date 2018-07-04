extern crate env_logger;
extern crate lz_quic;
extern crate tokio_core;

use lz_quic::{Client, ClientConfiguration, ServerId};
use std::net::ToSocketAddrs;
use tokio_core::reactor::Core;

fn main() {
    env_logger::init();
    let mut core = Core::new().expect("failed to create event loop");
    let server_id = ServerId::new("test.privateoctopus.com".to_owned(), 4433);

    let when_connected = Client::connect(
        "test.privateoctopus.com:4433"
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap(),
        server_id,
        ClientConfiguration::default(),
        &core.handle(),
    );

    core.run(when_connected).unwrap();
}
