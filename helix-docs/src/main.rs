#![allow(dead_code)] // Scaffolded code - types defined but not yet wired up

use clap::Parser;

mod cli;
mod config;
mod domain;
mod error;
mod ports;
mod services;

use cli::Cli;
use error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli::run(cli)
}
