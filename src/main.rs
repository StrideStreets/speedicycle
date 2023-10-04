extern crate speedicycle;
use anyhow::Error;
use clap::Parser;

use speedicycle::{make_route_from_dimacs, CLIArgs};

fn main() -> Result<(), Error> {
    let args: CLIArgs = CLIArgs::parse();

    make_route_from_dimacs::<u32, f64, u32>(args, false).map(|_i| ())
}
