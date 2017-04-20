#![feature(associated_consts, try_from, never_type, conservative_impl_trait, test, result_expect_err)]
#![recursion_limit="1024"]

extern crate byteorder;
extern crate tokio_core;
extern crate tokio_proto;
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

#[macro_use]
#[cfg(test)]
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
mod server_id;
mod version;
mod tag;
mod tag_value_map;
mod connection_id;
