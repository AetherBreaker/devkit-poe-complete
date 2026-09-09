use std::process::ExitCode;

use clap::Parser;

fn main() -> ExitCode {
  let args = devkit_poe_complete::Args::parse();
  devkit_poe_complete::run_real(&args)
}
