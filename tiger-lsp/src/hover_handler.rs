use tiger_tables::datatype::{Arg, Args, Datatype};

use crate::config::Game;
use crate::datatype_tables::DatatypeTables;
use crate::parse::loca_line::{LocaTokenKind, parse_line};
use crate::parse::util::Span;

pub fn hover_description(
    game: Game,
    tables: &DatatypeTables,
    line: &str,
    cursor: usize,
) -> Option<(String, Span)> {
    let v = parse_line(line);
    let cursor_i = v.iter().position(|(_, span)| span.contains_inclusive(cursor))?;
    match v[cursor_i].0 {
        LocaTokenKind::DatatypeLiteral => Some(("CString literal".to_string(), v[cursor_i].1)),
        LocaTokenKind::DatatypeId(expr_id, expr_depth) => {
            let chain: Vec<_> = v
                .iter()
                .filter_map(|(token, span)| {
                    if let LocaTokenKind::DatatypeId(id, depth) = token
                        && *id == expr_id
                        && *depth == expr_depth
                    {
                        Some((span.extract(line), span.contains_inclusive(cursor)))
                    } else {
                        None
                    }
                })
                .collect();
            // When checking functions, check promotes too because the user may be intending to add
            // a function after the current id.
            let (desc, args, dtype) = if chain.len() == 1 {
                if let Some((a, d)) = tables.lookup_global_function(game, chain[0].0) {
                    ("global function", a, d)
                } else if let Some((a, d)) = tables.lookup_global_promote(game, chain[0].0) {
                    ("global promote", a, d)
                } else {
                    ("unknown", Args::Unknown, Datatype::Unknown)
                }
            } else {
                let mut args = Args::Unknown;
                let mut dtype = Datatype::Unknown;
                let mut desc = "unknown";
                for (i, (name, is_cursor)) in chain.iter().enumerate() {
                    if i == 0 {
                        if let Some((a, d)) = tables.lookup_global_promote(game, name) {
                            desc = "global promote";
                            args = a;
                            dtype = d;
                        } else if let Ok(d) = name.parse::<Datatype>() {
                            desc = "data context";
                            args = Args::Args(&[]);
                            dtype = d;
                        } else {
                            desc = "unknown";
                            args = Args::Unknown;
                            dtype = Datatype::Unknown;
                        }
                    } else if i + 1 == chain.len() {
                        if let Some((a, d)) = tables.lookup_function(game, dtype, name) {
                            desc = "function";
                            args = a;
                            dtype = d;
                        } else if let Some((a, d)) = tables.lookup_promote(game, dtype, name) {
                            desc = "promote";
                            args = a;
                            dtype = d;
                        } else {
                            desc = "unknown function";
                            args = Args::Unknown;
                            dtype = Datatype::Unknown;
                        }
                    } else if let Some((a, d)) = tables.lookup_promote(game, dtype, name) {
                        desc = "promote";
                        args = a;
                        dtype = d;
                    }
                    if *is_cursor {
                        break;
                    }
                }
                (desc, args, dtype)
            };
            Some((
                format!("{desc} {}{}: {dtype}", v[cursor_i].1.extract(line), display_args(args)),
                v[cursor_i].1,
            ))
        }
        _ => None,
    }
}

fn display_args(args: Args) -> String {
    match args {
        Args::Unknown => "(?)".to_string(),
        Args::Args(a) => {
            if a.is_empty() {
                String::new()
            } else {
                format!(
                    "({})",
                    a.iter().map(|arg| display_arg(*arg)).collect::<Vec<_>>().join(", ")
                )
            }
        }
    }
}

fn display_arg(arg: Arg) -> String {
    match arg {
        Arg::DType(dtype) => dtype.to_string(),
        Arg::IType(itype) => format!("{itype} CString"),
        Arg::Choice(choices) => {
            if choices.len() <= 5 {
                choices.iter().map(|c| format!("'{c}'")).collect::<Vec<_>>().join(" | ")
            } else {
                format!(
                    "{} | ...",
                    choices[..5].iter().map(|c| format!("'{c}'")).collect::<Vec<_>>().join(" | ")
                )
            }
        }
    }
}
