use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "nixifest-typegen")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Kubernetes(InputArgs),
    Crd(InputArgs),
}

#[derive(Debug, Args)]
pub struct InputArgs {
    #[arg(long = "input", required = true)]
    pub input: Vec<PathBuf>,

    #[arg(short, long)]
    pub output: PathBuf,
}
