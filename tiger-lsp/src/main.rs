use std::io::LineWriter;

use anyhow::Result;
use log::error;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};

use tiger_lsp::{ConnectionLogger, main_loop::main_loop};

fn main() -> Result<()> {
    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Never,
        ),
        ConnectionLogger::new(LevelFilter::Warn),
    ];

    if let Some(dir) = directories::ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        && std::fs::create_dir_all(dir.cache_dir()).is_ok()
        && let Ok(file) =
            std::fs::File::create(dir.cache_dir().join(format!("{}.log", env!("CARGO_PKG_NAME"))))
    {
        loggers.push(WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            LineWriter::new(file),
        ));
    }

    CombinedLogger::init(loggers)?;

    if let Err(e) = main_loop() {
        error!("fatal error: {e}");
    }
    Ok(())
}
