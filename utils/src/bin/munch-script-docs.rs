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

    /// The folder with the `script_docs` files
    #[arg(long)]
    logs: PathBuf,

    /// The tables directory to modify
    #[arg(long)]
    out: PathBuf,
}

// TODO EU5: fill in any patterns EU5 has

const TRIGGER_PATTERNS: &[(Game, &str, &str, &str, Option<&str>)] = &[
    (Game::Ck3, "", "common/dynasty_legacies", "_perks", None),
    (Game::Ck3, "", "common/lifestyles", "_perk_points", None),
    (Game::Ck3, "", "common/lifestyles", "_perks", None),
    (Game::Ck3, "", "common/lifestyles", "_unlockable_perks", None),
    (Game::Ck3, "", "common/lifestyles", "_xp", None),
    (Game::Ck3, "perks_in_", "common/lifestyles", "", None),
    (Game::Ck3, "num_of_relation_", "common/scripted_relations", "", None),
    (Game::Ck3, "has_relation_", "common/scripted_relations", "", None),
    (Game::Ck3, "has_secret_relation_", "common/scripted_relations", "", None),
];
const EFFECT_PATTERNS: &[(Game, &str, &str, &str, Option<&str>)] = &[
    (Game::Ck3, "add_", "common/lifestyles", "_perk_points", None),
    (Game::Ck3, "add_", "common/lifestyles", "_xp", None),
    (Game::Ck3, "set_relation_", "common/scripted_relations", "", None),
    (Game::Ck3, "remove_relation_", "common/scripted_relations", "", None),
];
const ITERATOR_PATTERNS: &[(Game, &str, &str, &str, Option<&str>)] = &[
    (Game::Vic3, "country_in_", "common/geographic_regions", "", Some("short_key")),
    (Game::Vic3, "province_in_", "common/geographic_regions", "", Some("short_key")),
    (Game::Vic3, "state_in_", "common/geographic_regions", "", Some("short_key")),
    (Game::Vic3, "state_region_in_", "common/geographic_regions", "", Some("short_key")),
    (Game::Vic3, "strategic_region_in_", "common/geographic_regions", "", Some("short_key")),
];

const TRIGGERS_TABLE_START: &str = "const TRIGGER: ";
const EFFECTS_TABLE_START: &str = "const SCOPE_EFFECT: ";
const ITERATORS_TABLE_START: &str = "const ITERATOR: ";

fn is_table_start(line: &str) -> bool {
    line.starts_with(TRIGGERS_TABLE_START)
        || line.starts_with(EFFECTS_TABLE_START)
        || line.starts_with(ITERATORS_TABLE_START)
}

fn extract_old_entries(lines: &[String]) -> HashSet<String> {
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
        } else if is_table_start(line) {
            started = true;
        }
    }
    result
}

fn insert_new_entry(lines: &mut Vec<String>, name: &str, scopes: &[String]) {
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
        } else if is_table_start(line) {
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
        eprintln!("could not insert new entry {name}");
    }
}

fn remove_entry(lines: &mut [String], name: &str) {
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
        } else if is_table_start(line) {
            started = true;
        }
    }
    if let Some(comment_line) = comment_line {
        lines[comment_line] = format!("{} // TODO: REMOVED", lines[comment_line]);
    } else {
        eprintln!("could not remove obsolete entry {name}");
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

/// Parse the script docs file and return a vec of pairs: the trigger or effect names and a vec of their input scopes.
fn extract_new_entries(game: Game, log: &str) -> Vec<(&str, Vec<String>)> {
    let mut result = Vec::new();
    let mut in_entry = None;
    if game == Game::Ck3 {
        for line in log.lines() {
            if line == "--------------------" {
                if let Some(name) = in_entry {
                    eprintln!("Entry without input scopes: {name}");
                }
                in_entry = None;
            } else if let Some(name) = in_entry {
                if line.starts_with("Supported Scopes: ") {
                    let scopes =
                        line.split(": ").nth(1).unwrap().split(", ").map(from_snake_case).collect();
                    result.push((name, scopes));
                    in_entry = None;
                }
            } else if line.contains(" - ") {
                let name = line.split(" - ").next().unwrap();
                in_entry = Some(name);
            }
        }
    } else if game == Game::Vic3 || game == Game::Eu5 {
        for line in log.lines() {
            if let Some(name) = line.strip_prefix("## ") {
                in_entry = Some(name);
            } else if let Some(name) = in_entry
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

fn extract_new_iterators(
    triggers: &mut Vec<(&str, Vec<String>)>,
    effects: &mut Vec<(&str, Vec<String>)>,
) -> Vec<(String, Vec<String>)> {
    let effect_names: HashSet<&str> = effects.iter().map(|(name, _)| *name).collect();
    let mut iterator_names: HashSet<&str> = HashSet::new();
    let mut iterators: Vec<(String, Vec<String>)> = Vec::new();

    for (name, scopes) in triggers.iter() {
        if let Some(iterator) = name.strip_prefix("any_") {
            if effect_names.contains(&format!("every_{iterator}")[..])
                && effect_names.contains(&format!("random_{iterator}")[..])
                && effect_names.contains(&format!("ordered_{iterator}")[..])
            {
                iterators.push((iterator.to_owned(), scopes.to_owned()));
                iterator_names.insert(iterator);
            }
        }
    }

    triggers.retain(|(name, _)| {
        if let Some(iterator) = name.strip_prefix("any_") {
            !iterator_names.contains(&iterator)
        } else {
            true
        }
    });
    effects.retain(|(name, _)| {
        if let Some(iterator) = name.strip_prefix("every_") {
            !iterator_names.contains(&iterator)
        } else if let Some(iterator) = name.strip_prefix("random_") {
            !iterator_names.contains(&iterator)
        } else if let Some(iterator) = name.strip_prefix("ordered_") {
            !iterator_names.contains(&iterator)
        } else {
            true
        }
    });

    iterators
}

fn extract_toplevel_items(file: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let file = file.strip_prefix('\u{feff}').unwrap_or(file);
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

fn extract_item_fields<'a>(file: &'a str, field: &str) -> Vec<&'a str> {
    let mut result = Vec::new();
    let file = file.strip_prefix('\u{feff}').unwrap_or(file);
    let field_assign = format!("{field} = ");
    for line in file.lines() {
        if line.contains(&field_assign) {
            let key = line.split(" = ").nth(1).unwrap();
            let key = if key.contains('"') { key.split('"').nth(1).unwrap() } else { key };
            result.push(key);
        }
    }
    result
}

fn generate_patterns(
    game: Game,
    path: &Path,
    patterns: &[(Game, &str, &str, &str, Option<&str>)],
) -> Result<HashSet<String>> {
    let mut result = HashSet::new();

    for (for_game, prefix, dir, suffix, field) in patterns {
        if *for_game == game {
            for entry in read_dir(path.join(dir))? {
                let entry = entry?;
                if !entry.file_type()?.is_file()
                    || !entry.file_name().to_string_lossy().ends_with(".txt")
                {
                    continue;
                }
                let file = read_to_string(entry.path())?;
                let items = if let Some(field) = field {
                    extract_item_fields(&file, field)
                } else {
                    extract_toplevel_items(&file)
                };
                for item in items {
                    let name = [*prefix, item, *suffix].join("");
                    result.insert(name);
                }
            }
        }
    }
    Ok(result)
}

fn read_log(path: &Path) -> Result<String> {
    // Need to do a little dance because the log file can contain invalid utf-8
    let reader = BufReader::new(File::open(path)?);
    let mut log = String::new();
    DecodingReader::new(reader, UTF_8.new_decoder()).lossy().read_to_string(&mut log)?;
    Ok(log)
}

fn rewrite_file(path: &Path, lines: &[String], separator: &str) -> Result<()> {
    let mut outfile = lines.join(separator);
    outfile.push_str(separator);
    write(path, outfile)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let triggers_file = read_to_string(args.out.join("triggers.rs"))?;
    let effects_file = read_to_string(args.out.join("effects.rs"))?;
    let iterators_file = read_to_string(args.out.join("iterators.rs"))?;

    let line_separator = if triggers_file.contains("\r\n") { "\r\n" } else { "\n" };

    let mut triggers_lines: Vec<String> = triggers_file.lines().map(ToOwned::to_owned).collect();
    let mut effects_lines: Vec<String> = effects_file.lines().map(ToOwned::to_owned).collect();
    let mut iterators_lines: Vec<String> = iterators_file.lines().map(ToOwned::to_owned).collect();

    let old_trigger_names = extract_old_entries(&triggers_lines);
    let old_effect_names = extract_old_entries(&effects_lines);
    let old_iterator_names = extract_old_entries(&iterators_lines);

    let generated_triggers = generate_patterns(args.game, &args.gamefiles, TRIGGER_PATTERNS)?;
    let generated_effects = generate_patterns(args.game, &args.gamefiles, EFFECT_PATTERNS)?;
    let generated_iterators = generate_patterns(args.game, &args.gamefiles, ITERATOR_PATTERNS)?;

    let triggers_logfile = read_log(&args.logs.join("triggers.log"))?;
    let effects_logfile = read_log(&args.logs.join("effects.log"))?;

    let mut new_triggers = extract_new_entries(args.game, &triggers_logfile);
    let mut new_effects = extract_new_entries(args.game, &effects_logfile);
    let new_iterators = extract_new_iterators(&mut new_triggers, &mut new_effects);

    let new_trigger_names: HashSet<&str> = new_triggers.iter().map(|(name, _)| *name).collect();
    let new_effect_names: HashSet<&str> = new_effects.iter().map(|(name, _)| *name).collect();
    let new_iterator_names: HashSet<String> =
        new_iterators.iter().map(|(name, _)| name.clone()).collect();

    for (name, scopes) in &new_triggers {
        if !old_trigger_names.contains(&name[..]) && !generated_triggers.contains(&name[..]) {
            insert_new_entry(&mut triggers_lines, name, scopes);
        }
    }
    for (name, scopes) in &new_effects {
        if !old_effect_names.contains(&name[..]) && !generated_effects.contains(&name[..]) {
            insert_new_entry(&mut effects_lines, name, scopes);
        }
    }
    for (name, scopes) in &new_iterators {
        if !old_iterator_names.contains(&name[..]) && !generated_iterators.contains(&name[..]) {
            insert_new_entry(&mut iterators_lines, name, scopes);
        }
    }

    for name in &old_trigger_names {
        if !new_trigger_names.contains(&name[..]) {
            remove_entry(&mut triggers_lines, name);
        }
    }
    for name in &old_effect_names {
        if !new_effect_names.contains(&name[..]) {
            remove_entry(&mut effects_lines, name);
        }
    }
    for name in &old_iterator_names {
        if !new_iterator_names.contains(&name[..]) {
            remove_entry(&mut iterators_lines, name);
        }
    }

    rewrite_file(&args.out.join("triggers.rs"), &triggers_lines, line_separator)?;
    rewrite_file(&args.out.join("effects.rs"), &effects_lines, line_separator)?;
    rewrite_file(&args.out.join("iterators.rs"), &iterators_lines, line_separator)?;
    Ok(())
}
