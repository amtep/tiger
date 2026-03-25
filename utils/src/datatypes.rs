use std::collections::{HashMap, HashSet};
use std::fs::{File, read_to_string};
use std::hash::BuildHasher;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::ValueEnum;
use strum_macros::Display;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum, Display)]
pub enum Game {
    /// Crusader Kings 3
    Ck3,
    /// Victoria 3
    Vic3,
    /// Europa Universalis 5
    Eu5,
}

fn remove_game_wrapper(sometype: &str) -> &str {
    if let Some(sfx) = sometype.strip_prefix("Ck3(")
        && let Some(result) = sfx.strip_suffix(')')
    {
        return result;
    }
    if let Some(sfx) = sometype.strip_prefix("Vic3(")
        && let Some(result) = sfx.strip_suffix(')')
    {
        return result;
    }
    if let Some(sfx) = sometype.strip_prefix("Eu5(")
        && let Some(result) = sfx.strip_suffix(')')
    {
        return result;
    }
    sometype
}

pub fn add_game_wrapper(sometype: &str, game: Game) -> String {
    if GENERIC_TYPES.contains(&sometype) {
        sometype.to_owned()
    } else {
        format!("{game}({sometype})")
    }
}

pub const GENERIC_TYPES: &[&str] = &[
    "Unknown",
    "AnyScope",
    "CFixedPoint",
    "CString",
    "CUTF8String",
    "CVector2f",
    "CVector2i",
    "CVector3f",
    "CVector3i",
    "CVector4f",
    "CVector4i",
    "Date",
    "Scope",
    "TopScope",
    "bool",
    "double",
    "float",
    "int16",
    "int32",
    "int64",
    "int8",
    "uint16",
    "uint32",
    "uint64",
    "uint8",
    "void",
];

pub fn write_types<H: BuildHasher>(
    mut types: HashSet<String, H>,
    fname: PathBuf,
    game: Game,
) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Display, EnumString)]")?;
    writeln!(outf, "#[strum(use_phf)]")?;
    writeln!(outf, "pub enum {game}Datatype {{")?;
    let mut types: Vec<_> = types.drain().collect();
    types.sort();
    for t in types {
        if !GENERIC_TYPES.contains(&&*t) {
            writeln!(outf, "    {t},")?;
        }
    }
    writeln!(outf, "}}")?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Global {
    pub name: String,
    pub args: Vec<String>,
    pub rtype: String,
}

impl Global {
    pub fn new(name: String, args: Vec<String>, rtype: String) -> Self {
        Self { name, args, rtype }
    }

    pub fn print<F: Write>(&self, outf: &mut F) -> Result<()> {
        writeln!(
            outf,
            "    (\"{}\", Args::Args(&[{}]), {}),",
            self.name,
            self.args.join(", "),
            self.rtype
        )
        .context("print")
    }
}

pub fn load_globals(fname: &Path, game: Game) -> Result<HashMap<String, Global>> {
    let mut globals = HashMap::new();
    let global = read_to_string(fname)?;
    for line in global.lines() {
        // Skip header and footer lines
        if !line.starts_with(' ') {
            continue;
        }
        let line = line.strip_prefix("    (\"").context("parse error1")?;
        let (name, line) = line.split_once("\", Args::Args(&[").context("parse error2")?;
        let line = line.strip_suffix("),").context("parse error3")?;
        let (line, rtype) = line.rsplit_once("]), ").context("parse error4")?;
        let args: Vec<_> = if line.is_empty() {
            Vec::new()
        } else {
            line.split(", ").map(ToOwned::to_owned).collect()
        };
        let rtype = add_game_wrapper(remove_game_wrapper(rtype), game);
        globals.insert(name.to_owned(), Global::new(name.to_owned(), args, rtype));
    }
    Ok(globals)
}

pub fn write_globals<H: BuildHasher>(
    mut globals: HashMap<String, Global, H>,
    fname: PathBuf,
) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "&[")?;
    let mut globals: Vec<_> = globals.drain().map(|(_, v)| v).collect();
    globals.sort();
    for g in globals {
        g.print(&mut outf)?;
    }
    writeln!(outf, "]")?;
    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonGlobal {
    pub name: String,
    pub dtype: String,
    pub args: Vec<String>,
    pub rtype: String,
}

impl NonGlobal {
    pub fn new(name: String, dtype: String, args: Vec<String>, rtype: String) -> Self {
        Self { name, dtype, args, rtype }
    }

    pub fn print<F: Write>(&self, outf: &mut F) -> Result<()> {
        writeln!(
            outf,
            "    (\"{}\", {}, Args::Args(&[{}]), {}),",
            self.name,
            self.dtype,
            self.args.join(", "),
            self.rtype
        )
        .context("print")
    }
}

pub fn load_nonglobals(fname: &Path, game: Game) -> Result<HashMap<String, NonGlobal>> {
    let mut nonglobals = HashMap::new();
    let nonglobal = read_to_string(fname)?;
    for line in nonglobal.lines() {
        // Skip header and footer lines
        if !line.starts_with(' ') {
            continue;
        }
        let line = line.strip_prefix("    (\"").context("parse error1")?;
        let (name, line) = line.split_once("\", ").context("parse error2")?;
        let (dtype, line) = line.split_once(", Args::Args(&[").context("parse error2b")?;
        let line = line.strip_suffix("),").context("parse error3")?;
        let (line, rtype) = line.rsplit_once("]), ").context("parse error4")?;
        let dtype = remove_game_wrapper(dtype);
        let rtype = add_game_wrapper(remove_game_wrapper(rtype), game);
        let args: Vec<_> = if line.is_empty() {
            Vec::new()
        } else {
            line.split(", ").map(ToOwned::to_owned).collect()
        };
        let idx = format!("{dtype}.{name}");
        let dtype = add_game_wrapper(dtype, game);
        nonglobals.insert(idx, NonGlobal::new(name.to_owned(), dtype, args, rtype));
    }
    Ok(nonglobals)
}

pub fn write_nonglobals<H: BuildHasher>(
    mut nonglobals: HashMap<String, NonGlobal, H>,
    fname: PathBuf,
) -> Result<()> {
    let mut outf = File::create(fname)?;
    writeln!(outf, "&[")?;
    let mut nonglobals: Vec<_> = nonglobals.drain().map(|(_, v)| v).collect();
    nonglobals.sort();
    for n in nonglobals {
        n.print(&mut outf)?;
    }
    writeln!(outf, "]")?;
    Ok(())
}
