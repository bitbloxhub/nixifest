mod cli;
mod crd;
mod emit;
mod input;
mod kubernetes;
mod model;
mod normalize;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};
use emit::emit_module;
use input::collect_inputs;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (output_path, output) = match cli.command {
        Command::Kubernetes(args) => {
            let inputs = collect_inputs(&args.input)?;
            let resources = kubernetes::load(&inputs)?;
            (args.output, emit_module(&resources)?)
        }
        Command::Crd(args) => {
            let inputs = collect_inputs(&args.input)?;
            let resources = crd::load(&inputs)?;
            (args.output, emit_module(&resources)?)
        }
    };

    if output_path.as_os_str() == "-" {
        print!("{output}");
    } else {
        std::fs::write(output_path, output)?;
    }

    Ok(())
}
