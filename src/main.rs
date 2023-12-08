#![feature(slice_group_by)]
#![feature(future_join)]
#![feature(async_closure)]

extern crate base64;
extern crate chrono;
extern crate clap;
extern crate futures;
extern crate home;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate tokio;
extern crate uuid;

use clap::{Parser, Subcommand};

mod cmd;
pub mod azure;
pub mod kubernetes;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    RefreshAks(cmd::refresh::RefreshAksArgs),
    Pim(cmd::pim::PimArgs),
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::RefreshAks(args) => cmd::refresh::refresh(args),
        Commands::Pim(args) => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .build()
                .unwrap()
                .block_on(cmd::pim::pim(args));
        }
    }
}
