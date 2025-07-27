use serde::Deserialize;
use std::{fs, path::PathBuf, sync::LazyLock};

static CONFIG_PATH: &str = "../benches/ck3.toml";

// Sample Config File:
// vanilla_dir = "..."
// modfile_dir = "..."
// modfile_paths = ["...", "..."]

#[derive(Deserialize)]
struct Config {
    vanilla_dir: String,
    modfile_dir: Option<String>,
    modfile_paths: Vec<String>,
}

static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let content = fs::read_to_string(CONFIG_PATH).unwrap();
    toml::from_str(&content).unwrap()
});
static MODFILE_PATHS: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
    let mut modfile_paths = CONFIG.modfile_paths.iter().map(PathBuf::from).collect::<Vec<_>>();
    if let Some(modfile_dir) = &CONFIG.modfile_dir {
        let iter =
            fs::read_dir(modfile_dir).unwrap().filter_map(|entry| entry.ok()).filter_map(|entry| {
                entry.file_name().to_string_lossy().ends_with(".mod").then(|| entry.path())
            });
        modfile_paths.extend(iter);
    }
    modfile_paths
});

pub fn bench_mods<'a>() -> impl Iterator<Item = (&'a str, &'a PathBuf)> {
    MODFILE_PATHS.iter().map(move |modfile_path| (CONFIG.vanilla_dir.as_str(), modfile_path))
}
