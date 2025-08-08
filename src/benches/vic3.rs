use serde::Deserialize;
use std::{fs, path::PathBuf, sync::LazyLock};

static CONFIG_PATH: &str = "benches/vic3.toml";

// Sample Config File:
// vanilla_dir = "..."
// mod_dir = "..."
// mod_paths = ["...", "..."]

#[derive(Deserialize)]
struct Config {
    vanilla_dir: String,
    mod_dir: Option<String>,
    mod_paths: Vec<String>,
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let content = fs::read_to_string(CONFIG_PATH).unwrap();
    toml::from_str(&content).unwrap()
});
static MOD_PATHS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let mut mod_paths = CONFIG.mod_paths.iter().map(PathBuf::from).collect::<Vec<_>>();
    if let Some(mod_dir) = &CONFIG.mod_dir {
        let iter =
            fs::read_dir(mod_dir).unwrap().filter_map(|entry| entry.ok()).filter_map(|entry| {
                entry.path().join(".metadata/metadata.json").is_file().then(|| entry.path())
            });
        mod_paths.extend(iter);
    }
    mod_paths
});

pub fn bench_mods<'a>() -> impl Iterator<Item = (&'a str, &'a PathBuf)> {
    MOD_PATHS.iter().map(move |mod_path| (CONFIG.vanilla_dir.as_str(), mod_path))
}
