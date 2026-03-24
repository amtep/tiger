use std::fs::File;

use anyhow::Result;
use log::error;
use simplelog::{Config, LevelFilter, WriteLogger};

use tiger_lsp::main_loop::main_loop;

fn main() -> Result<()> {
    // TODO: find a proper home for the logfile
    WriteLogger::init(LevelFilter::Trace, Config::default(), File::create("/tmp/lsp.log")?)?;
    if let Err(e) = main_loop() {
        error!("fatal error: {e}");
    }
    Ok(())
}
