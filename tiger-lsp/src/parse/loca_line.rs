use std::mem::take;

use strum_macros::Display;

use crate::loca::{Kind, Node};
use crate::parse::util::Span;

#[derive(Debug)]
struct Parser {
    last_dquote: usize,
    stack: Vec<ParseContext>,
    context: ParseContext,
}

#[derive(Debug, Default)]
struct ParseContext {
    result: Vec<Node>,
    state: Expecting,
    span_start: usize,
    error: bool,
    inside: Kind,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Display)]
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

impl Parser {
    fn new(line: &str) -> Self {
        Self {
            last_dquote: line.rfind('"').unwrap_or(line.len()),
            stack: Vec::new(),
            context: ParseContext::default(),
        }
    }
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

impl Parser {
    fn pop_stack(&mut self, end: usize) {
        let result = take(&mut self.context.result);
        let inside = self.context.inside;
        self.context = self.stack.pop().expect("internal loca line parser error");
        self.context.push(inside, result, end);
    }

    /// Handle changes after a `[` is seen.
    fn open_datatype(&mut self, i: usize) {
        self.context.span_start = i;
        self.stack.push(take(&mut self.context));
        self.context.state = Expecting::DatatypeSpace;
        self.context.inside = Kind::DatatypeExpr;
        self.context.span_start = i + 1;
    }

    /// Handle changes after a `(` or `,` is seen.
    fn open_datatype_part(&mut self, i: usize) {
        self.context.span_start = i + 1;
        self.stack.push(take(&mut self.context));
        self.context.state = Expecting::DatatypeSpace;
        self.context.inside = Kind::DatatypeExpr;
        self.context.span_start = i + 1;
    }

    fn handle_leadingspace(&mut self, i: usize, c: char) {
        match c {
            ' ' => {}
            '#' => {
                self.context.state = Expecting::Comment;
                self.context.span_start = i;
            }
            _ => {
                if !(c.is_alphanumeric() || c == '_') {
                    self.context.error = true;
                }
                self.context.state = Expecting::Key;
                self.context.span_start = i;
            }
        }
    }

    fn handle_key(&mut self, i: usize, c: char) {
        match c {
            ':' => {
                self.context.push_simple(Kind::Key, i);
                self.context.state = Expecting::Num;
                self.context.span_start = i + 1;
            }
            _ => {
                if !(c.is_alphanumeric() || c == '_' || c == '.') {
                    self.context.error = true;
                }
            }
        }
    }

    fn handle_num(&mut self, i: usize, c: char) {
        match c {
            ' ' | '#' | '"' => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::VersionNumber, i);
                }
                self.context.error = false;
                self.context.state = Expecting::OpenQuote;
                self.handle_openquote(i, c);
            }
            _ => {
                if !c.is_numeric() {
                    self.context.error = true;
                }
            }
        }
    }

    fn handle_openquote(&mut self, i: usize, c: char) {
        match c {
            ' ' => {}
            '#' => {
                if self.context.error {
                    self.context.push_simple(Kind::Error, i);
                }
                self.context.state = Expecting::Comment;
                self.context.span_start = i;
            }
            '"' => {
                if self.context.error {
                    self.context.push_simple(Kind::Error, i);
                }
                self.context.state = Expecting::Freetext;
                self.context.span_start = i + 1;
            }
            _ => {
                self.context.error = true;
                self.context.span_start = i;
            }
        }
    }

    fn handle_trailingspace(&mut self, i: usize, c: char) {
        match c {
            '#' => {
                if self.context.error {
                    self.context.push_simple(Kind::Error, i);
                }
                self.context.state = Expecting::Comment;
                self.context.span_start = i;
            }
            ' ' => {}
            _ => {
                if !self.context.error {
                    self.context.error = true;
                    self.context.span_start = i;
                }
            }
        }
    }

    fn handle_freetext(&mut self, i: usize, c: char) {
        match c {
            '"' if i == self.last_dquote => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::Text, i);
                }
                self.context.error = false;
                self.context.state = Expecting::TrailingSpace;
                self.context.span_start = i + 1;
            }
            '@' => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::Text, i);
                }
                self.context.error = false;
                self.context.span_start = i;
                self.stack.push(take(&mut self.context));
                self.context.state = Expecting::Icon;
                self.context.inside = Kind::Icon;
                self.context.span_start = i + 1;
            }
            '[' => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::Text, i);
                }
                self.context.error = false;
                self.open_datatype(i);
            }
            '$' => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::Text, i);
                }
                self.context.error = false;
                self.context.span_start = i;
                self.stack.push(take(&mut self.context));
                self.context.state = Expecting::Macro;
                self.context.inside = Kind::Macro;
                self.context.span_start = i + 1;
            }
            _ => {}
        }
    }

    fn handle_icon(&mut self, i: usize, c: char) {
        match c {
            '[' => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::IconText, i);
                }
                self.context.error = false;
                self.open_datatype(i);
            }
            '!' => {
                if i > self.context.span_start {
                    self.context.push_simple(Kind::IconText, i);
                }
                self.context.error = false;
                self.pop_stack(i + 1);
                self.context.span_start = i + 1;
            }
            _ if c.is_alphanumeric() || c == '_' => {}
            _ => {
                self.context.error = true;
            }
        }
    }

    fn handle_macro(&mut self, i: usize, c: char) {
        match c {
            '|' => {
                self.context.push_simple(Kind::MacroText, i);
                self.context.state = Expecting::MacroFormat;
                self.context.span_start = i;
            }
            '$' => {
                self.context.push_simple(Kind::MacroText, i);
                self.pop_stack(i + 1);
                self.context.span_start = i + 1;
            }
            _ if c.is_alphanumeric() || c == '_' || c == '.' => {}
            _ => {
                self.context.error = true;
            }
        }
    }

    fn handle_macroformat(&mut self, i: usize, c: char) {
        match c {
            '$' => {
                self.context.push_simple(Kind::Format, i);
                self.pop_stack(i + 1);
                self.context.span_start = i + 1;
            }
            _ if c.is_alphanumeric() => {}
            _ => {
                self.context.error = true;
            }
        }
    }

    fn handle_datatypespace(&mut self, i: usize, c: char) {
        match c {
            ' ' => {}
            ']' => {
                // Be forgiving about missing close parens, because the game engine is too.
                while self.context.inside == Kind::DatatypeExpr
                    && self.stack.last().is_some_and(|c| c.inside == Kind::DatatypeExpr)
                {
                    self.pop_stack(i);
                }
                self.pop_stack(i + 1);
                self.context.span_start = i + 1;
            }
            '(' => {
                self.open_datatype_part(i);
            }
            ')' => {
                // Be forgiving about extra close parens, because the game engine is too.
                if self.stack.last().is_some_and(|c| c.inside == Kind::DatatypeExpr) {
                    assert!(self.context.inside == Kind::DatatypeExpr);
                    self.pop_stack(i);
                }
                self.context.span_start = i + 1;
            }
            '.' => {
                // Spaces are not allowed after `.`
                self.context.state = Expecting::DatatypeId;
                self.context.span_start = i + 1;
            }
            ',' => {
                // Close the previous Expr and open a new one.
                if self.context.inside == Kind::DatatypeExpr {
                    self.pop_stack(i);
                }
                self.open_datatype_part(i);
            }
            '|' => {
                // Be forgiving about missing close parens, because the game engine is too.
                while self.context.inside == Kind::DatatypeExpr
                    && self.stack.last().is_some_and(|c| c.inside == Kind::DatatypeExpr)
                {
                    self.pop_stack(i);
                }
                self.context.state = Expecting::DatatypeFormat;
                self.context.span_start = i;
            }
            '\'' => {
                self.context.state = Expecting::DatatypeLiteral;
                self.context.span_start = i;
            }
            _ => {
                if !c.is_alphabetic() {
                    self.context.error = true;
                }
                self.context.state = Expecting::DatatypeId;
                self.context.span_start = i;
            }
        }
    }

    fn handle_datatypeid(&mut self, i: usize, c: char) {
        match c {
            ']' | '|' | '.' | '(' | ')' | ',' | ' ' => {
                self.context.push_simple(Kind::DatatypeId, i);
                self.context.state = Expecting::DatatypeSpace;
                self.handle_datatypespace(i, c);
            }
            _ => {
                if !(c.is_alphanumeric() || c == '_') {
                    self.context.error = true;
                }
            }
        }
    }

    fn handle_datatypeformat(&mut self, i: usize, c: char) {
        match c {
            ']' => {
                self.context.push_simple(Kind::Format, i);
                self.pop_stack(i + 1);
                self.context.span_start = i + 1;
            }
            _ => {
                if !c.is_alphanumeric() {
                    self.context.error = true;
                }
            }
        }
    }

    fn handle_datatypeliteral(&mut self, i: usize, c: char) {
        #[allow(clippy::single_match)] // for consistency
        match c {
            '\'' => {
                self.context.push_simple(Kind::DatatypeLiteral, i + 1);
                self.context.state = Expecting::DatatypeSpace;
            }
            _ => {}
        }
    }

    fn handle_char(&mut self, i: usize, c: char) {
        eprintln!("{}", self.context.state);
        match self.context.state {
            Expecting::LeadingSpace => self.handle_leadingspace(i, c),
            Expecting::Key => self.handle_key(i, c),
            Expecting::Num => self.handle_num(i, c),
            Expecting::OpenQuote => self.handle_openquote(i, c),
            Expecting::TrailingSpace => self.handle_trailingspace(i, c),
            // Comments have jumped out of the loop, for efficiency.
            Expecting::Comment => unreachable!(),
            Expecting::Freetext => self.handle_freetext(i, c),
            Expecting::Icon => self.handle_icon(i, c),
            Expecting::Macro => self.handle_macro(i, c),
            Expecting::MacroFormat => self.handle_macroformat(i, c),
            Expecting::DatatypeSpace => self.handle_datatypespace(i, c),
            Expecting::DatatypeId => self.handle_datatypeid(i, c),
            Expecting::DatatypeFormat => self.handle_datatypeformat(i, c),
            Expecting::DatatypeLiteral => self.handle_datatypeliteral(i, c),
        }
    }

    fn cleanup(&mut self, i: usize) {
        // If something is left unterminated at the end of the line, clean it up here.
        // In most cases be forgiving, because the user may still be typing the line.
        while !self.stack.is_empty() {
            self.pop_stack(i);
            self.context.span_start = i;
        }
        if i > self.context.span_start {
            match self.context.state {
                Expecting::LeadingSpace => {}
                Expecting::OpenQuote | Expecting::TrailingSpace => {
                    if self.context.error {
                        self.context.push_simple(Kind::Error, i);
                    }
                }
                Expecting::Comment => {
                    self.context.push_simple(Kind::Comment, i);
                }
                Expecting::Key => {
                    self.context.push_simple(Kind::Key, i);
                }
                Expecting::Num => {
                    self.context.push_simple(Kind::VersionNumber, i);
                }
                Expecting::Freetext => {
                    self.context.push_simple(Kind::Text, i);
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
    }
}

pub fn parse_line(line: &str) -> Vec<Node> {
    let mut parser = Parser::new(line);

    for (i, c) in line.char_indices() {
        parser.handle_char(i, c);
        // Shortcut for efficiency
        if parser.context.state == Expecting::Comment {
            break;
        }
    }

    parser.cleanup(line.len());
    parser.context.result
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
        // empty expr at the end
        let expr3 = Node { kind: Kind::DatatypeExpr, content: vec![], span: Span::new(34, 34) };
        let expr2 =
            Node { kind: Kind::DatatypeExpr, content: vec![id2, expr3], span: Span::new(21, 34) };
        let expr =
            Node { kind: Kind::DatatypeExpr, content: vec![id1, expr2], span: Span::new(7, 34) };
        assert_eq!(v, vec![keynode, expr]);
    }

    #[test]
    fn gameconcept() {
        let v = parse_line("test: \"[concept|E]\"");
        let key = Node { kind: Kind::Key, content: vec![], span: Span::new(0, 4) };
        let id = Node { kind: Kind::DatatypeId, content: vec![], span: Span::new(8, 15) };
        let format = Node { kind: Kind::Format, content: vec![], span: Span::new(15, 17) };
        let expr =
            Node { kind: Kind::DatatypeExpr, content: vec![id, format], span: Span::new(7, 18) };
        assert_eq!(v, vec![key, expr])
    }

    #[test]
    fn interrupted_icon() {
        let line = "test: \"@[MEN_AT_ARMS_TYPE.GetIconKey]_icon!";
        let v = parse_line(line);
        let key = Node { kind: Kind::Key, content: vec![], span: Span::new(0, 4) };
        let id1 = Node { kind: Kind::DatatypeId, content: vec![], span: Span::new(9, 25) };
        let id2 = Node { kind: Kind::DatatypeId, content: vec![], span: Span::new(26, 36) };
        let expr =
            Node { kind: Kind::DatatypeExpr, content: vec![id1, id2], span: Span::new(8, 37) };
        let text =
            Node { kind: Kind::IconText, content: vec![], span: Span::new(37, line.len() - 1) };
        let icon =
            Node { kind: Kind::Icon, content: vec![expr, text], span: Span::new(7, line.len()) };
        assert_eq!(v, vec![key, icon])
    }
}
