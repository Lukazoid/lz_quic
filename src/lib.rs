#![feature(associated_consts, try_from, never_type, conservative_impl_trait, test, result_expect_err)]
#![recursion_limit="1024"]

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate bitflags;

extern crate byteorder;
extern crate tokio_core;
extern crate tokio_proto;
extern crate rand;
extern crate chrono;
extern crate futures;
extern crate conv;
extern crate hex;
extern crate num;

#[macro_use]
extern crate matches;

#[cfg(test)]
extern crate test;

mod errors;
mod options_slice_ext;
mod writable;
mod readable;
mod primitives;
mod byte_order_primitives;
mod read_quic_primitives;
mod write_quic_primitives;
mod crypto;
mod frames;
mod packets;
mod quic_server_id;
mod quic_version;
mod quic_tag;
mod quic_tag_value_map;
mod quic_connection_id;
mod quic_connection;
mod quic_client;
mod quic_server;
mod quic_codec;