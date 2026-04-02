use anyhow::Result;
use log::error;
use simplelog::{ColorChoice, CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode};

use tiger_lsp::{ConnectionLogger, main_loop::main_loop};

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Never,
        ),
        ConnectionLogger::new(LevelFilter::Warn),
    ])?;

    if let Err(e) = main_loop() {
        error!("fatal error: {e}");
    }
    Ok(())
}
