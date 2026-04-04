use std::mem::take;

use crate::loca::{Kind, Node};
use crate::parse::util::Span;

pub fn parse_line(line: &str) -> Vec<Node> {
    #[derive(Debug, Default)]
    struct ParseContext {
        result: Vec<Node>,
        state: Expecting,
        span_start: usize,
        error: bool,
        inside: Kind,
    }
    impl ParseContext {
        fn push_simple(&mut self, kind: Kind, i: usize) {
            let kind = if self.error { Kind::Error } else { kind };
            self.error = false;
            self.result.push(Node { kind, content: vec![], span: Span::new(self.span_start, i) });
        }
        fn push(&mut self, kind: Kind, content: Vec<Node>, i: usize) {
            let kind = if self.error { Kind::Error } else { kind };
            self.error = false;
            self.result.push(Node { kind, content, span: Span::new(self.span_start, i) });
        }
    }
    #[derive(Debug, Default, Clone, Copy)]
    enum Expecting {
        /// Space at the start of the line
        #[default]
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

    let mut stack: Vec<ParseContext> = Vec::new();
    let mut context = ParseContext::default();

    macro_rules! pop_stack {
        ($end: expr) => {
            let result = take(&mut context.result);
            let inside = context.inside;
            context = stack.pop().expect("internal loca line parser error");
            context.push(inside, result, $end);
        };
    }

    #[allow(clippy::single_match)]
    for (i, c) in line.char_indices() {
        match context.state {
            Expecting::LeadingSpace => match c {
                ' ' => {}
                '#' => {
                    context.state = Expecting::Comment;
                    context.span_start = i;
                    break;
                }
                _ => {
                    if !(c.is_alphanumeric() || c == '_') {
                        context.error = true;
                    }
                    context.state = Expecting::Key;
                    context.span_start = i;
                }
            },
            Expecting::Key => match c {
                ':' => {
                    context.push_simple(Kind::Key, i);
                    context.state = Expecting::Num;
                    context.span_start = i + 1;
                }
                _ => {
                    if !(c.is_alphanumeric() || c == '_' || c == '.') {
                        context.error = true;
                    }
                }
            },
            Expecting::Num => match c {
                ' ' | '#' | '"' => {
                    if i > context.span_start {
                        context.push_simple(Kind::VersionNumber, i);
                    }
                    context.error = false;
                    match c {
                        ' ' => {
                            context.state = Expecting::OpenQuote;
                        }
                        '#' => {
                            context.state = Expecting::Comment;
                            context.span_start = i;
                            break;
                        }
                        '"' => {
                            context.state = Expecting::Freetext;
                            context.span_start = i + 1;
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    if !c.is_numeric() {
                        context.error = true;
                    }
                }
            },
            Expecting::OpenQuote => match c {
                ' ' => {}
                '#' => {
                    if context.error {
                        context.push_simple(Kind::Error, i);
                    }
                    context.state = Expecting::Comment;
                    context.span_start = i;
                    break;
                }
                '"' => {
                    if context.error {
                        context.push_simple(Kind::Error, i);
                    }
                    context.state = Expecting::Freetext;
                    context.span_start = i + 1;
                }
                _ => {
                    context.error = true;
                    context.span_start = i;
                }
            },
            Expecting::TrailingSpace => match c {
                '#' => {
                    if context.error {
                        context.push_simple(Kind::Error, i);
                    }
                    context.state = Expecting::Comment;
                    context.span_start = i;
                    break;
                }
                ' ' => {}
                _ => {
                    if !context.error {
                        context.error = true;
                        context.span_start = i;
                    }
                }
            },
            // Comments have jumped out of the loop, for efficiency.
            Expecting::Comment => unreachable!(),
            Expecting::Freetext => match c {
                '"' if i == last_dquote => {
                    if i > context.span_start {
                        context.push_simple(Kind::Text, i);
                    }
                    context.error = false;
                    context.state = Expecting::TrailingSpace;
                    context.span_start = i + 1;
                }
                '@' => {
                    if i > context.span_start {
                        context.push_simple(Kind::Text, i);
                    }
                    context.error = false;
                    context.span_start = i;
                    stack.push(take(&mut context));
                    context.state = Expecting::Icon;
                    context.inside = Kind::Icon;
                    context.span_start = i + 1;
                }
                '[' => {
                    if i > context.span_start {
                        context.push_simple(Kind::Text, i);
                    }
                    context.error = false;
                    context.span_start = i;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeExpr;

                    context.span_start = i + 1;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeCall;

                    context.span_start = i + 1;
                }
                '$' => {
                    if i > context.span_start {
                        context.push_simple(Kind::Text, i);
                    }
                    context.error = false;
                    context.span_start = i;
                    stack.push(take(&mut context));
                    context.state = Expecting::Macro;
                    context.inside = Kind::Macro;
                    context.span_start = i + 1;
                }
                _ => {}
            },
            Expecting::Icon => match c {
                '[' => {
                    if i > context.span_start {
                        context.push_simple(Kind::IconText, i);
                    }
                    context.error = false;
                    context.span_start = i;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeExpr;

                    context.span_start = i + 1;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeCall;

                    context.span_start = i + 1;
                }
                '!' => {
                    if i > context.span_start {
                        context.push_simple(Kind::IconText, i);
                    }
                    context.error = false;
                    pop_stack!(i + 1);
                    context.span_start = i + 1;
                }
                _ if c.is_alphanumeric() || c == '_' => {}
                _ => {
                    context.error = true;
                }
            },
            Expecting::Macro => match c {
                '|' => {
                    context.push_simple(Kind::MacroText, i);
                    context.state = Expecting::MacroFormat;
                    context.span_start = i;
                }
                '$' => {
                    context.push_simple(Kind::MacroText, i);
                    pop_stack!(i + 1);
                    context.span_start = i + 1;
                }
                _ if c.is_alphanumeric() || c == '_' || c == '.' => {}
                _ => {
                    context.error = true;
                }
            },
            Expecting::MacroFormat => match c {
                '$' => {
                    context.push_simple(Kind::Format, i);
                    pop_stack!(i + 1);
                    context.span_start = i + 1;
                }
                _ if c.is_alphanumeric() => {}
                _ => {
                    context.error = true;
                }
            },
            Expecting::DatatypeSpace => match c {
                ' ' => {}
                ']' => {
                    // Be forgiving about missing close parens, because the game engine is too.
                    while matches!(context.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                        && stack.last().is_some_and(|c| {
                            matches!(c.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                        })
                    {
                        pop_stack!(i);
                    }
                    pop_stack!(i + 1);
                    context.span_start = i + 1;
                }
                '(' => {
                    context.span_start = i + 1;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeExpr;
                    context.span_start = i + 1;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeCall;
                    context.span_start = i + 1;
                }
                ')' => {
                    // Be forgiving about extra close parens, because the game engine is too.
                    if stack.last().is_some_and(|c| c.inside == Kind::DatatypeCall) {
                        assert!(context.inside == Kind::DatatypeExpr);
                        pop_stack!(i); // pop the Expr
                        pop_stack!(i); // pop the Call
                    }
                    context.span_start = i + 1;
                }
                '.' => {
                    // Close the previous Call and open a new one.
                    if context.inside == Kind::DatatypeCall {
                        pop_stack!(i);
                    }
                    stack.push(take(&mut context));
                    // Spaces are not allowed after `.`
                    context.state = Expecting::DatatypeId;
                    context.inside = Kind::DatatypeCall;
                    context.span_start = i + 1;
                }
                ',' => {
                    // Close the previous Expr and open a new one.
                    if context.inside == Kind::DatatypeExpr {
                        pop_stack!(i);
                    }
                    context.span_start = i + 1;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeExpr;
                    context.span_start = i + 1;
                    stack.push(take(&mut context));
                    context.state = Expecting::DatatypeSpace;
                    context.inside = Kind::DatatypeCall;
                    context.span_start = i + 1;
                }
                '|' => {
                    // Be forgiving about missing close parens, because the game engine is too.
                    while matches!(context.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                        && stack.last().is_some_and(|c| {
                            matches!(c.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                        })
                    {
                        pop_stack!(i);
                    }
                    context.state = Expecting::DatatypeFormat;
                    context.span_start = i;
                }
                '\'' => {
                    context.state = Expecting::DatatypeLiteral;
                    context.span_start = i;
                }
                _ => {
                    if !c.is_alphabetic() {
                        context.error = true;
                    }
                    context.state = Expecting::DatatypeId;
                    context.span_start = i;
                }
            },
            Expecting::DatatypeId => match c {
                ']' | '|' | '.' | '(' | ')' | ',' | ' ' => {
                    context.push_simple(Kind::DatatypeId, i);
                    match c {
                        ']' => {
                            // Be forgiving about missing close parens, because the game engine is too.
                            while matches!(context.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                                && stack.last().is_some_and(|c| {
                                    matches!(c.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                                })
                            {
                                pop_stack!(i);
                            }
                            pop_stack!(i + 1);
                            context.span_start = i + 1;
                        }
                        '|' => {
                            // Be forgiving about missing close parens, because the game engine is too.
                            while matches!(context.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                                && stack.last().is_some_and(|c| {
                                    matches!(c.inside, Kind::DatatypeExpr | Kind::DatatypeCall)
                                })
                            {
                                pop_stack!(i);
                            }
                            context.state = Expecting::DatatypeFormat;
                            context.span_start = i;
                        }
                        '.' => {
                            // Close the previous Call and open a new one.
                            if context.inside == Kind::DatatypeCall {
                                pop_stack!(i);
                            }
                            stack.push(take(&mut context));
                            // Spaces are not allowed after `.`
                            context.state = Expecting::DatatypeId;
                            context.inside = Kind::DatatypeCall;
                            context.span_start = i + 1;
                        }
                        ')' => {
                            // Be forgiving about extra close parens, because the game engine is too.
                            if stack.last().is_some_and(|c| c.inside == Kind::DatatypeCall) {
                                assert!(context.inside == Kind::DatatypeExpr);
                                pop_stack!(i); // pop the Expr
                                pop_stack!(i); // pop the Call
                            }
                            context.span_start = i + 1;
                        }
                        '(' => {
                            context.span_start = i + 1;
                            stack.push(take(&mut context));
                            context.state = Expecting::DatatypeSpace;
                            context.inside = Kind::DatatypeExpr;
                            context.span_start = i + 1;
                            stack.push(take(&mut context));
                            context.state = Expecting::DatatypeSpace;
                            context.inside = Kind::DatatypeCall;
                            context.span_start = i + 1;
                        }
                        ',' => {
                            // Close the previous Expr and open a new one.
                            if context.inside == Kind::DatatypeExpr {
                                pop_stack!(i);
                            }
                            stack.push(take(&mut context));
                            context.state = Expecting::DatatypeSpace;
                            context.inside = Kind::DatatypeExpr;
                            context.span_start = i + 1;
                            stack.push(take(&mut context));
                            context.state = Expecting::DatatypeSpace;
                            context.inside = Kind::DatatypeCall;
                            context.span_start = i + 1;
                        }
                        ' ' => {
                            context.state = Expecting::DatatypeSpace;
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    if !(c.is_alphanumeric() || c == '_') {
                        context.error = true;
                    }
                }
            },
            Expecting::DatatypeFormat => match c {
                ']' => {
                    context.push_simple(Kind::Format, i);
                    pop_stack!(i + 1);
                    context.span_start = i + 1;
                }
                _ => {
                    if !c.is_alphanumeric() {
                        context.error = true;
                    }
                }
            },
            Expecting::DatatypeLiteral => match c {
                '\'' => {
                    context.push_simple(Kind::DatatypeLiteral, i + 1);
                    context.state = Expecting::DatatypeSpace;
                }
                _ => {}
            },
        }
    }
    // If something is left unterminated at the end of the line, clean it up here.
    // In most cases be forgiving, because the user may still be typing the line.
    while !stack.is_empty() {
        pop_stack!(line.len());
        context.span_start = line.len();
    }
    if line.len() > context.span_start {
        match context.state {
            Expecting::LeadingSpace => {}
            Expecting::OpenQuote | Expecting::TrailingSpace => {
                if context.error {
                    context.push_simple(Kind::Error, line.len());
                }
            }
            Expecting::Comment => {
                context.push_simple(Kind::Comment, line.len());
            }
            Expecting::Key => {
                context.push_simple(Kind::Key, line.len());
            }
            Expecting::Num => {
                context.push_simple(Kind::VersionNumber, line.len());
            }
            Expecting::Freetext => {
                context.push_simple(Kind::Text, line.len());
            }
            Expecting::Icon
            | Expecting::Macro
            | Expecting::MacroFormat
            | Expecting::DatatypeId
            | Expecting::DatatypeLiteral
            | Expecting::DatatypeSpace
            | Expecting::DatatypeFormat => unreachable!(),
        }
    }
    context.result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty() {
        let v = parse_line("");
        assert_eq!(v, vec![]);
    }

    #[test]
    fn whitespace() {
        let v = parse_line("  ");
        assert_eq!(v, vec![])
    }

    #[test]
    fn just_comment() {
        let v = parse_line("#comment");
        assert_eq!(v, vec![Node { kind: Kind::Comment, content: vec![], span: Span::new(0, 8) }]);
    }

    #[test]
    fn unfinished() {
        let v = parse_line("test: \"[GetSomething(GetSomething(");
        let keynode = Node { kind: Kind::Key, content: vec![], span: Span::new(0, 4) };
        let id1 = Node { kind: Kind::DatatypeId, content: vec![], span: Span::new(8, 20) };
        let id2 = Node { kind: Kind::DatatypeId, content: vec![], span: Span::new(21, 33) };
        // An empty call at the end
        let call3 = Node { kind: Kind::DatatypeCall, content: vec![], span: Span::new(34, 34) };
        let expr3 =
            Node { kind: Kind::DatatypeExpr, content: vec![call3], span: Span::new(34, 34) };
        let call2 =
            Node { kind: Kind::DatatypeCall, content: vec![id2, expr3], span: Span::new(21, 34) };
        let expr2 =
            Node { kind: Kind::DatatypeExpr, content: vec![call2], span: Span::new(21, 34) };
        let call1 =
            Node { kind: Kind::DatatypeCall, content: vec![id1, expr2], span: Span::new(8, 34) };
        let expr = Node { kind: Kind::DatatypeExpr, content: vec![call1], span: Span::new(7, 34) };
        assert_eq!(v, vec![keynode, expr]);
    }
}
