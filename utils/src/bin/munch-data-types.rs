use std::collections::{HashMap, HashSet};
use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use utils::datatypes::{
    GENERIC_TYPES, Game, Global, NonGlobal, load_globals, load_nonglobals, write_globals,
    write_nonglobals, write_types,
};

#[derive(Debug, Parser)]
struct Cli {
    /// Which game the logs are for
    #[arg(long)]
    game: Game,

    /// Directory with the datatype logs (input)
    #[arg(long)]
    logs: PathBuf,

    /// Directory with the Rust include files (output)
    #[arg(long)]
    out: PathBuf,
}

fn merge_globals(globals: &mut HashMap<String, Global>, mut new_globals: HashMap<String, Global>) {
    globals.retain(|k, _| new_globals.contains_key(k));

    for (k, v) in new_globals.drain() {
        if let Some(old) = globals.get(&k) {
            if old.args.len() == v.args.len() && (v.rtype == "Unknown" || old.rtype == v.rtype) {
                continue;
            }
        }
        globals.insert(k, v);
    }
}

fn merge_nonglobals(
    nonglobals: &mut HashMap<String, NonGlobal>,
    mut new_nonglobals: HashMap<String, NonGlobal>,
) {
    nonglobals.retain(|k, _| new_nonglobals.contains_key(k));

    for (k, v) in new_nonglobals.drain() {
        if let Some(old) = nonglobals.get(&k) {
            if old.args.len() == v.args.len() && (v.rtype == "Unknown" || old.rtype == v.rtype) {
                continue;
            }
        }
        nonglobals.insert(k, v);
    }
}

fn parse_datafunction(
    item: &str,
    types: &mut HashSet<String>,
    global_promotes: &mut HashMap<String, Global>,
    global_functions: &mut HashMap<String, Global>,
    promotes: &mut HashMap<String, NonGlobal>,
    functions: &mut HashMap<String, NonGlobal>,
    game: Game,
) {
    if item.is_empty() {
        return;
    }
    let header = item.lines().next().unwrap();

    if item.contains("Definition type: Type") {
        types.insert(header.to_owned());
        return;
    }

    if item.contains("Definition type: Global macro") {
        return;
    }

    let mut name = header;
    let mut nargs = 0;
    if let Some((s1, s2)) = header.split_once('(') {
        name = s1;
        nargs = s2.split(',').count();
    }
    let mut args = Vec::new();
    for _ in 0..nargs {
        args.push("DType(Unknown)".to_string());
    }

    let mut rtype = "";
    if let Some((_, s2)) = item.split_once("Return type: ") {
        rtype = s2.trim();
    }
    if rtype == "[unregistered]" {
        rtype = "Unknown";
    } else if rtype == "_null_type_" {
        rtype = "void";
    }
    // Implement bugfixes to the game's logs
    if name == "IntToFixedPoint" && rtype == "float" {
        rtype = "CFixedPoint";
    } else if name == "IntToUnsigned" && rtype == "float" {
        rtype = "uint32";
    }
    let store;
    if !GENERIC_TYPES.contains(&rtype) {
        store = format!("{game}({rtype})");
        rtype = &store;
    }

    if item.contains("Definition type: Global promote") {
        global_promotes
            .insert(name.to_owned(), Global::new(name.to_owned(), args, rtype.to_owned()));
        return;
    } else if item.contains("Definition type: Global function") {
        global_functions
            .insert(name.to_owned(), Global::new(name.to_owned(), args, rtype.to_owned()));
        return;
    }

    let (mut dtype, barename) = name.split_once('.').unwrap();
    let store2;
    if !GENERIC_TYPES.contains(&dtype) {
        store2 = format!("{game}({dtype})");
        dtype = &store2;
    }
    if barename == "Self" || barename == "AccessSelf" {
        rtype = dtype;
    }

    if item.contains("Definition type: Promote") {
        promotes.insert(
            name.to_owned(),
            NonGlobal::new(barename.to_owned(), dtype.to_owned(), args, rtype.to_owned()),
        );
        return;
    }
    if item.contains("Definition type: Function") {
        functions.insert(
            name.to_owned(),
            NonGlobal::new(barename.to_owned(), dtype.to_owned(), args, rtype.to_owned()),
        );
    }
}

fn fill_in_functions(
    promotes: &HashMap<String, NonGlobal>,
    functions: &mut HashMap<String, NonGlobal>,
) {
    for (name, function) in functions.iter_mut() {
        if let Some(promote) = promotes.get(name) {
            if function.rtype == "Unknown"
                && promote.rtype != "Unknown"
                && function.args == promote.args
            {
                function.rtype.clone_from(&promote.rtype);
            }
        }
    }
}

fn fill_in_global_functions(
    promotes: &HashMap<String, Global>,
    functions: &mut HashMap<String, Global>,
) {
    for (name, function) in functions.iter_mut() {
        if let Some(promote) = promotes.get(name) {
            if function.rtype == "Unknown"
                && promote.rtype != "Unknown"
                && function.args == promote.args
            {
                function.rtype.clone_from(&promote.rtype);
            }
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // let types = load_types(args.out.join("datatypes.rs"))?;
    let mut global_promotes = load_globals(&args.out.join("data_global_promotes.rs"), args.game)?;
    let mut global_functions = load_globals(&args.out.join("data_global_functions.rs"), args.game)?;
    let mut promotes = load_nonglobals(&args.out.join("data_promotes.rs"), args.game)?;
    let mut functions = load_nonglobals(&args.out.join("data_functions.rs"), args.game)?;

    let mut new_types = HashSet::new();
    let mut new_global_promotes = HashMap::new();
    let mut new_global_functions = HashMap::new();
    let mut new_promotes = HashMap::new();
    let mut new_functions = HashMap::new();

    for entry in read_dir(args.logs)? {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().ends_with(".txt") {
            continue;
        }
        let content = read_to_string(entry.path())?;
        for item in content.split("\n-----------------------\n\n") {
            parse_datafunction(
                item,
                &mut new_types,
                &mut new_global_promotes,
                &mut new_global_functions,
                &mut new_promotes,
                &mut new_functions,
                args.game,
            );
        }
    }

    // Drop the game concepts.
    // Heuristic: they are all lowercase while real datafunctions contain uppercase.
    if args.game == Game::Ck3 || args.game == Game::Eu5 {
        new_global_functions.retain(|k, _| k.chars().any(char::is_uppercase));
    }
    // Heuristic: they all start with concept_
    if args.game == Game::Vic3 {
        new_global_functions.retain(|k, _| !k.starts_with("concept_"));
    }

    // Root seems to work as well as ROOT
    new_global_promotes
        .insert("Root".to_string(), Global::new("Root".to_string(), vec![], "Scope".to_string()));

    merge_globals(&mut global_promotes, new_global_promotes);
    merge_globals(&mut global_functions, new_global_functions);
    merge_nonglobals(&mut promotes, new_promotes);
    merge_nonglobals(&mut functions, new_functions);

    fill_in_functions(&promotes, &mut functions);
    fill_in_global_functions(&global_promotes, &mut global_functions);

    write_types(new_types, args.out.join("datatypes.rs"), args.game)?;
    write_globals(global_promotes, args.out.join("data_global_promotes.rs"))?;
    write_globals(global_functions, args.out.join("data_global_functions.rs"))?;
    write_nonglobals(promotes, args.out.join("data_promotes.rs"))?;
    write_nonglobals(functions, args.out.join("data_functions.rs"))?;

    Ok(())
}
