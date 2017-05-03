extern crate vergen;

use vergen::{vergen, OutputFns};

fn main() {
    vergen(OutputFns::all());
}