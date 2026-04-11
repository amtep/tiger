use std::cmp::Ordering;
use std::mem::take;

use tiger_tables::datatype::{Arg, Args, Datatype};
use tiger_tables::game::Game;

use crate::datatype_tables::DatatypeTables;
use crate::game_concepts::GameConcepts;
use crate::key_texts::KeyTexts;
use crate::loca::{Kind, find_cursor_index};
use crate::parse::loca_line::parse_line;
use crate::parse::util::Span;
use crate::util::tree;

#[derive(Debug, PartialEq, Eq)]
struct SortArgs(Args);

impl Ord for SortArgs {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0, other.0) {
            (Args::Unknown, Args::Unknown) => Ordering::Equal,
            (Args::Unknown, Args::Args(_)) => Ordering::Less,
            (Args::Args(_), Args::Unknown) => Ordering::Greater,
            (Args::Args(args1), Args::Args(args2)) => args1
                .len()
                .cmp(&args2.len())
                .then_with(|| display_args(self.0).cmp(&display_args(other.0))),
        }
    }
}

impl PartialOrd for SortArgs {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn hover_description(
    game: Game,
    tables: &DatatypeTables,
    game_concepts: &GameConcepts,
    key_texts: &KeyTexts,
    line: &str,
    cursor: usize,
) -> Option<(String, Span)> {
    let mut v = parse_line(line);
    let mut cursor_i = find_cursor_index(&v, cursor)?;
    log::trace!("\n{}", tree(&v[cursor_i], line));

    while !v[cursor_i].content.is_empty() {
        v = take(&mut v[cursor_i].content);
        cursor_i = find_cursor_index(&v, cursor)?;
    }

    match v[cursor_i].kind {
        Kind::DatatypeLiteral => {
            Some((display_dtype_literal(v[cursor_i].span.extract(line)).into(), v[cursor_i].span))
        }
        Kind::DatatypeId => {
            let chain: Vec<_> = v
                .iter()
                .filter(|&node| node.kind == Kind::DatatypeId)
                .map(|node| (node.span.extract(line), node.span.contains_inclusive(cursor)))
                .collect();

            let mut aliases: Option<Vec<&str>> = None;
            // When checking functions, check promotes too because the user may be intending to add
            // a function after the current id.
            let (desc, mut args_vec, dtypes) = if chain.len() == 1 {
                if let Some((a, d)) = tables.lookup_global_function(game, chain[0].0) {
                    ("global function", vec![(a, vec![d])], vec![])
                } else if let Some((a, d)) = tables.lookup_global_promote(game, chain[0].0) {
                    ("global promote", vec![(a, vec![d])], vec![])
                } else if let Some(a) = game_concepts.get(chain[0].0) {
                    if a.len() != 1 {
                        // INFO: at least one alias
                        aliases = Some(
                            a.iter().map(String::as_str).filter(|a| *a != chain[0].0).collect(),
                        );
                    }
                    ("game concept", vec![(Args::Args(&[]), vec![Datatype::CString])], vec![])
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

            args_vec.sort_unstable_by_key(|(a, _)| SortArgs(*a));

            #[allow(clippy::comparison_chain)]
            let mut message = args_vec
                .into_iter()
                .enumerate()
                .map_while(|(i, (args, dtypes))| {
                    if i < 6 {
                        Some(format!(
                            "{desc} {}{}: {}  ",
                            v[cursor_i].span.extract(line),
                            display_args(args),
                            display_dtypes(&dtypes)
                        ))
                    } else if i == 6 {
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
            if let Some(aliases) = aliases {
                message = format!("{message}\naliases: {}", aliases.join(", "));
            }

            Some((message, v[cursor_i].span))
        }
        Kind::IconText => Some(("Icon".to_string(), v[cursor_i].span)),
        Kind::MacroText => {
            let macro_ = v[cursor_i].span.extract(line);
            log::debug!("{macro_}");
            let message = if let Some((version, text)) = key_texts.get(macro_) {
                format!(
                    "Localization macro:{}\n{text}",
                    version.map(|v| v.to_string()).unwrap_or_else(String::new)
                )
            } else {
                "Macro".to_string()
            };

            Some((message, v[cursor_i].span))
        }
        _ => None,
    }
}

fn display_dtype_literal(literal: &str) -> &'static str {
    let inner = &literal[1..literal.len() - 1];

    for (prefix, literal) in [
        ("(int8)", "literal: int8"),
        ("(int16)", "literal: int16"),
        ("(int32)", "literal: int32"),
        ("(int64)", "literal: int64"),
        ("(uint8)", "literal: uint8"),
        ("(uint16)", "literal: uint16"),
        ("(uint32)", "literal: uint32"),
        ("(uint64)", "literal: uint64"),
        ("(CFixedPoint)", "literal: CFixedPoint"),
        ("(bool)", "literal: bool"),
        ("(float)", "literal: float"),
        ("(double)", "literal: double"),
        ("(CVector2f)", "literal: CVector2f"),
        ("(CVector2i)", "literal: CVector2i"),
        ("(CVector3f)", "literal: CVector3f"),
        ("(CVector3i)", "literal: CVector3i"),
        ("(CVector4f)", "literal: CVector4f"),
        ("(CVector4i)", "literal: CVector4i"),
    ] {
        if inner.starts_with(prefix) {
            return literal;
        }
    }

    "literal: CString"
}

fn display_dtypes(dtypes: &[Datatype]) -> String {
    #[allow(clippy::comparison_chain)]
    dtypes
        .iter()
        .enumerate()
        .map_while(|(i, d)| {
            if i < 6 {
                Some(d.to_string())
            } else if i == 6 {
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
                if i < 6 {
                    Some(c.to_string())
                } else if i == 6 {
                    Some("...".into())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(" | "),
    }
}
