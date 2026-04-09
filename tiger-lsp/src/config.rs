use std::path::{Path, PathBuf};

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

impl Config {
    pub fn game_dir(&self) -> Option<&Path> {
        match self.game {
            Game::Ck3 => self.ck3_dir.as_deref(),
            Game::Vic3 => self.vic3_dir.as_deref(),
            Game::Imperator => self.imperator_dir.as_deref(),
            Game::Eu5 => self.eu5_dir.as_deref(),
        }
    }
}
