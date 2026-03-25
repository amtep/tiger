use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use utils::datatypes::{
    Game, Global, NonGlobal, add_game_wrapper, load_globals, load_nonglobals, write_globals,
    write_nonglobals,
};

#[derive(Debug, Parser)]
struct Cli {
    /// Which game the scan is from
    #[arg(long)]
    game: Game,

    /// The file with the scan output
    #[arg(long)]
    input: PathBuf,

    /// Directory with the Rust include files
    #[arg(long)]
    out: PathBuf,
}

fn scan_global(args: &mut HashMap<(String, usize), String>, line: &str, game: Game) -> Result<()> {
    let split: Vec<&str> = line.split(' ').collect();
    let name = split[0].to_owned();
    let arg = split[2].parse::<usize>()?;
    let dtype = split[4];
    if let Some(other_dtype) = args.get(&(name.clone(), arg)) {
        if dtype != other_dtype {
            args.insert((name, arg), String::new());
        }
        return Ok(());
    }
    let dtype = format!("DType({})", add_game_wrapper(dtype, game));
    args.insert((name, arg), dtype);
    Ok(())
}

fn scan_nonglobal(
    args: &mut HashMap<(String, String, usize), String>,
    line: &str,
    game: Game,
) -> Result<()> {
    let split: Vec<&str> = line.split(' ').collect();
    let name = split[0].to_owned();
    let parent_type = split[1].to_owned();
    let arg = split[3].parse::<usize>()?;
    let dtype = split[5];
    if let Some(other_dtype) = args.get(&(name.clone(), parent_type.clone(), arg)) {
        if dtype != other_dtype {
            args.insert((name, parent_type, arg), String::new());
        }
        return Ok(());
    }
    let dtype = format!("DType({})", add_game_wrapper(dtype, game));
    args.insert((name, parent_type, arg), dtype);
    Ok(())
}

fn apply_global(globals: &mut HashMap<String, Global>, scan: HashMap<(String, usize), String>) {
    for ((name, argnr), dtype) in scan {
        if let Some(entry) = globals.get_mut(&name)
            && entry.args[argnr] == "DType(Unknown)"
            && !dtype.is_empty()
        {
            entry.args[argnr] = dtype;
        }
    }
}

fn apply_nonglobal(
    nonglobals: &mut HashMap<String, NonGlobal>,
    scan: HashMap<(String, String, usize), String>,
) {
    for ((name, parent_dtype, argnr), dtype) in scan {
        let index = format!("{parent_dtype}.{name}");
        if let Some(entry) = nonglobals.get_mut(&index)
            && entry.args[argnr] == "DType(Unknown)"
            && !dtype.is_empty()
        {
            entry.args[argnr] = dtype;
        }
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();

    let mut global_promotes = load_globals(&args.out.join("data_global_promotes.rs"), args.game)?;
    let mut global_functions = load_globals(&args.out.join("data_global_functions.rs"), args.game)?;
    let mut promotes = load_nonglobals(&args.out.join("data_promotes.rs"), args.game)?;
    let mut functions = load_nonglobals(&args.out.join("data_functions.rs"), args.game)?;

    let mut global_promote_args = HashMap::new();
    let mut global_function_args = HashMap::new();
    let mut promote_args = HashMap::new();
    let mut function_args = HashMap::new();

    for line in read_to_string(args.input)?.lines() {
        if let Some(line) = line.strip_prefix("global promote ") {
            scan_global(&mut global_promote_args, line, args.game)?;
        } else if let Some(line) = line.strip_prefix("global function ") {
            scan_global(&mut global_function_args, line, args.game)?;
        } else if let Some(line) = line.strip_prefix("promote ") {
            scan_nonglobal(&mut promote_args, line, args.game)?;
        } else if let Some(line) = line.strip_prefix("function ") {
            scan_nonglobal(&mut function_args, line, args.game)?;
        }
    }

    apply_global(&mut global_promotes, global_promote_args);
    apply_global(&mut global_functions, global_function_args);
    apply_nonglobal(&mut promotes, promote_args);
    apply_nonglobal(&mut functions, function_args);

    write_globals(global_promotes, args.out.join("data_global_promotes.rs"))?;
    write_globals(global_functions, args.out.join("data_global_functions.rs"))?;
    write_nonglobals(promotes, args.out.join("data_promotes.rs"))?;
    write_nonglobals(functions, args.out.join("data_functions.rs"))?;

    Ok(())
}
