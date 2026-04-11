use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::parse::LOCA_KEY_TEXTS_PARSER;
use crate::util::HashMap;

type KeyTextMap = HashMap<String, (Option<u16>, String)>;
type FileKeysMap = HashMap<String, Vec<String>>;

#[derive(Debug)]
pub struct KeyTexts {
    game: (KeyTextMap, FileKeysMap),
}

impl KeyTexts {
    pub fn new() -> Self {
        Self { game: (HashMap::new(), HashMap::new()) }
    }

    pub fn load_game(game_dir_path: &Path) -> Result<Self, std::io::Error> {
        let game = Self::load_game_key_texts(game_dir_path)?;
        Ok(Self { game })
    }

    pub fn get(&self, key: &str) -> Option<&(Option<u16>, String)> {
        self.game.0.get(key)
    }

    fn load_key_texts<F: FnMut(String, Vec<(String, Option<u16>, String)>)>(
        root_path: &Path,
        mut f: F,
    ) -> Result<(), std::io::Error> {
        // FIXME: check per language
        let localization_path = root_path.join("localization/english");

        for loca_file in WalkDir::new(localization_path) {
            let loca_file = loca_file?;
            let loca_file = loca_file.path();
            if loca_file.extension() == Some(std::ffi::OsStr::new("yml")) {
                let key = loca_file.file_name().unwrap().to_string_lossy().into_owned();

                let loca_content = fs::read_to_string(loca_file)?;
                let loca_content = loca_content.strip_prefix('\u{feff}').unwrap_or(&loca_content);

                match LOCA_KEY_TEXTS_PARSER.parse(loca_content) {
                    Ok(key_texts) => f(key, key_texts),
                    Err(err) => {
                        log::trace!("failed to parse game loca key texts:\n{key}: {err}");
                    }
                }
            }
        }

        Ok(())
    }

    fn load_game_key_texts(
        game_dir_path: &Path,
    ) -> Result<(KeyTextMap, FileKeysMap), std::io::Error> {
        let mut key_text_map = HashMap::new();
        let mut file_keys_map = HashMap::new();

        Self::load_key_texts(game_dir_path, |file, key_texts| {
            let keys = key_texts.iter().map(|(k, _, _)| k.clone()).collect();
            for (key, version, text) in key_texts {
                key_text_map.insert(key, (version, text));
            }
            file_keys_map.insert(file, keys);
        })?;

        Ok((key_text_map, file_keys_map))
    }
}
