#![forbid(unsafe_code)]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

/// Utilities for loading data collections from NeXus / HDF5
mod collection;

use clap::{Parser, Subcommand};
use collection::Collection;
use ptree::print_tree;
use std::path::PathBuf;
use tracing::Level;

/// Tristimg performs binning of event mode data, generating images
#[derive(Debug, Parser)]
struct Cli {
    /// The minimum log level which should be produced
    #[clap(long, env = "LOG_LEVEL", default_value_t=Level::INFO)]
    log_level: Level,
    /// Various tristimg commands
    #[clap(subcommand)]
    command: Commands,
}

/// Various tristimg commands
#[derive(Debug, Clone, Subcommand)]
enum Commands {
    /// Run one of the debugging tools, producing (hopefully) useful intermediate information
    Debug {
        /// The debug command to run
        #[clap(subcommand)]
        subcommand: DebugCommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum DebugCommands {
    /// Display infromation about the datasets which make up the data collection
    Datasets(DebugDatasetsCommand),
}

#[derive(Debug, Clone, Parser)]
struct DebugDatasetsCommand {
    /// The path to the nexus file which describes the data collection
    nexus_path: PathBuf,
    /// The width to which the count field in data file names should be padded
    #[clap(long, env = "DATA_FILE_PADDING", default_value_t = 6)]
    data_file_padding: usize,
}

fn main() {
    _ = dotenvy::dotenv();
    let args = Cli::parse();
    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    match args.command {
        Commands::Debug {
            subcommand: DebugCommands::Datasets(args),
        } => {
            let collection =
                Collection::from_nexus(args.nexus_path, args.data_file_padding).unwrap();
            print_tree(&collection.as_tree()).unwrap();
        }
    }
}
