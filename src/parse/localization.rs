use std::iter::Peekable;
use std::mem::take;
use std::str::Chars;

use crate::data::localization::{Language, LocaEntry, LocaValue, MacroValue};
use crate::datatype::{Code, CodeArg, CodeChain};
use crate::fileset::FileEntry;
use crate::game::Game;
use crate::parse::cob::Cob;
use crate::parse::ignore::{parse_comment, IgnoreFilter, IgnoreSize};
use crate::report::register_ignore_filter;
use crate::report::{untidy, warn, ErrorKey};
use crate::token::{leak, Loc, Token};

fn is_key_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '\''
}

// code_char might end up being identical to key_char, since we can write [gameconcept] and
// game concepts can be any id
fn is_code_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\''
}

#[derive(Clone, Debug)]
struct LocaParser {
    loc: Loc,
    offset: usize,
    content: &'static str,
    chars: Peekable<Chars<'static>>,
    language: Language,
    expecting_language: bool,
    loca_end: usize,
    value: Vec<LocaValue>,
    pending_line_ignores: Vec<IgnoreFilter>,
    active_range_ignores: Vec<(u32, IgnoreFilter)>,
}

impl LocaParser {
    fn new(entry: &FileEntry, content: &'static str, lang: Language) -> Self {
        let mut chars = content.chars().peekable();
        let mut offset = 0;

        let mut loc = Loc::from(entry);
        // loc.line == 0 making this a whole file report
        loc.column = 1; // From our perspective the BOM is a character and needs to be included in column offset

        if chars.peek() == Some(&'\u{feff}') {
            offset += '\u{feff}'.len_utf8();
            loc.column += 1;
            chars.next();

            if chars.peek() == Some(&'\u{feff}') {
                // Second BOM is file content, not header, and should be reported with line number
                loc.line = 1;
                let msg = "double BOM in localization file";
                let info = "This will make the game engine skip the whole file.";
                warn(ErrorKey::Encoding).strong().msg(msg).info(info).loc(loc).push();
                offset += '\u{feff}'.len_utf8();
                loc.column += 1;
                chars.next();
            }
        } else {
            warn(ErrorKey::Encoding).msg("Expected UTF-8 BOM encoding").abbreviated(loc).push();
        }

        // From here on we are reporting on file content
        loc.line = 1;

        LocaParser {
            loc,
            offset,
            content,
            chars,
            language: lang,
            expecting_language: true,
            value: Vec::new(),
            loca_end: 0,
            pending_line_ignores: Vec::new(),
            active_range_ignores: Vec::new(),
        }
    }

    fn next_char(&mut self) {
        // self.loc is always the loc of the peekable char
        if let Some(c) = self.chars.next() {
            self.offset += c.len_utf8();
            if c == '\n' {
                self.loc.line += 1;
                self.loc.column = 1;
            } else {
                self.loc.column += 1;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_linear_whitespace(&mut self) {
        while let Some(c) = self.chars.peek() {
            if c.is_whitespace() && *c != '\n' {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_line(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            self.next_char();
        }
        self.next_char(); // Eat the newline
    }

    // This function returns an Option so that the caller can return
    // its value without further boilerplate.
    #[allow(clippy::unnecessary_wraps)]
    fn error_line(&mut self, key: Token) -> Option<LocaEntry> {
        self.skip_line();
        Some(LocaEntry::new(key, LocaValue::Error, None))
    }

    fn get_key(&mut self) -> Token {
        let loc = self.loc;
        let start_offset = self.offset;
        while let Some(c) = self.chars.peek() {
            if is_key_char(*c) {
                self.next_char();
            } else {
                break;
            }
        }
        let s = &self.content[start_offset..self.offset];
        Token::from_static_str(s, loc)
    }

    fn unexpected_char(&mut self, expected: &str) {
        match self.chars.peek() {
            None => warn(ErrorKey::Localization)
                .msg(format!("Unexpected end of file, {expected}"))
                .loc(self.loc)
                .push(),
            Some(c) => warn(ErrorKey::Localization)
                .msg(format!("Unexpected character `{c}`, {expected}"))
                .loc(self.loc)
                .push(),
        }
    }

    // Look ahead to the last `"` on the line
    fn find_dquote(&self) -> Option<usize> {
        let mut offset = self.offset;
        let mut dquote_offset = None;
        for c in self.chars.clone() {
            if c == '"' {
                dquote_offset = Some(offset);
            } else if c == '\n' {
                return dquote_offset;
            }
            offset += c.len_utf8();
        }
        dquote_offset
    }

    fn parse_format(&mut self) -> Option<Token> {
        (self.chars.peek() == Some(&'|')).then(|| {
            self.next_char(); // eat the |
            let loc = self.loc;
            let mut text = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == '$' || c == ']' || c == '\n' {
                    break;
                }
                text.push(c);
                self.next_char();
            }
            Token::new(&text, loc)
        })
    }

    fn line_has_macros(&self) -> bool {
        for c in self.chars.clone() {
            if c == '\n' {
                return false;
            } else if c == '$' {
                return true;
            }
        }
        false
    }

    fn parse_macros(&mut self) {
        // TODO: vanilla uses $[DATE_MIN.GetStringShort|V]$ which breaks all my assumptions
        let mut v = Vec::new();
        let mut loc = self.loc;
        let mut offset = self.offset;
        while let Some(&c) = self.chars.peek() {
            if c == '$' {
                let s = &self.content[offset..self.offset];
                v.push(MacroValue::Text(Token::from_static_str(s, loc)));

                if let Some(mv) = self.parse_keyword() {
                    v.push(mv);
                } else {
                    self.value.push(LocaValue::Error);
                    return;
                }
                loc = self.loc;
                offset = self.offset;
            } else if c == '"' && self.offset == self.loca_end {
                let s = &self.content[offset..self.offset];
                v.push(MacroValue::Text(Token::from_static_str(s, loc)));
                self.value.push(LocaValue::Macro(v));
                self.next_char();
                return;
            } else {
                self.next_char();
            }
        }
        let s = &self.content[offset..self.offset];
        v.push(MacroValue::Text(Token::from_static_str(s, loc)));
        self.value.push(LocaValue::Macro(v));
    }

    fn parse_keyword(&mut self) -> Option<MacroValue> {
        self.next_char(); // Skip the $
        let loc = self.loc;
        let start_offset = self.offset;
        let key = self.get_key();
        let end_offset = self.offset;
        self.parse_format();
        if self.chars.peek() != Some(&'$') {
            // TODO: check if there is a closing $, adapt warning text
            let msg = "didn't recognize a key between $";
            warn(ErrorKey::Localization).weak().msg(msg).loc(key).push();
            return None;
        }
        self.next_char();
        let s = &self.content[start_offset..end_offset];
        Some(MacroValue::Keyword(Token::from_static_str(s, loc)))
    }

    fn get_rest_of_line(&mut self) -> &str {
        let start_offset = self.offset;
        while let Some(&c) = self.chars.peek() {
            if c == '\n' {
                break;
            }
            self.next_char();
        }
        let end_offset = self.offset;
        self.next_char(); // Eat the newline
        &self.content[start_offset..end_offset]
    }

    fn skip_until_key(&mut self) {
        loop {
            // Skip comments and blank lines
            self.skip_whitespace();
            if self.chars.peek() == Some(&'#') {
                self.next_char();
                if let Some(spec) = parse_comment(self.get_rest_of_line()) {
                    match spec.size {
                        IgnoreSize::Line => self.pending_line_ignores.push(spec.filter),
                        IgnoreSize::Block => (),
                        IgnoreSize::File => register_ignore_filter(
                            self.loc.pathname().to_path_buf(),
                            ..,
                            spec.filter,
                        ),
                        IgnoreSize::Begin => {
                            self.active_range_ignores.push((self.loc.line + 1, spec.filter));
                        }
                        IgnoreSize::End => {
                            if let Some((start_line, filter)) = self.active_range_ignores.pop() {
                                let path = self.loc.pathname().to_path_buf();
                                register_ignore_filter(path, start_line..self.loc.line, filter);
                            }
                        }
                    }
                }
                continue;
            }

            match self.chars.peek() {
                Some(&c) if is_key_char(c) => break,
                Some(_) => {
                    self.unexpected_char("expected localization key");
                    self.skip_line();
                }
                None => break,
            }
        }
    }

    /// Return the next `LocaEntry`, or None if there are no more in the file.
    fn parse_loca(&mut self) -> Option<LocaEntry> {
        // Loop until we have a key. Once we have a key, we'll definitely
        // return a LocaEntry for the current line, though it might be an Error entry.
        self.skip_until_key();
        self.chars.peek()?;
        for filter in self.pending_line_ignores.drain(..) {
            let path = self.loc.pathname().to_path_buf();
            let line = self.loc.line;
            register_ignore_filter(path, line..=line, filter);
        }

        let key = self.get_key();
        self.skip_linear_whitespace();
        if self.chars.peek() == Some(&':') {
            self.next_char();
        } else {
            self.unexpected_char("expected `:`");
            return self.error_line(key);
        }

        // Skip optional number after :
        while let Some(c) = self.chars.peek() {
            if c.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        self.skip_linear_whitespace();

        // Now we should see the value. But what if the line ends here?
        if matches!(self.chars.peek(), Some('#' | '\n') | None) {
            if self.expecting_language {
                if !key.is(&format!("l_{}", self.language)) {
                    let msg = format!("wrong language header, should be `l_{}:`", self.language);
                    warn(ErrorKey::Localization).msg(msg).loc(key).push();
                }
                self.expecting_language = false;
                self.skip_line();
                // Recursing here is safe because it can happen only once.
                return self.parse_loca();
            }
            warn(ErrorKey::Localization).msg("key with no value").loc(&key).push();
            return self.error_line(key);
        } else if self.expecting_language {
            let msg = format!("expected language header `l_{}:`", self.language);
            warn(ErrorKey::Localization).msg(msg).loc(&key).push();
            self.expecting_language = false;
            // Continue to parse this entry as usual
        }
        if self.chars.peek() == Some(&'"') {
            self.next_char();
        } else {
            self.unexpected_char("expected `\"`");
            return self.error_line(key);
        }

        // We need to pre-parse because the termination of localization entries
        // is ambiguous. A loca value ends at the last " on the line.
        // Any # or " before that are part of the value; an # after that
        // introduces a comment.
        if let Some(i) = self.find_dquote() {
            self.loca_end = i;
        } else {
            let msg = "localization entry without ending quote";
            warn(ErrorKey::Localization).msg(msg).loc(self.loc).push();
            return self.error_line(key);
        }

        self.value = Vec::new();
        let s = &self.content[self.offset..self.loca_end];
        let token = Token::from_static_str(s, self.loc);

        // We also need to pre-parse because $macros$ can appear anywhere and
        // we don't know how to parse the results until we know what to
        // substitute. If there are macros in the line, return it as a special
        // `LocaValue::Macro` array
        if self.line_has_macros() {
            self.parse_macros();
            if matches!(self.value.last(), Some(&LocaValue::Error)) {
                return self.error_line(key);
            }
        } else {
            self.value = ValueParser::new(vec![&token]).parse_vec();
            while self.offset <= self.loca_end {
                self.next_char();
            }
        }

        self.skip_linear_whitespace();
        match self.chars.peek() {
            None | Some('#' | '\n') => (),
            _ => {
                let msg = "content after final `\"` on line";
                warn(ErrorKey::Localization).strong().msg(msg).loc(self.loc).push();
            }
        }

        self.skip_line();
        let value = if self.value.len() == 1 {
            self.value.remove(0)
        } else {
            LocaValue::Concat(take(&mut self.value))
        };
        Some(LocaEntry::new(key, value, Some(token)))
    }
}

pub struct ValueParser<'a> {
    loc: Loc,
    offset: usize,
    content: Vec<&'a Token>,
    content_iters: Vec<Peekable<Chars<'a>>>,
    content_idx: usize,
    value: Vec<LocaValue>,
}

// TODO: some duplication of helper functions between `LocaParser` and `ValueParser`
impl<'a> ValueParser<'a> {
    pub fn new(content: Vec<&'a Token>) -> Self {
        assert!(!content.is_empty());

        Self {
            loc: content[0].loc,
            offset: 0,
            content_iters: content.iter().map(|t| t.as_str().chars().peekable()).collect(),
            content,
            content_idx: 0,
            value: Vec::new(),
        }
    }

    fn peek(&mut self) -> Option<char> {
        let p = self.content_iters[self.content_idx].peek();
        if p.is_none() {
            if self.content_idx + 1 == self.content.len() {
                None
            } else {
                self.content_idx += 1;
                self.loc = self.content[self.content_idx].loc;
                self.offset = 0;
                self.peek()
            }
        } else {
            p.copied()
        }
    }

    fn next_char(&mut self) {
        // self.peek advances content_idx if needed
        if self.peek().is_some() {
            let c = self.content_iters[self.content_idx].next().unwrap();
            self.offset += c.len_utf8();
            self.loc.column += 1;
        }
    }

    fn start_text(&self) -> Cob {
        let mut cob = Cob::new();
        cob.set(self.content[self.content_idx].as_str(), self.offset, self.loc);
        cob
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn unexpected_char(&mut self, expected: &str, errorkey: ErrorKey) {
        // TODO: handle EOF better
        let c = self.peek().unwrap_or(' ');
        let msg = format!("Unexpected character `{c}`, {expected}");
        warn(errorkey).msg(msg).loc(self.loc).push();
    }

    fn get_key(&mut self) -> Token {
        let mut text = self.start_text();
        while let Some(c) = self.peek() {
            if is_key_char(c) {
                text.add_char(c);
                self.next_char();
            } else {
                break;
            }
        }
        text.take_to_token()
    }

    fn parse_format(&mut self) -> Option<Token> {
        (self.peek() == Some('|')).then(|| {
            self.next_char(); // eat the |
            let mut text = self.start_text();
            while let Some(c) = self.peek() {
                if c == '$' || c == ']' {
                    break;
                }
                text.add_char(c);
                self.next_char();
            }
            text.take_to_token()
        })
    }

    fn parse_code_args(&mut self) -> Vec<CodeArg> {
        self.next_char(); // eat the opening (
        let mut v = Vec::new();

        loop {
            self.skip_whitespace();
            if self.peek() == Some('\'') {
                self.next_char();
                let loc = self.loc;
                let mut parens: isize = 0;
                let mut text = self.start_text();
                while let Some(c) = self.peek() {
                    match c {
                        '\'' => break,
                        ']' => {
                            let msg = "possible unterminated argument string";
                            let info = "Using [ ] inside argument strings does not work";
                            warn(ErrorKey::Localization).msg(msg).info(info).loc(self.loc).push();
                        }
                        ')' if parens == 0 => warn(ErrorKey::Localization)
                            .msg("possible unterminated argument string")
                            .loc(self.loc)
                            .push(),
                        '(' => parens += 1,
                        ')' => parens -= 1,
                        '\u{feff}' => {
                            let msg = "found unicode BOM in middle of file";
                            warn(ErrorKey::ParseError).strong().msg(msg).loc(loc).push();
                        }
                        _ => (),
                    }
                    text.add_char(c);
                    self.next_char();
                }
                if self.peek() != Some('\'') {
                    self.value.push(LocaValue::Error);
                    return Vec::new();
                }
                v.push(CodeArg::Literal(text.take_to_token()));
                self.next_char();
            } else if self.peek() == Some(')') {
                // Empty () means no arguments
            } else {
                v.push(CodeArg::Chain(self.parse_code_inner()));
            }
            self.skip_whitespace();
            if self.peek() != Some(',') {
                break;
            }
            self.next_char(); // Eat the comma
        }
        if self.peek() == Some(')') {
            self.next_char();
        } else {
            self.unexpected_char("expected `)`", ErrorKey::Datafunctions);
        }
        v
    }

    fn parse_code_code(&mut self) -> Code {
        let mut text = self.start_text();

        if Game::is_hoi4() && self.peek() == Some('?') {
            text.add_char('?');
            self.next_char();
        }

        while let Some(c) = self.peek() {
            if is_code_char(c) {
                text.add_char(c);
                self.next_char();
            } else {
                break;
            }
        }
        let name = text.take_to_token();
        if self.peek() == Some('(') {
            Code { name, arguments: self.parse_code_args() }
        } else {
            Code { name, arguments: Vec::new() }
        }
    }

    fn parse_code_inner(&mut self) -> CodeChain {
        let mut v = Vec::new();
        loop {
            v.push(self.parse_code_code());
            // Newlines followed by whitespace are allowed in code sequences,
            // but not random whitespace.
            if matches!(self.peek(), Some('\r' | '\n')) {
                self.next_char();
                self.skip_whitespace();
            }
            if self.peek() != Some('.') {
                break;
            }
            self.next_char(); // Eat the '.'
        }
        CodeChain { codes: v.into_boxed_slice() }
    }

    fn parse_code(&mut self) {
        self.next_char(); // eat the opening [
        self.skip_whitespace();

        let chain = self.parse_code_inner();

        self.skip_whitespace();

        // The game engine doesn't mind if there are too many `)` before the `]`, so handle
        // that at "untidy" severity.
        let mut warned_extra_parens = false;
        while self.peek() == Some(')') {
            if !warned_extra_parens {
                let msg = "too many `)`";
                untidy(ErrorKey::Datafunctions).msg(msg).loc(self.loc).push();
                warned_extra_parens = true;
            }
            self.next_char();
            self.skip_whitespace();
        }

        let format = self.parse_format();

        if self.peek() == Some(']') {
            self.next_char();
            self.value.push(LocaValue::Code(chain, format));
        } else {
            self.unexpected_char("expected `]`", ErrorKey::Datafunctions);
            self.value.push(LocaValue::Error);
        }
    }

    fn handle_tooltip(&mut self, value: &str, loc: Loc) {
        if value.contains(',') {
            // If the value contains commas, then it's #tooltip:tag,key or #tooltip:tag,key,value
            // Separate out the tooltip.
            let value = Token::new(value, loc);
            let values: Vec<_> = value.split(',');
            self.value
                .push(LocaValue::ComplexTooltip(Box::new(values[0].clone()), values[1].clone()));
            return;
        }

        // Otherwise, then it's just #tooltip:tooltip
        self.value.push(LocaValue::Tooltip(Token::new(value, loc)));
    }

    fn parse_markup(&mut self) {
        let loc = self.loc;
        self.next_char(); // skip the #
        if self.peek() == Some('#') {
            // double # means a literal #
            self.next_char();
            self.value.push(LocaValue::Text(Token::from_static_str("#", loc)));
        } else if self.peek() == Some('!') {
            self.next_char();
            self.value.push(LocaValue::MarkupEnd);
        } else {
            // examples:
            // #indent_newline:2
            // #color:{1.0,1.0,1.0}
            // #font:TitleFont
            // #tooltippable;positive_value;TOOLTIP:expedition_progress_explanation_tt
            // #TOOLTIP:GAME_TRAIT,lifestyle_physician,[GetNullCharacter]
            // #tooltip:[Party.GetTooltipTag]|[InterestGroup.GetTooltipTag],INTEREST_GROUP_AFFILIATION_BREAKDOWN
            enum State {
                InKey(String),
                InValue(String, String, Loc, usize),
            }
            let mut state = State::InKey(String::new());
            while let Some(c) = self.peek() {
                if c.is_whitespace() {
                    break;
                }
                let mut consumed = false;
                match &mut state {
                    State::InKey(s) => {
                        if c == ':' {
                            if s.is_empty() {
                                self.unexpected_char("expected markup key", ErrorKey::Markup);
                            }
                            state = State::InValue(s.clone(), String::new(), self.loc, 0);
                        } else if c == ';' {
                            if s.is_empty() {
                                self.unexpected_char("expected markup key", ErrorKey::Markup);
                            }
                            // TODO: warn about markup keys that expect a value
                            state = State::InKey(String::new());
                        } else if c.is_alphanumeric() || c == '_' {
                            s.push(c);
                        } else {
                            break;
                        }
                    }
                    State::InValue(key, value, loc, bracecount) => {
                        if c == ':' {
                            value.push(c);
                            self.unexpected_char("expected `;`", ErrorKey::Markup);
                        } else if c == ';' {
                            if key.eq_ignore_ascii_case("tooltip") {
                                self.handle_tooltip(value, *loc);
                            }
                            state = State::InKey(String::new());
                        } else if c == '{' {
                            *bracecount += 1;
                        } else if c == '}' {
                            if *bracecount > 0 {
                                *bracecount -= 1;
                            } else {
                                let msg = "mismatched braces in markup";
                                warn(ErrorKey::Markup).msg(msg).loc(self.loc).push();
                                self.value.push(LocaValue::Error);
                            }
                        } else if c == '.'
                            || c == ','
                            || c.is_alphanumeric()
                            || c == '_'
                            || c == '|'
                        {
                            // . and , are freely allowed in markup values because with #tooltip
                            // the value might be a loca key.
                            // | is allowed because in some tooltips it separates components of the tooltip tag
                            value.push(c);
                        } else if c == '[' {
                            // Generating part of the markup with a code block is valid.
                            // Assume (hope) that it generates the current value and not some random chunk
                            // of markup. The next thing we see ought to be a comma or a space.
                            self.parse_code();
                            consumed = true;
                        } else {
                            break;
                        }
                    }
                }
                if !consumed {
                    self.next_char();
                }
            }
            // Clean up leftover state at end
            match state {
                State::InKey(_) => {
                    self.value.push(LocaValue::Markup);
                }
                State::InValue(key, value, loc, bracecount) => {
                    if key.eq_ignore_ascii_case("tooltip") {
                        self.handle_tooltip(&value, loc);
                    }
                    if bracecount > 0 {
                        let msg = "mismatched braces in markup";
                        warn(ErrorKey::Markup).msg(msg).loc(self.loc).push();
                        self.value.push(LocaValue::Error);
                    } else {
                        self.value.push(LocaValue::Markup);
                    }
                }
            }
            if self.peek().is_none_or(char::is_whitespace) {
                self.next_char();
            } else {
                let msg = "#markup should be followed by a space";
                warn(ErrorKey::Markup).msg(msg).loc(self.loc).push();
                self.value.push(LocaValue::Error);
            }
        }
    }

    fn parse_icon(&mut self) {
        self.next_char(); // eat the @

        let mut old_value = take(&mut self.value);

        while let Some(c) = self.peek() {
            if c == '[' {
                self.parse_code();
            } else if is_key_char(c) {
                let key = self.get_key();
                self.value.push(LocaValue::Text(key));
            } else if c == '!' {
                self.next_char();
                break;
            } else if self.value.is_empty() {
                self.unexpected_char("expected icon name", ErrorKey::Localization);
                self.value.push(LocaValue::Error);
                break;
            } else {
                self.unexpected_char("expected `!`", ErrorKey::Localization);
                self.value.push(LocaValue::Error);
                break;
            }
        }

        if matches!(self.value.last(), Some(LocaValue::Error)) {
            old_value.push(LocaValue::Error);
            self.value = take(&mut old_value);
        } else if self.value.len() == 1 {
            if let Some(LocaValue::Text(icon)) = self.value.last() {
                // The usual case: a simple @icon!
                old_value.push(LocaValue::Icon(icon.clone()));
                self.value = take(&mut old_value);
            } else {
                // This can happen if the whole icon is a @[...]!
                old_value.push(LocaValue::CalculatedIcon(take(&mut self.value)));
                self.value = take(&mut old_value);
            }
        } else {
            old_value.push(LocaValue::CalculatedIcon(take(&mut self.value)));
            self.value = take(&mut old_value);
        }
    }

    #[allow(dead_code)] // only needed for hoi4
    fn parse_flag(&mut self) {
        self.next_char(); // eat the @

        let mut text = self.start_text();
        while let Some(c) = self.peek() {
            if c.is_ascii_uppercase() {
                text.add_char(c);
                self.next_char();
            } else {
                break;
            }
        }
        let flag = text.take_to_token();
        if flag.is("") {
            self.unexpected_char("expected country tag", ErrorKey::Localization);
            self.value.push(LocaValue::Error);
        } else {
            self.value.push(LocaValue::Flag(flag));
        }
    }

    fn parse_escape(&mut self) {
        let loc = self.loc;
        self.next_char(); // Skip the \
        let s = match self.peek() {
            Some('n') => '\n'.to_string(),
            Some(c) => c.to_string(),
            None => {
                self.value.push(LocaValue::Error);
                return;
            }
        };
        self.next_char();
        self.value.push(LocaValue::Text(Token::new(&s, loc)));
    }

    fn parse_text(&mut self) {
        let mut text = self.start_text();
        while let Some(c) = self.peek() {
            match c {
                '[' | '#' | '@' | '\\' => break,
                _ => {
                    text.add_char(c);
                    self.next_char();
                }
            }
        }
        self.value.push(LocaValue::Text(text.take_to_token()));
    }

    pub fn parse_vec(mut self) -> Vec<LocaValue> {
        while let Some(c) = self.peek() {
            match c {
                '[' => self.parse_code(),
                '#' => self.parse_markup(),
                '@' if Game::is_hoi4() => self.parse_flag(),
                '@' => self.parse_icon(),
                '\\' => self.parse_escape(),
                _ => self.parse_text(),
            }
            if matches!(self.value.last(), Some(&LocaValue::Error)) {
                return vec![LocaValue::Error];
            }
        }
        self.value
    }

    pub fn parse(self) -> LocaValue {
        let mut value = self.parse_vec();
        if value.len() == 1 {
            value.remove(0)
        } else {
            LocaValue::Concat(value)
        }
    }
}

pub struct LocaReader {
    parser: LocaParser,
}

impl Iterator for LocaReader {
    type Item = LocaEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.parse_loca()
    }
}

pub fn parse_loca(entry: &FileEntry, content: String, lang: Language) -> LocaReader {
    let content = leak(content);
    let parser = LocaParser::new(entry, content, lang);
    LocaReader { parser }
}
