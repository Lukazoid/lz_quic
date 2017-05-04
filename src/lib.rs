#![recursion_limit="1024"]

extern crate byteorder;
extern crate tokio_core;
extern crate tokio_io;
extern crate rand;
extern crate chrono;
extern crate conv;
extern crate hex;
extern crate num;
extern crate fnv;
extern crate flate2;
extern crate itertools;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate futures;

#[macro_use]
extern crate lazy_static;
extern crate openssl;
extern crate ring;
extern crate untrusted;
#[cfg(test)] #[macro_use] extern crate matches;

mod crate_info {
    include!(concat!(env!("OUT_DIR"), "/version.rs"));
}

mod errors;
mod options_slice_ext;
mod writable;
mod readable;
mod primitives;
mod byte_order_primitives;
mod read_quic_primitives;
mod write_quic_primitives;
mod diversification_nonce;
mod crypto;
mod frames;
mod packets;
mod server_id;
mod version;
mod tag;
mod tag_value_map;
mod connection_id;
mod data_stream;
mod stream_id;
mod session;
mod client_configuration;
mod client;
mod server;