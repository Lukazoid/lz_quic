#![recursion_limit = "1024"]
#![cfg_attr(feature = "unstable", feature(test))]
#![allow(dead_code)]

extern crate binary_tree;
#[macro_use]
extern crate bitflags;
extern crate byteorder;
extern crate bytes;
extern crate chrono;
extern crate conv;
extern crate debugit;
#[macro_use]
extern crate error_chain;
extern crate flate2;
#[macro_use]
extern crate futures;
extern crate hex;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate lz_diet;
extern crate lz_fnv;
extern crate lz_shared_udp;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate matches;
extern crate num;
extern crate rand;
extern crate ring;
extern crate rustls;
#[macro_use]
extern crate smallvec;
extern crate time;
extern crate tokio_core;
#[macro_use]
extern crate tokio_io;
extern crate tokio_rustls;
extern crate untrusted;
extern crate webpki;
#[cfg(test)]
extern crate webpki_roots;

#[cfg(all(feature = "unstable", test))]
extern crate test;

mod crate_info {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

macro_rules! async_io {
    ($e:expr) => {
        match $e {
            ::futures::Async::Ready(result) => result,
            ::futures::Async::NotReady => return Err(::std::io::ErrorKind::WouldBlock.into()),
        }
    };
}

mod errors;
pub use self::errors::{Error, ErrorKind, Result};

mod protocol;
pub use self::protocol::ServerId;

mod crypto;
mod frames;
mod packets;
mod primitives;
mod utils;

mod connection_map;
use self::connection_map::{AddressConnectionIds, ConnectionMap};

mod connection_termination_mode;
pub use self::connection_termination_mode::ConnectionTerminationMode;

mod new_data_streams;
pub use self::new_data_streams::NewDataStreams;

mod connection;
use self::connection::Connection;

mod shared_connection;
use self::shared_connection::SharedConnection;

mod data_stream;
pub use self::data_stream::DataStream;

mod stream_state;
use self::stream_state::StreamState;

mod stream_map;
use self::stream_map::{StreamMap, StreamMapEntry};

mod client_configuration;
pub use self::client_configuration::ClientConfiguration;

mod server_configuration;
pub use self::server_configuration::ServerConfiguration;

mod new_client;
pub use self::new_client::NewClient;

mod perspective;
use self::perspective::Perspective;

mod client_perspective;
pub use self::client_perspective::ClientPerspective;

mod server_perspective;
pub use self::server_perspective::ServerPerspective;

mod client;
pub use self::client::Client;

mod remote_client;
pub use self::remote_client::RemoteClient;

mod new_remote_clients;
pub use self::new_remote_clients::NewRemoteClients;

mod server;
pub use self::server::Server;
