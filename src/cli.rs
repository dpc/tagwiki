//! Command line options handling

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "TagWiki")]
#[structopt(global_setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct Opts {
    pub path: PathBuf,

    #[structopt(long = "port", default_value = "3030")]
    pub port: u16,
}
