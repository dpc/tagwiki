//! Command line options handling

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Rust application template")] // CHANGEME
#[structopt(global_setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct Opts {
    pub path: PathBuf,
}
