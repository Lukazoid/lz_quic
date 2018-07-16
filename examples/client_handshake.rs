extern crate env_logger;
extern crate futures;
extern crate lz_quic;
extern crate tokio_core;

use futures::Future;
use lz_quic::Result as QuicResult;
use lz_quic::{Client, ClientConfiguration, ServerId};
use std::net::ToSocketAddrs;
use tokio_core::reactor::Core;

fn main() {
    env_logger::init();

    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> QuicResult<()> {
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
    ).map(|_| ());

    core.run(when_connected)
}
