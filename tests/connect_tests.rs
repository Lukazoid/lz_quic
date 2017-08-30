extern crate lz_quic;
extern crate futures;
extern crate tokio_core;

use std::thread;
use lz_quic::{Client, ClientConfiguration, Error as QuicError, Server, ServerId};
use tokio_core::reactor::Core;
use futures::future;
use futures::Future;
use futures::sync::oneshot;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

#[test]
pub fn client_connecting_to_server() {
    let any_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);

    let (server_bound, when_server_bound) = oneshot::channel();

    let server_future = future::lazy(move || {
        let mut server_event_loop = Core::new().expect("error creating server event loop");

        let server = Server::bind(any_address, &server_event_loop.handle())
            .expect("error binding server to IP/Port");

        server_bound
            .send(server.local_addr().expect("failed to get server address"))
            .expect("error marking server as bound");

        let server_future = future::ok::<_, QuicError>(());
        server_event_loop.run(server_future)
    });

    let when_client_connected = when_server_bound.then(move |result| {
        let server_addr = result.expect("binding the server should never get cancelled");

        let mut client_event_loop = Core::new().expect("error creating client event loop");

        let server_id = ServerId::new("localhost".to_owned(), server_addr.port());

        let client_configuration = ClientConfiguration::default();

        let when_client_connected = Client::connect(
            server_addr,
            server_id,
            client_configuration,
            &client_event_loop.handle(),
        ).map(|client| ());

        client_event_loop.run(when_client_connected)
    });

    server_future
        .join(when_client_connected)
        .wait()
        .expect("error occurred in server or client");
}
