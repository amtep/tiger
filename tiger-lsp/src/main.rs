use std::fs::File;

use anyhow::Result;
use log::error;
use simplelog::{CombinedLogger, Config, LevelFilter, WriteLogger};

use tiger_lsp::{ConnectionLogger, main_loop::main_loop};

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        // TODO: find a proper home for the logfile
        WriteLogger::new(LevelFilter::Trace, Config::default(), File::create("/tmp/lsp.log")?),
        ConnectionLogger::new(LevelFilter::Warn),
    ])?;

    if let Err(e) = main_loop() {
        error!("fatal error: {e}");
    }
    Ok(())
}
