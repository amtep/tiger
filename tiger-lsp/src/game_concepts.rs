use std::fs;
use std::path::Path;

use crate::parse::GAME_CONCEPTS_PARSER;
use crate::util::{HashMap, HashSet};

type ConceptFileMap = HashMap<String, HashSet<String>>;

#[derive(Debug)]
pub struct GameConcepts {
    game: ConceptFileMap,
    mod_: (ConceptFileMap, ConceptFileMap),
}

impl GameConcepts {
    pub fn new() -> Self {
        Self { game: HashMap::new(), mod_: (HashMap::new(), HashMap::new()) }
    }

    pub fn load_game(game_dir_path: &Path) -> Result<Self, std::io::Error> {
        let game = Self::load_game_concepts(game_dir_path)?;
        Ok(Self { game, mod_: (HashMap::new(), HashMap::new()) })
    }

    pub fn load_mod(&mut self, workspace_path: &Path) -> Result<(), std::io::Error> {
        // * if workspace_path does not contain the common/game_concepts folder,
        // * simply return Ok(()) rather than an error.
        if !workspace_path.join("common/game_concepts").exists() {
            return Ok(());
        }

        let mod_ = Self::load_mod_game_concepts(workspace_path, &self.game)?;
        self.mod_ = mod_;
        Ok(())
    }

    pub fn contains(&self, concept: &str) -> bool {
        self.mod_.0.values().any(|h| h.contains(concept))
            || (!self.mod_.1.values().any(|h| h.contains(concept))
                && self.game.values().any(|h| h.contains(concept)))
    }

    fn load_game_concepts(game_dir_path: &Path) -> Result<ConceptFileMap, std::io::Error> {
        let game_concepts_path = game_dir_path.join("common/game_concepts");

        let mut results = HashMap::new();

        for concept_file in fs::read_dir(game_concepts_path)? {
            let concept_file = concept_file?.path();
            if concept_file.extension() == Some(std::ffi::OsStr::new("txt")) {
                let key = concept_file.file_name().unwrap().to_string_lossy().into_owned();

                let concept_content = fs::read_to_string(concept_file)?;
                let concept_content =
                    concept_content.strip_prefix('\u{feff}').unwrap_or(&concept_content);

                if let Ok(concepts) = GAME_CONCEPTS_PARSER.parse(concept_content) {
                    results.insert(key, HashSet::from_iter(concepts));
                }
            }
        }

        Ok(results)
    }

    fn load_mod_game_concepts(
        workspace_path: &Path,
        game: &ConceptFileMap,
    ) -> Result<(ConceptFileMap, ConceptFileMap), std::io::Error> {
        let game_concepts_path = workspace_path.join("common/game_concepts");

        let mut added = HashMap::new();
        let mut removed = HashMap::new();

        for concept_file in fs::read_dir(game_concepts_path)? {
            let concept_file = concept_file?.path();
            if concept_file.extension() == Some(std::ffi::OsStr::new("txt")) {
                let key = concept_file.file_name().unwrap().to_string_lossy().into_owned();

                let concept_content = fs::read_to_string(concept_file)?;
                let concept_content =
                    concept_content.strip_prefix('\u{feff}').unwrap_or(&concept_content);

                match GAME_CONCEPTS_PARSER.parse(concept_content) {
                    Ok(concepts) => {
                        if let Some(concepts) = game.get(&key) {
                            removed.insert(key.clone(), concepts.clone());
                        }
                        added.insert(key, HashSet::from_iter(concepts));
                    }
                    Err(err) => {
                        log::trace!("failed to parse mod game concept: {err}");
                    }
                }
            }
        }

        Ok((added, removed))
    }
}
