use lsp_types::{Position, Range};

use std::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result},
};

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
            start: Position { line: line_nr, character: u32::try_from(self.start).unwrap() },
            end: Position { line: line_nr, character: u32::try_from(self.end).unwrap() },
        }
    }

    pub fn compare_inclusive(&self, pos: usize) -> Ordering {
        if pos < self.start {
            Ordering::Greater
        } else if pos > self.end {
            Ordering::Less
        } else {
            Ordering::Equal
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

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn length(&self) -> usize {
        self.end - self.start
    }
}
