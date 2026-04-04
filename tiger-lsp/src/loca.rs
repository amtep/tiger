use crate::parse::util::Span;

#[derive(Debug, Clone)]
pub struct Node {
    pub kind: Kind,
    /// Will be empty for most kinds.
    pub content: Vec<Node>,
    /// Start and end in the relevant line.
    /// For nodes with content, the span encompasses all their content and surounding delimiters.
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The key for the whole line. There should be only one of these in the parse result.
    Key,
    /// The optional number after the key. It does nothing.
    VersionNumber,
    /// Free text in the localization value.
    /// Also used for the content of `Icon`, `Macro`, `DatatypeCall`.
    Text,
    /// The optional comment, either on its own or after a value.
    Comment,
    /// A `@icon!` reference.
    /// The corresponding [`Node`] content contains `IconText` and possibly `Macro` or `DatatypeExpr` nodes.
    Icon,
    IconText,
    /// A `$macro$` that will inline another localization value into this one.
    /// The corresponding [`Node`] content contains `MacroText` and possibly a `Format` node.
    Macro,
    MacroText,
    /// A datatype expression like `[GetPlayer.GetName]`.
    /// The corresponding [`Node`] content contains `DatatypeCall` nodes and possibly a `Format` node.
    /// Macro nodes are also possible.
    DatatypeExpr,
    /// Part of a datatype expression, like `GetPlayer` in `[GetPlayer.GetName]`.
    /// If the part has arguments, as in `GetTrait('presence')`, it includes the arguments.
    /// The corresponding [`Node`] content contains a `DatatypeId` node, and `DatatypeCall` or `DatatypeLiteral` nodes
    /// for the arguments. `Macro` nodes are also possible.
    DatatypeCall,
    DatatypeId,
    /// A string literal in a datatype expression. It includes the surrounding quote marks.
    DatatypeLiteral,
    /// The optional formatting string at the end of a datatype expression or macro,
    /// for example the `|E` in `[concept|E]` for CK3 game concepts.
    /// The span includes the leading `|`.
    Format,
    /// Any part of the line that contains characters not allowed there.
    /// For example a line that starts `loca_%key` will start with an `Error` instead of a `Key`.
    Error,
}
