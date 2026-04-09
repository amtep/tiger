use std::path::Path;

use crate::parse::load_game_concepts;
use crate::util::HashSet;

#[derive(Debug)]
pub struct GameConcepts {
    game: HashSet<String>,
}

impl GameConcepts {
    pub fn new() -> Self {
        Self { game: HashSet::new() }
    }

    pub fn load(game_dir_path: &Path) -> Result<Self, std::io::Error> {
        let game = load_game_concepts(game_dir_path)?;
        Ok(Self { game })
    }

    pub fn contains(&self, concept: &str) -> bool {
        self.game.contains(concept)
    }
}
