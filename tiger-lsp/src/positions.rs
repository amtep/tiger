use crop::Rope;
use lsp_types::{CompletionItem, CompletionTextEdit, InsertReplaceEdit, Position, Range, TextEdit};

pub trait ClientToServer: Sized {
    fn to_server(&mut self, utf16: bool, text: &Rope);

    fn into_server(mut self, utf16: bool, text: &Rope) -> Self {
        self.to_server(utf16, text);
        self
    }
}

pub trait ServerToClient: Sized {
    fn to_client(&mut self, utf16: bool, text: &Rope);

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

impl ClientToServer for TextEdit {
    fn to_server(&mut self, utf16: bool, text: &Rope) {
        self.range.to_server(utf16, text);
    }
}

impl ServerToClient for TextEdit {
    fn to_client(&mut self, utf16: bool, text: &Rope) {
        self.range.to_client(utf16, text);
    }
}

impl ClientToServer for InsertReplaceEdit {
    fn to_server(&mut self, utf16: bool, text: &Rope) {
        self.insert.to_server(utf16, text);
        self.replace.to_server(utf16, text);
    }
}

impl ServerToClient for InsertReplaceEdit {
    fn to_client(&mut self, utf16: bool, text: &Rope) {
        self.insert.to_client(utf16, text);
        self.replace.to_client(utf16, text);
    }
}

impl ClientToServer for CompletionTextEdit {
    fn to_server(&mut self, utf16: bool, text: &Rope) {
        match self {
            CompletionTextEdit::Edit(e) => e.to_server(utf16, text),
            CompletionTextEdit::InsertAndReplace(e) => e.to_server(utf16, text),
        }
    }
}

impl ServerToClient for CompletionTextEdit {
    fn to_client(&mut self, utf16: bool, text: &Rope) {
        match self {
            CompletionTextEdit::Edit(e) => e.to_client(utf16, text),
            CompletionTextEdit::InsertAndReplace(e) => e.to_client(utf16, text),
        }
    }
}

impl ClientToServer for CompletionItem {
    fn to_server(&mut self, utf16: bool, text: &Rope) {
        if let Some(e) = self.text_edit.as_mut() {
            e.to_server(utf16, text);
        }
        if let Some(v) = self.additional_text_edits.as_mut() {
            for e in v {
                e.to_server(utf16, text);
            }
        }
    }
}

impl ServerToClient for CompletionItem {
    fn to_client(&mut self, utf16: bool, text: &Rope) {
        if let Some(e) = self.text_edit.as_mut() {
            e.to_client(utf16, text);
        }
        if let Some(v) = self.additional_text_edits.as_mut() {
            for e in v {
                e.to_client(utf16, text);
            }
        }
    }
}
