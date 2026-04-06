use crop::Rope;
use lsp_types::{Position, Range};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount, EnumIter, IntoStaticStr};

use crate::loca::{Kind, Node};
use crate::parse::loca_line::parse_line;
use crate::parse::util::Span;
use crate::util::ServerToClient;

#[derive(Debug, Clone, Copy, IntoStaticStr, EnumCount, EnumIter)]
#[strum(serialize_all = "camelCase")]
pub enum SemanticTokenType {
    Type, // fallback for Class
    Class,
    Variable, // fallback for Label
    Property, // fallback for Decorator
    Method,   // fallback for Promote
    Macro,
    Modifier,
    Comment,
    String,
    Number,
    // New in 3.17
    Decorator,
    // Not in the spec, but IntelliJ has it and it's perfect for Kind::Key
    Label,
    // Invented for tiger, clients can be configured for it
    Promote,
}

#[derive(Debug, Clone, Copy, IntoStaticStr, EnumCount, EnumIter)]
#[strum(serialize_all = "lowercase")]
pub enum SemanticTokenModifier {
    Definition,
    Static,
    // Invented for tiger, clients can be configured for it
    Global,
}

#[derive(Debug, Default)]
pub struct SemanticTokens {
    token_types: [Option<u32>; SemanticTokenType::COUNT],
    token_modifiers: [Option<u32>; SemanticTokenModifier::COUNT],
}

impl SemanticTokenType {
    fn fallback(self) -> Option<Self> {
        match self {
            SemanticTokenType::Class => Some(SemanticTokenType::Type),
            SemanticTokenType::Decorator => Some(SemanticTokenType::Property),
            SemanticTokenType::Label => Some(SemanticTokenType::Variable),
            SemanticTokenType::Promote => Some(SemanticTokenType::Method),
            _ => None,
        }
    }
}

impl SemanticTokenModifier {
    fn fallback(self) -> Option<Self> {
        match self {
            SemanticTokenModifier::Global => Some(SemanticTokenModifier::Static),
            _ => None,
        }
    }
}

impl SemanticTokens {
    /// Remember which labels a client supports, and return the filtered list of ones we will use.
    pub fn initialize<'a>(
        &mut self,
        client_types: &'a [&'a str],
        client_modifiers: &'a [&'a str],
    ) -> (Vec<&'a str>, Vec<&'a str>) {
        // TODO: technically we don't need to return the elements that are only used as fallbacks,
        // if we didn't need the fallback.

        let mut server_types = Vec::new();
        let mut server_modifiers = Vec::new();

        // process client list
        let mut type_index = 0;
        for token_type in SemanticTokenType::iter() {
            let token_type_name: &str = token_type.into();
            if let Some(&s) = client_types.iter().find(|&&name| name == token_type_name) {
                server_types.push(s);
                self.token_types[token_type as usize] = Some(type_index);
                type_index += 1;
            }
        }
        // fill in fallbacks
        for token_type in SemanticTokenType::iter() {
            if self.token_types[token_type as usize].is_none()
                && let Some(fallback) = token_type.fallback()
            {
                self.token_types[token_type as usize] = self.token_types[fallback as usize];
            }
        }

        // Token types and token modifiers are handled similarly but not quite the same.
        // Modifiers are returned as flag values, while types are a simple index.

        // process client list
        let mut modifier_value = 1;
        for token_modifier in SemanticTokenModifier::iter() {
            let token_modifier_name: &str = token_modifier.into();
            if let Some(&s) = client_modifiers.iter().find(|&&name| name == token_modifier_name) {
                server_modifiers.push(s);
                self.token_modifiers[token_modifier as usize] = Some(modifier_value);
                modifier_value *= 2;
            }
        }
        // fill in fallbacks
        for token_modifier in SemanticTokenModifier::iter() {
            if self.token_modifiers[token_modifier as usize].is_none()
                && let Some(fallback) = token_modifier.fallback()
            {
                self.token_modifiers[token_modifier as usize] =
                    self.token_modifiers[fallback as usize];
            }
        }

        (server_types, server_modifiers)
    }

    // TODO: do this as an iterator instead of creating many vecs?
    fn semantic_tokens_loca_helper(
        nodes: &[Node],
    ) -> Vec<(SemanticTokenType, &[SemanticTokenModifier], Span)> {
        let mut result: Vec<(SemanticTokenType, &[SemanticTokenModifier], Span)> = Vec::new();

        for node in nodes {
            match node.kind {
                Kind::Text | Kind::MacroText | Kind::Error => {}
                Kind::Key => result.push((
                    SemanticTokenType::Label,
                    &[SemanticTokenModifier::Definition],
                    node.span,
                )),
                Kind::VersionNumber => result.push((SemanticTokenType::Number, &[], node.span)),
                Kind::Comment => result.push((SemanticTokenType::Comment, &[], node.span)),
                Kind::IconText => result.push((SemanticTokenType::Decorator, &[], node.span)),
                // Color the whole icon if it is not interrupted by other things
                Kind::Icon => {
                    if node.content.len() == 1 {
                        result.push((SemanticTokenType::Decorator, &[], node.span));
                    } else {
                        result.append(&mut Self::semantic_tokens_loca_helper(&node.content));
                    }
                }
                // Color the whole macro instead of analyzing its contents
                Kind::Macro => result.push((SemanticTokenType::Macro, &[], node.span)),
                Kind::DatatypeExpr => {
                    result.append(&mut Self::semantic_tokens_loca_helper(&node.content));
                }
                // TODO: distinguish between global and not, function and promote, datatype name or not
                Kind::DatatypeId => result.push((SemanticTokenType::Method, &[], node.span)),
                Kind::DatatypeLiteral => result.push((SemanticTokenType::String, &[], node.span)),
                Kind::Format => result.push((SemanticTokenType::Modifier, &[], node.span)),
            }
        }

        result
    }

    /// Annotate the text with semantic information based on the parse results.
    /// The `lines` given are only the ones from the `range`.
    /// The resulting token encoding should use positions relative to the entire file.
    ///
    /// The parameters `utf16` and `text` are provided to support utf8-to-utf16 translation
    /// of position encodings.
    pub fn semantic_tokens_loca<I>(
        &self,
        utf16: bool,
        text: &Rope,
        range: Range,
        lines: I,
    ) -> Vec<u32>
    where
        I: Iterator<Item = String>,
    {
        let mut last_token_line: u32 = 0;
        let mut last_token_character: u32 = 0;
        let mut v = Vec::new();

        for (i, line) in lines.enumerate() {
            for (token_type, token_modifiers, span) in
                Self::semantic_tokens_loca_helper(&parse_line(&line))
            {
                let i = u32::try_from(i).expect("2^32 lines");
                let span_start = u32::try_from(span.start()).expect("line length 4GiB");

                let mut token_position =
                    Position { line: range.start.line + i, character: span_start };
                token_position.to_client(utf16, text);

                if let Some(token_index) = self.token_types[token_type as usize] {
                    v.push(token_position.line - last_token_line);
                    if token_position.line == last_token_line {
                        v.push(token_position.character - last_token_character);
                    } else {
                        v.push(token_position.character);
                    }
                    v.push(u32::try_from(span.length()).expect("4GiB token"));
                    v.push(token_index);
                    let mut modifier_value = 0;
                    for modifier in token_modifiers {
                        if let Some(token_modifier_value) = self.token_modifiers[*modifier as usize]
                        {
                            modifier_value |= token_modifier_value;
                        }
                    }
                    v.push(modifier_value);
                    last_token_line = token_position.line;
                    last_token_character = token_position.character;
                }
            }
        }
        v
    }
}
