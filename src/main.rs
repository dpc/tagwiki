// CHANGEME
//! rust-bin-template

use anyhow::{Context, Result};
use log::debug;
use std::path::Path;
use structopt::StructOpt;

/// Command line options
mod cli;

fn open(path: &Path) -> Result<()> {
    let _f =
        std::fs::File::open(path).with_context(|| format!("Failed to open: {}", path.display()))?;

    Ok(())
}
fn main() -> Result<()> {
    env_logger::init();
    debug!("Parsing command line arguments");
    let opts = cli::Opts::from_args();

    open(&opts.path)?;

    Ok(())
}
