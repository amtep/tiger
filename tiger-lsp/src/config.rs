use std::path::PathBuf;

use partially::Partial;
use serde_derive::Deserialize;

use tiger_tables::game::Game;

#[derive(Debug, Partial)]
#[partially(derive(Deserialize))]
#[partially(attribute(serde(deny_unknown_fields)))]
pub struct Config {
    pub game: Game,
    ck3_dir: Option<PathBuf>,
    vic3_dir: Option<PathBuf>,
    imperator_dir: Option<PathBuf>,
    eu5_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self { game: Game::Ck3, ck3_dir: None, vic3_dir: None, imperator_dir: None, eu5_dir: None }
    }
}
