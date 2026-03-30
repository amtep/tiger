use lsp_types::{Position, Range};

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn into_range(self, line_nr: u32) -> Range {
        Range {
            start: Position { line: line_nr, character: self.start as u32 },
            end: Position { line: line_nr, character: self.end as u32 },
        }
    }

    pub fn contains_inclusive(&self, pos: usize) -> bool {
        pos >= self.start && pos <= self.end
    }

    /// # PANICS
    /// Panics if the span is not contained within the line.
    pub fn extract<'a>(&self, line: &'a str) -> &'a str {
        &line[self.start..self.end]
    }
}
