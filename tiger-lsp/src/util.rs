use crop::Rope;
use lsp_types::{Position, Range};

pub use ahash::AHashMap as HashMap;
pub use ahash::AHashSet as HashSet;

use termtree::Tree;

use crate::loca::Node;

pub trait ClientToServer: Sized {
    fn to_server(&mut self, _utf16: bool, _text: &Rope);

    fn into_server(mut self, utf16: bool, text: &Rope) -> Self {
        self.to_server(utf16, text);
        self
    }
}

pub trait ServerToClient: Sized {
    fn to_client(&mut self, _utf16: bool, _text: &Rope) {}

    fn into_client(mut self, utf16: bool, text: &Rope) -> Self {
        self.to_client(utf16, text);
        self
    }
}

impl ClientToServer for Position {
    fn to_server(&mut self, utf16: bool, text: &Rope) {
        if !utf16 || self.line as usize >= text.line_len() {
            return;
        }
        let slice = text.line_slice(self.line as usize..=self.line as usize);
        if self.character as usize > slice.utf16_len() {
            return;
        }
        self.character = slice.byte_of_utf16_code_unit(self.character as usize).try_into().unwrap();
    }
}

impl ServerToClient for Position {
    fn to_client(&mut self, utf16: bool, text: &Rope) {
        if !utf16 || self.line as usize >= text.line_len() {
            return;
        }
        let slice = text.line_slice(self.line as usize..=self.line as usize);
        if self.character as usize > slice.byte_len() {
            return;
        }
        self.character = slice.utf16_code_unit_of_byte(self.character as usize).try_into().unwrap();
    }
}

impl ClientToServer for Range {
    fn to_server(&mut self, utf16: bool, text: &Rope) {
        self.start.to_server(utf16, text);
        self.end.to_server(utf16, text);
    }
}

impl ServerToClient for Range {
    fn to_client(&mut self, utf16: bool, text: &Rope) {
        self.start.to_client(utf16, text);
        self.end.to_client(utf16, text);
    }
}

pub fn tree(node: &Node, line: &str) -> Tree<String> {
    Tree::new(format!("{}: {}  ({})", node.span.extract(line), node.kind, node.span))
        .with_leaves(node.content.iter().map(|n| tree(n, line)))
}
