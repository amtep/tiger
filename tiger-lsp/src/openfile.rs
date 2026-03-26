use log::error;
use serde_derive::Deserialize;

use crate::lsp_types::{Range, TextDocumentItem};

#[derive(Debug, Deserialize)]
pub struct OpenFile {
    pub version: i32,
    language_id: String,
    /// A vector of lines, with each line including its line ending characters
    text: Vec<String>,
}

impl From<TextDocumentItem> for OpenFile {
    fn from(tdi: TextDocumentItem) -> Self {
        Self {
            version: tdi.version,
            language_id: tdi.languageId,
            text: tdi.text.split_inclusive('\n').map(str::to_string).collect(),
        }
    }
}

impl OpenFile {
    // TODO: need a lot of tests for this.
    pub fn apply_change(&mut self, range: &Range, text: &str) {
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;
        let start_char = range.start.character as usize;
        let end_char = range.end.character as usize;

        // some checks to avoid panics
        if range.end < range.start {
            error!("  negative range {range}");
            return;
        }
        if let Some(line) = self.text.get(start_line) {
            if !line.is_char_boundary(start_char) {
                error!("  range {range} does not start at a character boundary");
            }
        } else {
            error!("  range {range} starts beyond end of document");
        }
        if let Some(line) = self.text.get(end_line) {
            if !line.is_char_boundary(end_char) {
                error!("  range {range} does not end at a character boundary");
            }
        } else {
            error!("  range {range} ends beyond end of document");
        }

        if start_line == end_line {
            self.text[start_line].replace_range(start_char..end_char, text);
        } else {
            self.text[start_line].replace_range(start_char.., text);
            self.text[end_line].replace_range(..end_char, "");
            // TODO: it would be more efficient to replace these lines with the newly inserted lines,
            // instead of removing and then inserting.
            self.text.splice(start_line + 1..end_line, []);
        }
        // possibly glue the next line to this one
        if !self.text[start_line].ends_with('\n') && start_line + 1 < self.text.len() {
            let next = self.text.remove(start_line);
            self.text[start_line].push_str(&next);
        }
        // now split any lines in the new content
        // gather the line break indices first to avoid fights with the borrow checker
        let indices: Vec<_> = self.text[start_line].rmatch_indices('\n').map(|(i, _)| i).collect();
        for i in indices {
            if i + 1 < self.text[start_line].len() {
                self.text.insert(start_line + 1, self.text[start_line][i + 1..].to_string());
                self.text[start_line].truncate(i + 1);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lsp_types::Position;

    fn open_file(text: Vec<&str>) -> OpenFile {
        OpenFile {
            version: 1,
            language_id: "pdx-locale".to_string(),
            text: text.into_iter().map(str::to_string).collect(),
        }
    }

    fn assert_result(open: &OpenFile, text: Vec<&str>) {
        let text: Vec<String> = text.into_iter().map(str::to_string).collect();
        assert_eq!(open.text, text);
    }

    // single line tests

    #[test]
    fn single_line_delete_char() {
        let mut open = open_file(vec!["single"]);

        // delete middle
        let start = Position { line: 0, character: 2 };
        let end = Position { line: 0, character: 3 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec!["sigle"]);

        // delete first
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 0, character: 1 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec!["igle"]);

        // delete last
        let start = Position { line: 0, character: 3 };
        let end = Position { line: 0, character: 4 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec!["igl"]);
    }

    #[test]
    fn single_line_delete_text() {
        let mut open = open_file(vec!["single"]);
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 0, character: 6 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec![""]);
    }

    #[test]
    fn single_line_delete_all() {
        let mut open = open_file(vec!["single"]);
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 1, character: 0 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec![]);
    }

    #[test]
    fn split_line_with_insert() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 2, character: 2 };
        let end = Position { line: 2, character: 2 };
        open.apply_change(&Range { start, end }, "ooga\nbooga");
        assert_result(&open, vec!["first", "second", "thooga", "boogaird", "fourth", "fifth"]);
    }

    #[test]
    fn join_two_lines() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 1, character: 6 };
        let end = Position { line: 2, character: 0 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec!["first", "secondthird", "fourth", "fifth"]);
    }

    #[test]
    fn delete_last_line() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 4, character: 0 };
        let end = Position { line: 5, character: 0 };
        open.apply_change(&Range { start, end }, "");
        assert_result(&open, vec!["first", "second", "third", "fourth"]);
    }

    #[test]
    fn multiline_replace_with_more_lines() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 1, character: 2 };
        let end = Position { line: 3, character: 3 };
        open.apply_change(&Range { start, end }, "foo\nbar\ngnu\nxyzzy\n");
        assert_result(&open, vec!["first", "sefoo", "bar", "gnu", "xyzzy", "rth", "fifth"]);
    }

    #[test]
    fn multiline_replace_with_fewer_lines() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 1, character: 2 };
        let end = Position { line: 3, character: 3 };
        open.apply_change(&Range { start, end }, "foo\nbar");
        assert_result(&open, vec!["first", "sefoo", "barrth", "fifth"]);
    }
}
