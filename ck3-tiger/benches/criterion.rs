use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tiger_lib::{Everything, Fileset};

static CONFIG_PATH: &str = "../benches/ck3.toml";

// Sample Config File:
// vanilla_dir = "..."
// modfile_dir = "..."
// modfile_paths = ["...", "..."]
// sample_size = 30

#[derive(Deserialize)]
struct Config {
    vanilla_dir: String,
    modfile_dir: Option<String>,
    modfile_paths: Vec<String>,
    sample_size: Option<usize>,
}

fn workspace_path(s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    if p.is_relative() {
        PathBuf::from("..").join(p)
    } else {
        p
    }
}

fn bench_multiple(c: &mut Criterion) {
    let content = fs::read_to_string(CONFIG_PATH).unwrap();
    let config: Config = toml::from_str(&content).unwrap();
    let mut modfile_paths =
        config.modfile_paths.iter().map(|p| workspace_path(p)).collect::<Vec<_>>();

    if let Some(modfile_dir) = config.modfile_dir {
        let modfile_dir = workspace_path(&modfile_dir);
        let iter =
            fs::read_dir(modfile_dir).unwrap().filter_map(|entry| entry.ok()).filter_map(|entry| {
                entry.file_name().to_string_lossy().ends_with(".mod").then(|| entry.path())
            });
        modfile_paths.extend(iter);
    }

    let vanilla_dir = PathBuf::from(config.vanilla_dir);

    let mut group = c.benchmark_group("benchmark");
    group.sample_size(config.sample_size.unwrap_or(10));
    for (index, modfile_path) in modfile_paths.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("mods", format!("{}. {}", index + 1, modfile_path.display())),
            modfile_path,
            |b, modfile_path| {
                b.iter(|| bench_mod(&vanilla_dir, modfile_path.clone()));
            },
        );
    }

    group.finish();
}

fn bench_mod(vanilla_dir: &Path, modfile_path: PathBuf) {
    let fileset = Fileset::builder(Some(vanilla_dir)).with_modfile(modfile_path).unwrap();
    let mut everything = Everything::new(fileset, None, None, None).unwrap();
    everything.load_all();
    everything.validate_all();
    everything.check_rivers();
}

criterion_group!(benches, bench_multiple);
criterion_main!(benches);
