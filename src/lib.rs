#![recursion_limit="1024"]
#![cfg_attr(feature = "unstable", feature(test))]

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
mod handshake;
mod frames;
mod crypto;
mod packets;

mod session;
pub use self::session::Session;

mod data_stream;
pub use self::data_stream::DataStream;

mod client_configuration;
pub use self::client_configuration::ClientConfiguration;

mod client;
pub use self::client::Client;

mod server;
pub use self::server::Server;