use std::borrow::Cow;

use log::{error, warn};
use lsp_types::{CompletionItem, CompletionItemKind};

use tiger_tables::datatype::{Arg, Args, Datatype};
use tiger_tables::game::Game;

use crate::datatype_tables::DatatypeTables;
use crate::loca::{Kind, find_cursor_index};
use crate::parse::loca_line::parse_line;
use crate::util::{HashSet, tree};

#[derive(Debug, Default)]
pub struct Completion {
    pub commit_characters_support: bool,
}

impl Completion {
    pub fn completions(
        &self,
        game: Game,
        tables: &DatatypeTables,
        line: &str,
        cursor: usize,
    ) -> Option<Vec<CompletionItem>> {
        let mut v = &parse_line(line);
        let mut cursor_i = find_cursor_index(v, cursor)?;
        log::trace!("\n{}", tree(&v[cursor_i], line));

        // Find the datatype expression in the parse result. (It may be hidden inside an Icon or something)
        while v[cursor_i].kind != Kind::DatatypeExpr && !v[cursor_i].content.is_empty() {
            v = &v[cursor_i].content;
            cursor_i = find_cursor_index(v, cursor)?;
        }
        // We only do completion for datatype expressions for now.
        // TODO: handle icons, macros, and formatting too.
        if v[cursor_i].kind != Kind::DatatypeExpr {
            return None;
        }

        // Enter the datatype expression we found.
        // v will now be a sequence starting with a DatatypeId or DatatypeLiteral,
        // followed by zero or more DatatypeExpr for its arguments, possibly followed
        // by another for the next promote or function, etc.
        v = &v[cursor_i].content;
        cursor_i = find_cursor_index(v, cursor)?;

        if v[cursor_i].kind == Kind::Format {
            // There's no useful completion for formats, since they are strings of single-letter codes.
            return None;
        }

        // Examine the entire expression up to the cursor, looking for the parent args the
        // cursor's node has to fulfill and the incoming datatypes of a promote or function.
        // (incoming datatype will be None for the first (global) promote or function)
        //
        // `parent_arg_possibilities` will be initialized to a fake argument that expects a CString
        // for the whole expression.
        // TODO: This is not correct, since more types than CString can be inlined into a localization.
        // Find the whole list.
        let mut parent_arg_possibilities = vec![Arg::DType(Datatype::CString)];
        loop {
            // datatypes that can come out of the previous promote (will be None for globals)
            let mut incoming_dtypes: Option<Vec<Datatype>> = None;
            let mut possibilities = vec![];
            let mut possibilities_i = 0;
            for (i, node) in v[..cursor_i].iter().enumerate() {
                match node.kind {
                    Kind::DatatypeLiteral => {
                        // Don't know what to do with literals that are not simple arguments,
                        // and if we got here then there is a literal *before* the cursor.
                        return None;
                    }
                    Kind::DatatypeExpr => {
                        // TODO: check its output dtype to narrow down the current possibilities list
                    }
                    Kind::DatatypeId => {
                        let id = node.span.extract(line);
                        // Count the number of following DatatypeExpr to indicate how many arguments the
                        // current DatatypeId expects. However, don't go past the cursor to determine this.
                        let nargs_range = {
                            let nargs = v[i + 1..]
                                .iter()
                                .take_while(|node| node.kind == Kind::DatatypeExpr)
                                .count();
                            // The first range is to usize::MAX because an unbounded range would be a different
                            // type than the nargs..=nargs one
                            if i + nargs >= cursor_i {
                                (cursor_i - i)..=usize::MAX
                            } else {
                                nargs..=nargs
                            }
                        };
                        let can_be_function =
                            !v[i + 1..].iter().any(|node| node.kind == Kind::DatatypeId);
                        possibilities.clear();
                        possibilities_i = i;
                        if let Some(mut incoming_dtypes) = incoming_dtypes {
                            // TODO: remove need to clone here
                            possibilities = tables
                                .lookup_promote(game, &mut incoming_dtypes.clone(), id)
                                .unwrap_or_default();
                            if can_be_function {
                                possibilities.append(
                                    &mut tables
                                        .lookup_function(game, &mut incoming_dtypes, id)
                                        .unwrap_or_default(),
                                );
                            }
                        } else {
                            if let Some((args, dtype)) = tables.lookup_global_promote(game, id) {
                                possibilities.push((args, vec![dtype]));
                            }
                            if can_be_function
                                && let Some((args, dtype)) = tables.lookup_global_function(game, id)
                            {
                                possibilities.push((args, vec![dtype]));
                            }
                            if let Ok(dtype) = Datatype::from_str(game, id) {
                                possibilities.push((Args::Args(&[]), vec![dtype]));
                            }
                            // We don't have to consider game concepts here, because those can
                            // never precede the cursor.
                        }
                        possibilities.retain(|(args, _)| match args {
                            Args::Unknown => true,
                            Args::Args(a) => nargs_range.contains(&a.len()),
                        });
                        if possibilities.is_empty() {
                            possibilities.push((Args::Unknown, vec![Datatype::Unknown]));
                        }

                        incoming_dtypes = Some(
                            possibilities.iter().flat_map(|(_, dtypes)| dtypes).copied().collect(),
                        );
                    }
                    // We shouldn't find anything else in a DatatypeExpr since we already ruled out Format.
                    _ => {
                        warn!(
                            "internal error: unexpected node kind in completion loop: {}",
                            node.kind
                        );
                        return None;
                    }
                }
            }
            match v[cursor_i].kind {
                Kind::DatatypeExpr => {
                    // Prepare to enter the next level of arguments.
                    // Update the parent arg.
                    if possibilities_i + 1 > cursor_i {
                        error!(
                            "internal error: completion loop {possibilities_i} + 1 > {cursor_i}"
                        );
                        return None;
                    }
                    let args_nr = cursor_i - possibilities_i - 1;
                    parent_arg_possibilities = possibilities
                        .iter()
                        .filter_map(|(a, _)| match a {
                            Args::Unknown => Some(Arg::DType(Datatype::Unknown)),
                            Args::Args(a) => a.get(args_nr).copied(),
                        })
                        .collect();
                    // Enter the next level of expression (must be an argument of the latest id)
                    v = &v[cursor_i].content;
                    cursor_i = find_cursor_index(v, cursor)?;
                }
                Kind::DatatypeId => {
                    let mut candidates: Vec<(
                        Cow<'static, str>,
                        Cow<'static, str>,
                        CompletionItemKind,
                    )>;
                    let outtypes: HashSet<Datatype> = parent_arg_possibilities
                        .iter()
                        .map(|arg| match arg {
                            Arg::DType(dtype) => *dtype,
                            Arg::IType(_) | Arg::Choice(_) => Datatype::CString,
                        })
                        .collect();
                    if let Some(incoming_dtypes) = incoming_dtypes {
                        let intypes: HashSet<Datatype> = incoming_dtypes.into_iter().collect();
                        candidates = tables
                            .list_promotes(game, &intypes)
                            .map(|(name, _, args, _)| {
                                (Cow::Borrowed(name), args_detail(args), CompletionItemKind::METHOD)
                            })
                            .collect();
                        candidates.extend(tables.list_functions(game, &intypes, &outtypes).map(
                            |(name, _, args, _)| {
                                (
                                    Cow::Borrowed(name),
                                    args_detail(args),
                                    CompletionItemKind::FUNCTION,
                                )
                            },
                        ));
                    } else {
                        fn dtype_filter(
                            dtype: Datatype,
                        ) -> Option<(Cow<'static, str>, Cow<'static, str>, CompletionItemKind)>
                        {
                            if dtype.can_literal() {
                                Some((
                                    Cow::Owned(format!("'({dtype})")),
                                    Cow::Borrowed(""),
                                    CompletionItemKind::TEXT,
                                ))
                            } else if dtype.can_global_promote() {
                                // TODO: make this Borrowed
                                Some((
                                    Cow::Owned(dtype.to_string()),
                                    Cow::Borrowed(""),
                                    CompletionItemKind::CLASS,
                                ))
                            } else {
                                None
                            }
                        }
                        candidates = tables
                            .list_global_promotes(game)
                            .map(|(name, args, _)| {
                                (Cow::Borrowed(name), args_detail(args), CompletionItemKind::METHOD)
                            })
                            .collect();
                        candidates.extend(tables.list_global_functions(game, &outtypes).map(
                            |(name, args, _)| {
                                (
                                    Cow::Borrowed(name),
                                    args_detail(args),
                                    CompletionItemKind::FUNCTION,
                                )
                            },
                        ));
                        if outtypes.contains(&Datatype::Unknown)
                            || outtypes.contains(&Datatype::AnyScope)
                        {
                            candidates
                                .extend(Datatype::list_datatypes(game).filter_map(dtype_filter));
                        } else {
                            candidates.extend(outtypes.iter().copied().filter_map(dtype_filter));
                        }
                        // TODO: game concepts
                        // TODO: IType and Choice args
                    }
                    return Some(
                        candidates
                            .iter()
                            .map(|(label, details, kind)| {
                                let commit_v = self.commit_characters_support.then(|| {
                                    let mut commit_v =
                                        vec![")".to_string(), "]".to_string(), "|".to_string()];
                                    if details.starts_with('(') {
                                        commit_v.push("(".to_string());
                                    }
                                    if *kind == CompletionItemKind::METHOD {
                                        commit_v.push(".".to_string());
                                    }
                                    commit_v
                                });
                                CompletionItem {
                                    label: label.to_string(),
                                    detail: if details.is_empty() {
                                        None
                                    } else {
                                        Some(details.to_string())
                                    },
                                    kind: Some(*kind),
                                    commit_characters: commit_v,
                                    ..Default::default()
                                }
                            })
                            .collect(),
                    );
                }
                _ => {
                    warn!(
                        "internal error: unexpected node kind in completion: {}",
                        v[cursor_i].kind
                    );
                    return None;
                }
            }
        }
    }
}

fn args_detail(args: Args) -> Cow<'static, str> {
    match args {
        Args::Unknown => Cow::Borrowed(""),
        Args::Args(a) => {
            if a.is_empty() {
                Cow::Borrowed("")
            } else {
                Cow::Owned(format!(
                    "({})",
                    a.iter().map(|arg| arg_detail(*arg)).collect::<Vec<_>>().join(", ")
                ))
            }
        }
    }
}

fn arg_detail(arg: Arg) -> String {
    match arg {
        Arg::DType(dtype) => dtype.to_string(),
        Arg::IType(itype) => format!("{itype} CString"),
        Arg::Choice(_) => "CString".to_string(),
    }
}
