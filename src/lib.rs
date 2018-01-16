#![recursion_limit="1024"]
#![cfg_attr(feature = "unstable", feature(test))]

#![allow(dead_code)]

extern crate byteorder;
extern crate tokio_core;
extern crate tokio_io;
extern crate rand;
extern crate chrono;
extern crate conv;
extern crate hex;
extern crate num;
extern crate lz_fnv;
extern crate flate2;
extern crate itertools;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate bitflags;
#[macro_use] extern crate futures;
#[macro_use] extern crate lazy_static;
extern crate openssl;
extern crate ring;
extern crate webpki;
extern crate untrusted;
#[macro_use] extern crate matches;
extern crate smallvec;
extern crate time;
#[cfg(test)] extern crate webpki_roots;
extern crate lz_diet;
extern crate extprim;
extern crate binary_tree;
extern crate bytes;
#[macro_use] extern crate log;
extern crate debugit;
extern crate bimap;

#[cfg(all(feature = "unstable", test))]
extern crate test;

mod crate_info {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

mod errors;
pub use self::errors::{Error, Result, ErrorKind};

mod protocol;
pub use self::protocol::ServerId;

mod primitives;
mod utils;
mod frames;
mod packets;

mod connection_map;
use self::connection_map::ConnectionMap;

mod connection_termination_mode;
pub use self::connection_termination_mode::ConnectionTerminationMode;

mod new_data_streams;
pub use self::new_data_streams::NewDataStreams;

mod connection;
use self::connection::Connection;

mod data_stream;
pub use self::data_stream::DataStream;

mod stream_map;
use self::stream_map::StreamMap;

mod client_configuration;
pub use self::client_configuration::ClientConfiguration;

mod new_data_stream;
pub use self::new_data_stream::NewDataStream;

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