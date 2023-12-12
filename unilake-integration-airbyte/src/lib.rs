pub mod airbyte;
pub mod backend;
pub mod docker;
pub mod error;
pub mod flatten;
pub mod model;
pub mod parquet;
pub mod schema;
pub mod transform;
pub mod utils;

use std::ffi::OsString;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;

use clap::Parser;
use clap::Subcommand;
use duct::cmd;

pub(crate) fn exec_stream<F>(command: &str, args: &Vec<&str>, mut x: F)
where
    F: FnMut(Result<String, Error>) -> ExecResult,
{
    let exec_cmd = cmd(command, args);
    let reader = exec_cmd.stderr_to_stdout().reader().unwrap();
    let lines = BufReader::new(reader).lines();
    for line in lines {
        if let ExecResult::Break = x(line) {
            break;
        }
    }
}
pub(crate) enum ExecResult {
    Continue,
    Break,
}

#[derive(Debug, Parser)]
#[command(about = "Unilake - Airbyte protocol integration", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// build target Airbyte image
    #[command(name = "build")]
    Build {
        /// source image to build an image for
        source_image: OsString,
        /// target image name and tag
        target_image: OsString,
        /// optional, base image to use when building
        base_image: Option<OsString>,
    },
    #[command(subcommand)]
    Airbyte(AirbyteCommand),
}

#[derive(Debug, Subcommand)]
pub enum AirbyteCommand {
    /// outputs the json configuration specification
    Spec,
    /// checks the config can be used to connect
    #[command(arg_required_else_help = true)]
    Check {
        /// path to the json configuration file
        #[arg(long)]
        config: OsString,
    },
    /// outputs a catalog describing the source's schema
    #[command(arg_required_else_help = true)]
    Discover {
        /// path to the json configuration file
        #[arg(long)]
        config: OsString,
    },
    /// reads the source and outputs messages to STDOUT
    #[command(arg_required_else_help = true)]
    Read {
        /// path to the json configuration file
        #[arg(long)]
        config: OsString,
        /// path to the catalog used to determine which data to read
        #[arg(long)]
        catalog: OsString,
        /// path to the json-encoded state file
        #[arg(long)]
        state: Option<OsString>,
    },
}

pub enum MeltanoCommand {
    Invoke,
}
