use std::path::PathBuf;

use partially::Partial;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Game {
    #[default]
    Ck3,
    Vic3,
    Imperator,
    Eu5,
}

#[derive(Debug, Partial, Default)]
#[partially(derive(Deserialize))]
#[partially(attribute(serde(deny_unknown_fields)))]
pub struct Config {
    pub game: Game,
    ck3_dir: Option<PathBuf>,
    vic3_dir: Option<PathBuf>,
    imperator_dir: Option<PathBuf>,
    eu5_dir: Option<PathBuf>,
}
