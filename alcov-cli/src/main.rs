use crate::dump::Dump;
use clap::{Parser, Subcommand};

pub mod dump;
pub mod merge;

#[derive(Clone, Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    Dump(Dump),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dump(dump) => {
            dump.run().unwrap();
        }
    }
}
