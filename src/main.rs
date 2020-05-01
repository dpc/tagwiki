//! tagwiki

use anyhow::Result;
use log::info;
use structopt::StructOpt;
use warp::{path::FullPath, Filter};

/// Command line options
mod cli;
/// Page
mod page;

mod index;

/// Utils
mod util;

async fn handler(path: FullPath) -> Result<String, std::convert::Infallible> {
    let tags: Vec<_> = path
        .as_str()
        .split('/')
        .map(|t| t.trim())
        .filter(|t| t != &"")
        .collect();
    Ok(format!("Path: {:?}", tags))
}

fn start(opts: &cli::Opts) -> Result<()> {
    let handler = warp::path::full().and_then(handler);
    let serve = warp::serve(handler).run(([127, 0, 0, 1], opts.port));
    info!("Listening on port {}", opts.port);

    tokio::runtime::Runtime::new().unwrap().block_on(serve);

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let opts = cli::Opts::from_args();

    start(&opts)?;

    Ok(())
}
