extern crate speedicycle;
use anyhow::Error;
use clap::Parser;

use speedicycle::{make_route_from_dimacs, CLIArgs};

fn main() -> Result<(), Error> {
    let args: CLIArgs = CLIArgs::parse();

    make_route_from_dimacs(args, false).map(|_i| ())
}
