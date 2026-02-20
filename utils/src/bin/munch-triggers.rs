use std::borrow::ToOwned;
use std::collections::HashSet;
use std::fs::{File, read_dir, read_to_string, write};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::{Parser, ValueEnum};
use encoding_rs::UTF_8;
use encoding_rs_rw::DecodingReader;
use strum_macros::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum, Display)]
enum Game {
    /// Crusader Kings 3
    Ck3,
    /// Victoria 3
    Vic3,
    /// Europa Universalis 5
    Eu5,
}

#[derive(Debug, Parser)]
struct Cli {
    /// Which game the logs are for
    #[arg(long)]
    game: Game,

    /// The game directory of that game
    #[arg(long)]
    gamefiles: PathBuf,

    /// The triggers.log file
    #[arg(long)]
    log: PathBuf,

    /// The tables/triggers.rs file to modify
    #[arg(long)]
    out: PathBuf,
}

const PATTERNS: &[(Game, &str, &str, &str)] = &[
    (Game::Ck3, "", "common/dynasty_legacies", "_perks"),
    (Game::Ck3, "", "common/lifestyles", "_perk_points"),
    (Game::Ck3, "", "common/lifestyles", "_perks"),
    (Game::Ck3, "", "common/lifestyles", "_unlockable_perks"),
    (Game::Ck3, "", "common/lifestyles", "_xp"),
    (Game::Ck3, "perks_in_", "common/lifestyles", ""),
    (Game::Ck3, "num_of_relation_", "common/scripted_relations", ""),
    (Game::Ck3, "has_relation_", "common/scripted_relations", ""),
    (Game::Ck3, "has_secret_relation_", "common/scripted_relations", ""),
];

fn extract_old_triggers(lines: &[String]) -> HashSet<String> {
    let mut result = HashSet::new();

    let mut started = false;
    for line in lines {
        if started {
            if line.starts_with("];") {
                break;
            }
            // cargo fmt creates two kinds of entries: single-line, or multi-line with one field per line.
            if line.starts_with("    (Scopes::") || line.starts_with("        \"") {
                let name = line.split('"').nth(1).unwrap();
                result.insert(name.to_owned());
            }
        } else if line.starts_with("const TRIGGER: ") {
            started = true;
        }
    }
    result
}

fn insert_new_trigger(lines: &mut Vec<String>, name: &str, scopes: Vec<String>) {
    let mut started = false;
    let mut insert_line = None;
    let mut trigger_start_line = 0;
    for (i, line) in lines.iter().enumerate() {
        if started {
            if line.starts_with("];") {
                insert_line = Some(i);
                break;
            }
            if line.starts_with("    (") {
                trigger_start_line = i;
            }
            if line.starts_with("    (Scopes::") || line.starts_with("        \"") {
                let trigger_name = line.split('"').nth(1).unwrap();
                if trigger_name > name {
                    insert_line = Some(trigger_start_line);
                    break;
                }
            }
        } else if line.starts_with("const TRIGGER: ") {
            started = true;
        }
    }
    if let Some(insert_line) = insert_line {
        let mut scope_txt = String::new();
        let mut first = true;
        for s in scopes {
            if first {
                scope_txt = format!("Scopes::{s}");
                first = false;
            } else {
                scope_txt = format!("{scope_txt}.union(Scopes::{s})");
            }
        }
        lines.insert(insert_line, format!("    ({scope_txt}, \"{name}\", UncheckedTodo),"));
    } else {
        eprintln!("could not insert new trigger {name}");
    }
}

fn remove_trigger(lines: &mut [String], name: &str) {
    let mut started = false;
    let mut comment_line = None;
    for (i, line) in lines.iter().enumerate() {
        if started {
            if line.starts_with("];") {
                break;
            }
            if line.starts_with("    (") && comment_line.is_some() {
                break;
            }
            if line.starts_with("    (Scopes::") || line.starts_with("        \"") {
                let trigger_name = line.split('"').nth(1).unwrap();
                if trigger_name == name {
                    comment_line = Some(i);
                }
            }
            if comment_line.is_some() && line.contains("Removed(") {
                return;
            }
        } else if line.starts_with("const TRIGGER: ") {
            started = true;
        }
    }
    if let Some(comment_line) = comment_line {
        lines[comment_line] = format!("{} // TODO: REMOVED", lines[comment_line]);
    } else {
        eprintln!("could not remove new trigger {name}");
    }
}

fn from_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut cap = true;
    for c in s.trim().chars() {
        if c == '_' {
            cap = true;
        } else if cap {
            result.push(c.to_ascii_uppercase());
            cap = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse the `triggers.log` file and return a vec of pairs: the trigger names and a vec of their
/// input scopes.
fn extract_new_triggers(game: Game, log: &str) -> Vec<(&str, Vec<String>)> {
    let mut result = Vec::new();
    let mut in_trigger = None;
    if game == Game::Ck3 {
        for line in log.lines() {
            if line == "--------------------" {
                if let Some(name) = in_trigger {
                    eprintln!("Trigger without input scopes: {name}");
                }
                in_trigger = None;
            } else if let Some(name) = in_trigger {
                if line.starts_with("Supported Scopes: ") {
                    let scopes =
                        line.split(": ").nth(1).unwrap().split(", ").map(from_snake_case).collect();
                    result.push((name, scopes));
                    in_trigger = None;
                }
            } else if line.contains(" - ") {
                let name = line.split(" - ").next().unwrap();
                if name.starts_with("any_") && name != "any_false" {
                    in_trigger = None;
                } else {
                    in_trigger = Some(name);
                }
            }
        }
    } else if game == Game::Vic3 || game == Game::Eu5 {
        for line in log.lines() {
            if let Some(name) = line.strip_prefix("## ") {
                if name.starts_with("any_") && name != "any_false" {
                    in_trigger = None;
                } else {
                    in_trigger = Some(name);
                }
            } else if let Some(name) = in_trigger
                && line.starts_with("**Supported Scopes**: ")
            {
                let scopes =
                    line.split(": ").nth(1).unwrap().split(", ").map(from_snake_case).collect();
                result.push((name, scopes));
            }
        }
    }
    result
}

fn extract_toplevel_items(mut file: &str) -> Vec<&str> {
    let mut result = Vec::new();
    file = if let Some(file) = file.strip_prefix('\u{feff}') { file } else { file };
    for line in file.lines() {
        if !line.starts_with(' ')
            && !line.starts_with('\t')
            && !line.starts_with('#')
            && line.contains(" = {")
        {
            result.push(line.split(" = ").next().unwrap());
        }
    }
    result
}

fn generate_patterns(game: Game, path: &Path) -> Result<HashSet<String>> {
    let mut result = HashSet::new();

    for (for_game, prefix, dir, suffix) in PATTERNS {
        if *for_game == game {
            for entry in read_dir(path.join(dir))? {
                let entry = entry?;
                if !entry.file_type()?.is_file()
                    || !entry.file_name().to_string_lossy().ends_with(".txt")
                {
                    continue;
                }
                for item in extract_toplevel_items(&read_to_string(entry.path())?) {
                    let name = [*prefix, item, *suffix].join("");
                    result.insert(name);
                }
            }
        }
    }
    Ok(result)
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut lines: Vec<String> =
        read_to_string(&args.out)?.lines().map(ToOwned::to_owned).collect();

    // Need to do a little dance because the log file can contain invalid utf-8
    let logfile = BufReader::new(File::open(&args.log)?);
    let mut log = String::new();
    DecodingReader::new(logfile, UTF_8.new_decoder()).lossy().read_to_string(&mut log)?;

    let old_trigger_names = extract_old_triggers(&lines);
    let generated_triggers = generate_patterns(args.game, &args.gamefiles)?;
    let new_triggers = extract_new_triggers(args.game, &log);
    let new_trigger_names: HashSet<&str> = new_triggers.iter().map(|(name, _)| *name).collect();

    for (name, scopes) in new_triggers {
        if !old_trigger_names.contains(name) && !generated_triggers.contains(name) {
            insert_new_trigger(&mut lines, name, scopes);
        }
    }

    for name in &old_trigger_names {
        if !new_trigger_names.contains(&name[..]) {
            remove_trigger(&mut lines, name);
        }
    }

    let mut outfile = lines.join("\n");
    outfile.push('\n');
    write(&args.out, outfile)?;
    Ok(())
}
