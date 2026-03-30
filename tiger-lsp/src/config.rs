use std::path::PathBuf;

use partially::Partial;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Game {
    Ck3,
    Vic3,
    Imperator,
    Eu5,
}

#[derive(Debug, Partial)]
#[partially(derive(Deserialize))]
pub struct Config {
    game: Game,
    game_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self { game: Game::Ck3, game_dir: None }
    }
}
