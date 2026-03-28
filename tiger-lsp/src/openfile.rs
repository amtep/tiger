use crop::Rope;
use log::error;
use lsp_types::{Range, TextDocumentItem};

#[derive(Debug)]
pub struct OpenFile {
    pub version: i32,
    language_id: String,
    /// A vector of lines, with each line including its line ending characters
    text: Rope,
}

impl From<TextDocumentItem> for OpenFile {
    fn from(tdi: TextDocumentItem) -> Self {
        Self { version: tdi.version, language_id: tdi.language_id, text: Rope::from(tdi.text) }
    }
}

impl OpenFile {
    #[cfg(test)]
    pub fn get_lines(&self) -> impl Iterator<Item = String> {
        self.text.lines().map(|l| l.chunks().collect::<String>())
    }

    pub fn new_text(&mut self, new_text: &str) {
        self.text = Rope::from(new_text);
    }

    pub fn apply_change(&mut self, range: Range, new_text: &str) {
        let start_line = range.start.line as usize;
        let end_line = range.end.line as usize;
        let start_char = range.start.character as usize;
        let end_char = range.end.character as usize;

        // some checks to avoid panics
        if range.end < range.start {
            error!("  negative range {}", display_range(range));
            return;
        }

        let line_len = self.text.line_len();

        if start_line > line_len || end_line > line_len {
            error!("  range {} line beyond end of document", display_range(range));
            return;
        }

        let start_byte = self.text.byte_of_line(start_line) + start_char;
        let end_byte = self.text.byte_of_line(end_line) + end_char;

        if start_byte > self.text.byte_len() || end_byte > self.text.byte_len() {
            error!("  range {} byte offset beyond end of document", display_range(range));
            return;
        } else if !self.text.is_char_boundary(start_byte) || !self.text.is_char_boundary(end_byte) {
            error!("  range {} not at character boundary", display_range(range));
            return;
        }

        self.text.replace(start_byte..end_byte, new_text);
    }
}

fn display_range(range: Range) -> String {
    format!(
        "{}:{}-{}:{}",
        range.start.line, range.start.character, range.end.line, range.end.character
    )
}

#[cfg(test)]
mod test {
    use line_index::{LineIndex, TextSize};
    use lsp_types::Position;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[allow(clippy::needless_pass_by_value)]
    fn open_file(text: Vec<&str>) -> OpenFile {
        let mut text = text.join("\n");
        text.push('\n');
        OpenFile { version: 1, language_id: "pdx-locale".to_string(), text: Rope::from(text) }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn assert_result(open: &OpenFile, text: Vec<&str>) {
        assert_eq!(open.get_lines().collect::<Vec<_>>(), text);
    }

    // single line tests

    #[test]
    fn single_line_delete_char() {
        let mut open = open_file(vec!["single"]);

        // delete middle
        let start = Position { line: 0, character: 2 };
        let end = Position { line: 0, character: 3 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec!["sigle"]);

        // delete first
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 0, character: 1 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec!["igle"]);

        // delete last
        let start = Position { line: 0, character: 3 };
        let end = Position { line: 0, character: 4 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec!["igl"]);
    }

    #[test]
    fn single_line_delete_text() {
        let mut open = open_file(vec!["single"]);
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 0, character: 6 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec![""]);
    }

    #[test]
    fn single_line_delete_all() {
        let mut open = open_file(vec!["single"]);
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 1, character: 0 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec![]);
    }

    #[test]
    fn single_line_replace_text() {
        let mut open = open_file(vec!["single"]);
        let start = Position { line: 0, character: 1 };
        let end = Position { line: 0, character: 6 };
        open.apply_change(Range { start, end }, "implex");
        assert_result(&open, vec!["simplex"]);
    }

    #[test]
    fn split_line_with_insert() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 2, character: 2 };
        let end = Position { line: 2, character: 2 };
        open.apply_change(Range { start, end }, "ooga\nbooga");
        assert_result(&open, vec!["first", "second", "thooga", "boogaird", "fourth", "fifth"]);
    }

    #[test]
    fn join_two_lines() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 1, character: 6 };
        let end = Position { line: 2, character: 0 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec!["first", "secondthird", "fourth", "fifth"]);
    }

    #[test]
    fn delete_last_line() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 4, character: 0 };
        let end = Position { line: 5, character: 0 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec!["first", "second", "third", "fourth"]);
    }

    #[test]
    fn multiline_replace_with_more_lines() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 1, character: 2 };
        let end = Position { line: 3, character: 3 };
        open.apply_change(Range { start, end }, "foo\nbar\ngnu\nxyzzy\n");
        assert_result(&open, vec!["first", "sefoo", "bar", "gnu", "xyzzy", "rth", "fifth"]);
    }

    #[test]
    fn multiline_replace_with_fewer_lines() {
        let mut open = open_file(vec!["first", "second", "third", "fourth", "fifth"]);
        let start = Position { line: 1, character: 2 };
        let end = Position { line: 3, character: 3 };
        open.apply_change(Range { start, end }, "foo\nbar");
        assert_result(&open, vec!["first", "sefoo", "barrth", "fifth"]);
    }

    #[test]
    fn empty() {
        let mut open = open_file(vec![""]);
        let start = Position { line: 0, character: 0 };
        let end = Position { line: 0, character: 0 };
        open.apply_change(Range { start, end }, "");
        assert_result(&open, vec![""]);
    }

    #[quickcheck]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::needless_pass_by_value)]
    fn prop_apply_change(
        mut input: String,
        new_text: String,
        start: usize,
        end: usize,
    ) -> TestResult {
        fn open_file(text: &str) -> OpenFile {
            OpenFile { version: 1, language_id: "pdx-locale".to_string(), text: Rope::from(text) }
        }

        if start > input.len()
            || end > input.len()
            || start > end
            || !input.is_char_boundary(start)
            || !input.is_char_boundary(end)
        {
            return TestResult::discard();
        }

        let line_index = LineIndex::new(&input);

        let start_lc = line_index.line_col(TextSize::new(start as u32));
        let start_pos = Position { line: start_lc.line, character: start_lc.col };

        let end_lc = line_index.line_col(TextSize::new(end as u32));
        let end_pos = Position { line: end_lc.line, character: end_lc.col };

        let mut open_file = open_file(&input);
        open_file.apply_change(Range { start: start_pos, end: end_pos }, &new_text);
        let open_file_result = open_file.get_lines().collect::<Vec<_>>();

        input.replace_range(start..end, &new_text);
        let result: Vec<String> = input.lines().map(String::from).collect();

        TestResult::from_bool(open_file_result == result)
    }
}
