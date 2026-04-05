use std::mem::take;

use tiger_tables::datatype::{Arg, Args, Datatype};
use tiger_tables::game::Game;

use crate::datatype_tables::DatatypeTables;
use crate::loca::Kind;
use crate::parse::loca_line::parse_line;
use crate::parse::util::Span;

pub fn hover_description(
    game: Game,
    tables: &DatatypeTables,
    line: &str,
    cursor: usize,
) -> Option<(String, Span)> {
    let mut v = parse_line(line);
    let mut cursor_i = v.binary_search_by(|node| node.span.compare_inclusive(cursor)).ok()?;
    while !v[cursor_i].content.is_empty() {
        v = take(&mut v[cursor_i].content);
        cursor_i = v.binary_search_by(|node| node.span.compare_inclusive(cursor)).ok()?;
    }
    match v[cursor_i].kind {
        // TODO: parse leading type cast from the string literal, if any
        Kind::DatatypeLiteral => Some(("literal: CString".to_string(), v[cursor_i].span)),
        Kind::DatatypeId => {
            let chain: Vec<_> = v
                .iter()
                .filter(|&node| node.kind == Kind::DatatypeId)
                .map(|node| (node.span.extract(line), node.span.contains_inclusive(cursor)))
                .collect();
            // When checking functions, check promotes too because the user may be intending to add
            // a function after the current id.
            let (desc, args_vec, dtypes) = if chain.len() == 1 {
                if let Some((a, d)) = tables.lookup_global_function(game, chain[0].0) {
                    ("global function", vec![(a, vec![d])], vec![])
                } else if let Some((a, d)) = tables.lookup_global_promote(game, chain[0].0) {
                    ("global promote", vec![(a, vec![d])], vec![])
                } else {
                    ("unknown", vec![(Args::Unknown, vec![Datatype::Unknown])], vec![])
                }
            } else {
                let mut args_vec = vec![(Args::Unknown, vec![])];
                let mut dtypes = vec![];
                let mut desc = "unknown";
                for (i, (name, is_cursor)) in chain.iter().enumerate() {
                    let mut temp = Vec::new();
                    for (_, v) in &args_vec {
                        for dtype in v {
                            if !temp.contains(dtype) {
                                temp.push(*dtype);
                            }
                        }
                    }
                    dtypes = temp;

                    (desc, args_vec) = if i == 0 {
                        if let Some((a, d)) = tables.lookup_global_promote(game, name) {
                            ("global promote", vec![(a, vec![d])])
                        } else if let Ok(d) = Datatype::from_str(game, name) {
                            ("data context", vec![(Args::Args(&[]), vec![d])])
                        } else {
                            ("unknown", vec![(Args::Unknown, vec![Datatype::Unknown])])
                        }
                    } else if i + 1 == chain.len() {
                        if let Some(a) = tables.lookup_function(game, &mut dtypes, name) {
                            ("function", a)
                        } else if let Some(a) = tables.lookup_promote(game, &mut dtypes, name) {
                            ("promote", a)
                        } else {
                            ("unknown function", vec![(Args::Unknown, vec![Datatype::Unknown])])
                        }
                    } else if let Some(a) = tables.lookup_promote(game, &mut dtypes, name) {
                        ("promote", a)
                    } else {
                        ("unknown", vec![(Args::Unknown, vec![Datatype::Unknown])])
                    };

                    if *is_cursor {
                        break;
                    }
                }
                (desc, args_vec, dtypes)
            };

            #[allow(clippy::comparison_chain)]
            let mut message = args_vec
                .into_iter()
                .enumerate()
                .map_while(|(i, (args, dtypes))| {
                    if i < 5 {
                        Some(format!(
                            "{desc} {}{}: {}  ",
                            v[cursor_i].span.extract(line),
                            display_args(args),
                            display_dtypes(&dtypes)
                        ))
                    } else if i == 5 {
                        Some("...".into())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !dtypes.is_empty() {
                message = format!("{}  \n{message}", display_dtypes(&dtypes));
            }

            Some((message, v[cursor_i].span))
        }
        Kind::IconText => Some(("Icon".to_string(), v[cursor_i].span)),
        Kind::MacroText => Some(("Macro".to_string(), v[cursor_i].span)),
        _ => None,
    }
}

fn display_dtypes(dtypes: &[Datatype]) -> String {
    #[allow(clippy::comparison_chain)]
    dtypes
        .iter()
        .enumerate()
        .map_while(|(i, d)| {
            if i < 5 {
                Some(d.to_string())
            } else if i == 5 {
                Some("...".into())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" | ")
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
    #[allow(clippy::comparison_chain)]
    match arg {
        Arg::DType(dtype) => dtype.to_string(),
        Arg::IType(itype) => format!("{itype} CString"),
        Arg::Choice(choices) => choices
            .iter()
            .enumerate()
            .map_while(|(i, c)| {
                if i < 5 {
                    Some(c.to_string())
                } else if i == 5 {
                    Some("...".into())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" | "),
    }
}
