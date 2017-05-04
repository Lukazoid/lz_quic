extern crate vergen;

use vergen::{vergen, OutputFns};

fn main() {
    vergen(OutputFns::all()).expect("there should be no issue generating the version info");
}

