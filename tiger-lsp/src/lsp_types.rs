#![allow(non_snake_case)]

use std::fmt::{Display, Error, Formatter};

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TextDocumentItem {
    pub uri: String,
    pub languageId: String,
    pub version: i32,
    pub text: String,
}

/// Params for textDocument/didOpen notification
#[derive(Debug, Deserialize)]
pub struct DidOpenTextDocumentParams {
    pub textDocument: TextDocumentItem,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Display for Position {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}:{}", self.line, self.character)
    }
}

#[derive(Debug, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Display for Range {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}-{}", &self.start, &self.end)
    }
}

#[derive(Debug, Deserialize)]
pub struct VersionedTextDocumentIdentifier {
    pub version: i32,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct TextDocumentContentChangeEvent {
    pub range: Range,
    pub text: String,
}

#[derive(Debug, Deserialize)]
/// Params for textDocument/didChange notification
pub struct DidChangeTextDocumentParams {
    pub textDocument: VersionedTextDocumentIdentifier,
    pub contentChanges: Vec<TextDocumentContentChangeEvent>,
}
