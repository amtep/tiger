use crate::parse::util::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocaTokenKind {
    Key,
    VersionNumber,
    Text,
    Comment,
    Icon,
    IconStart,
    IconEnd,
    Macro,
    DatatypeId(usize, usize),
    DatatypeLiteral,
    DatatypeFormat,
    Error,
}

pub fn parse_line(line: &str) -> Vec<(LocaTokenKind, Span)> {
    enum Expecting {
        /// Space at the start of the line
        LeadingSpace,
        /// The loca key at the start of the line (or the `l_language:` special key)
        Key,
        /// The optional number after the loca key
        Num,
        /// The double quote that opens the loca value
        OpenQuote,
        /// Space after the final closing dqoute
        TrailingSpace,
        /// Optional comment at the end of the line
        Comment,
        /// Text between special identifiers
        Freetext,
        /// @icon!
        Icon,
        /// $macro$
        Macro,
        /// $macro|q$ the q part
        MacroFormat,
        /// `[GetPlayer]` where there may be spaces before or after an id
        DatatypeSpace,
        /// `[GetPlayer]` inside the id
        DatatypeId,
        /// `[GetMaA('nomadic_riders')]` inside the literal
        DatatypeLiteral,
        /// `[concept|E]` the `|E` part
        DatatypeFormat,
    }

    let last_dquote = line.rfind('"').unwrap_or(line.len());
    let mut result = Vec::new();
    let mut state = Expecting::LeadingSpace;
    let mut span_start = 0;
    let mut error = false;
    let mut interrupted_icon = false;
    let mut expr_nr: usize = 0;
    let mut expr_depth: usize = 0;
    // INVARIANT: len is equal to expr_depth + 1 if within a datatype expression
    let mut chain_ids: Vec<usize> = Vec::new();
    let mut next_chain_id: usize = 0;

    #[allow(clippy::single_match)]
    for (i, c) in line.char_indices() {
        match state {
            Expecting::LeadingSpace => match c {
                ' ' => {}
                '#' => {
                    state = Expecting::Comment;
                    span_start = i;
                    break;
                }
                _ => {
                    if !(c.is_alphanumeric() || c == '_') {
                        error = true;
                    }
                    state = Expecting::Key;
                    span_start = i;
                }
            },
            Expecting::Key => match c {
                ':' => {
                    let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::Key };
                    result.push((ltk, Span::new(span_start, i)));
                    state = Expecting::Num;
                    span_start = i + 1;
                }
                _ => {
                    if !(c.is_alphanumeric() || c == '_' || c == '.') {
                        error = true;
                    }
                }
            },
            Expecting::Num => match c {
                ' ' | '#' | '"' => {
                    if i > span_start {
                        let ltk =
                            if error { LocaTokenKind::Error } else { LocaTokenKind::VersionNumber };
                        result.push((ltk, Span::new(span_start, i)));
                    }
                    error = false;
                    match c {
                        ' ' => {
                            state = Expecting::OpenQuote;
                        }
                        '#' => {
                            state = Expecting::Comment;
                            span_start = i;
                            break;
                        }
                        '"' => {
                            state = Expecting::Freetext;
                            span_start = i + 1;
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    if !c.is_numeric() {
                        error = true;
                    }
                }
            },
            Expecting::OpenQuote => match c {
                ' ' => {}
                '#' => {
                    if error {
                        result.push((LocaTokenKind::Error, Span::new(span_start, i)));
                    }
                    state = Expecting::Comment;
                    span_start = i;
                    break;
                }
                '"' => {
                    if error {
                        result.push((LocaTokenKind::Error, Span::new(span_start, i)));
                    }
                    state = Expecting::Freetext;
                    span_start = i + 1;
                }
                _ => {
                    error = true;
                    span_start = i;
                }
            },
            Expecting::TrailingSpace => match c {
                '#' => {
                    if error {
                        result.push((LocaTokenKind::Error, Span::new(span_start, i)));
                    }
                    state = Expecting::Comment;
                    span_start = i;
                    break;
                }
                ' ' => {}
                _ => {
                    error = true;
                    span_start = i;
                }
            },
            // Comments have jumped out of the loop, for efficiency.
            Expecting::Comment => unreachable!(),
            Expecting::Freetext => match c {
                '"' if i == last_dquote => {
                    if i > span_start {
                        result.push((LocaTokenKind::Text, Span::new(span_start, i)));
                    }
                    state = Expecting::TrailingSpace;
                }
                '@' => {
                    if i > span_start {
                        result.push((LocaTokenKind::Text, Span::new(span_start, i)));
                    }
                    state = Expecting::Icon;
                    span_start = i;
                }
                '[' => {
                    if i > span_start {
                        result.push((LocaTokenKind::Text, Span::new(span_start, i)));
                    }
                    state = Expecting::DatatypeSpace;
                    span_start = i + 1;
                    chain_ids.push(next_chain_id);
                    next_chain_id += 1;
                }
                '$' => {
                    if i > span_start {
                        result.push((LocaTokenKind::Text, Span::new(span_start, i)));
                    }
                    state = Expecting::Macro;
                    span_start = i;
                }
                _ => {}
            },
            Expecting::Icon => match c {
                '[' => {
                    if i > span_start {
                        let ltk =
                            if error { LocaTokenKind::Error } else { LocaTokenKind::IconStart };
                        result.push((ltk, Span::new(span_start, i)));
                    }
                    state = Expecting::DatatypeSpace;
                    interrupted_icon = true;
                    chain_ids.push(next_chain_id);
                    next_chain_id += 1;
                }
                '!' => {
                    let ltk = if error {
                        LocaTokenKind::Error
                    } else if interrupted_icon {
                        LocaTokenKind::IconEnd
                    } else {
                        LocaTokenKind::Icon
                    };
                    error = false;
                    interrupted_icon = false;
                    result.push((ltk, Span::new(span_start, i)));
                    state = Expecting::Freetext;
                    span_start = i + 1;
                }
                _ if c.is_alphanumeric() || c == '_' => {}
                _ => {
                    error = true;
                }
            },
            Expecting::Macro => match c {
                '|' => {
                    state = Expecting::MacroFormat;
                }
                '$' => {
                    let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::Macro };
                    error = false;
                    result.push((ltk, Span::new(span_start, i + 1)));
                    state = Expecting::Freetext;
                    span_start = i + 1;
                }
                _ if c.is_alphanumeric() || c == '_' || c == '.' => {}
                _ => {
                    error = true;
                }
            },
            Expecting::MacroFormat => match c {
                '$' => {
                    let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::Macro };
                    error = false;
                    result.push((ltk, Span::new(span_start, i + 1)));
                    state = Expecting::Freetext;
                    span_start = i + 1;
                }
                _ if c.is_alphanumeric() => {}
                _ => {
                    error = true;
                }
            },
            Expecting::DatatypeSpace => match c {
                ' ' => {}
                ']' => {
                    state = if interrupted_icon { Expecting::Icon } else { Expecting::Freetext };
                    span_start = i + 1;
                    expr_nr += 1;
                    expr_depth = 0;
                    chain_ids.clear();
                }
                '(' => {
                    expr_depth += 1;
                    chain_ids.push(next_chain_id);
                    next_chain_id += 1;
                }
                ')' => {
                    if expr_depth > 0 {
                        expr_depth -= 1;
                        chain_ids.pop();
                    }
                }
                '.' => {
                    state = Expecting::DatatypeId;
                    span_start = i + 1;
                }
                '|' => {
                    state = Expecting::DatatypeFormat;
                    span_start = i + 1;
                }
                '\'' => {
                    state = Expecting::DatatypeLiteral;
                    span_start = i;
                }
                _ => {
                    if !c.is_alphabetic() {
                        error = true;
                    }
                    state = Expecting::DatatypeId;
                    span_start = i;
                }
            },
            Expecting::DatatypeId => match c {
                ']' | '|' | '.' | '(' | ')' | ',' | ' ' => {
                    let ltk = if error {
                        LocaTokenKind::Error
                    } else {
                        LocaTokenKind::DatatypeId(expr_nr, chain_ids[expr_depth])
                    };
                    error = false;
                    result.push((ltk, Span::new(span_start, i)));
                    match c {
                        ']' => {
                            state = if interrupted_icon {
                                Expecting::Icon
                            } else {
                                Expecting::Freetext
                            };
                            span_start = i + 1;
                            expr_nr += 1;
                            expr_depth = 0;
                            chain_ids.clear();
                        }
                        '|' => {
                            state = Expecting::DatatypeFormat;
                            span_start = i + 1;
                        }
                        '.' => {
                            span_start = i + 1;
                        }
                        '(' => {
                            state = Expecting::DatatypeSpace;
                            expr_depth += 1;
                            chain_ids.push(next_chain_id);
                            next_chain_id += 1;
                        }
                        ')' => {
                            state = Expecting::DatatypeSpace;
                            if expr_depth > 0 {
                                expr_depth -= 1;
                                chain_ids.pop();
                            }
                        }
                        ',' | ' ' => {
                            state = Expecting::DatatypeSpace;
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    if !(c.is_alphanumeric() || c == '_') {
                        error = true;
                    }
                }
            },
            Expecting::DatatypeFormat => match c {
                ']' => {
                    let ltk =
                        if error { LocaTokenKind::Error } else { LocaTokenKind::DatatypeFormat };
                    error = false;
                    result.push((ltk, Span::new(span_start, i)));
                    state = if interrupted_icon { Expecting::Icon } else { Expecting::Freetext };
                    span_start = i + 1;
                    expr_nr += 1;
                    expr_depth = 0;
                    chain_ids.clear();
                }
                _ => {
                    if !c.is_alphanumeric() {
                        error = true;
                    }
                }
            },
            Expecting::DatatypeLiteral => match c {
                '\'' => {
                    result.push((LocaTokenKind::DatatypeLiteral, Span::new(span_start, i + 1)));
                    state = Expecting::DatatypeSpace;
                }
                _ => {}
            },
        }
    }
    // If something is left unterminated at the end of the line, clean it up here.
    // In most cases be forgiving, because the user may still be typing the line.
    // TODO: should the parser get the cursor position, to verify this?
    if line.len() > span_start {
        match state {
            Expecting::LeadingSpace | Expecting::DatatypeSpace => {}
            Expecting::OpenQuote | Expecting::TrailingSpace => {
                if error {
                    result.push((LocaTokenKind::Error, Span::new(span_start, line.len())));
                }
            }
            Expecting::Comment => {
                result.push((LocaTokenKind::Comment, Span::new(span_start, line.len())));
            }
            Expecting::Key => {
                let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::Key };
                result.push((ltk, Span::new(span_start, line.len())));
            }
            Expecting::Num => {
                let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::VersionNumber };
                result.push((ltk, Span::new(span_start, line.len())));
            }
            Expecting::Freetext => {
                result.push((LocaTokenKind::Text, Span::new(span_start, line.len())));
            }
            Expecting::Icon => {
                let ltk = if error {
                    LocaTokenKind::Error
                } else if interrupted_icon {
                    LocaTokenKind::IconEnd
                } else {
                    LocaTokenKind::Icon
                };
                result.push((ltk, Span::new(span_start, line.len())));
            }
            Expecting::Macro | Expecting::MacroFormat => {
                let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::Macro };
                result.push((ltk, Span::new(span_start, line.len())));
            }
            Expecting::DatatypeId => {
                let ltk = if error {
                    LocaTokenKind::Error
                } else {
                    LocaTokenKind::DatatypeId(expr_nr, chain_ids[expr_depth])
                };
                result.push((ltk, Span::new(span_start, line.len())));
            }
            Expecting::DatatypeLiteral => {
                result.push((LocaTokenKind::DatatypeLiteral, Span::new(span_start, line.len())));
            }
            Expecting::DatatypeFormat => {
                let ltk = if error { LocaTokenKind::Error } else { LocaTokenKind::DatatypeFormat };
                result.push((ltk, Span::new(span_start, line.len())));
            }
        }
    }
    result
}
