use crate::block::{Block, BlockItem, Field, BV};
use crate::fileset::FileEntry;
use crate::token::{LocStack, Token};
use crate::trigger::Part;
use crate::validator::ValueValidator;

/// This trait lets the error reporting functions accept a variety of things as the error locator.
pub trait ErrorLoc {
    fn loc_length(&self) -> usize {
        1
    }
    fn into_loc(self) -> LocStack;
}

impl ErrorLoc for ValueValidator<'_> {
    fn loc_length(&self) -> usize {
        self.value().loc_length()
    }

    fn into_loc(self) -> LocStack {
        self.value().into_loc()
    }
}

impl ErrorLoc for &ValueValidator<'_> {
    fn loc_length(&self) -> usize {
        self.value().loc_length()
    }

    fn into_loc(self) -> LocStack {
        self.value().into_loc()
    }
}

impl ErrorLoc for &mut ValueValidator<'_> {
    fn loc_length(&self) -> usize {
        self.value().loc_length()
    }

    fn into_loc(self) -> LocStack {
        self.value().into_loc()
    }
}

impl ErrorLoc for BlockItem {
    fn loc_length(&self) -> usize {
        match self {
            BlockItem::Value(token) => token.loc_length(),
            BlockItem::Block(block) => block.loc_length(),
            BlockItem::Field(field) => field.loc_length(),
        }
    }

    fn into_loc(self) -> LocStack {
        match self {
            BlockItem::Value(token) => token.into_loc(),
            BlockItem::Block(block) => block.into_loc(),
            BlockItem::Field(field) => field.into_loc(),
        }
    }
}

impl ErrorLoc for &BlockItem {
    fn loc_length(&self) -> usize {
        match self {
            BlockItem::Value(token) => token.loc_length(),
            BlockItem::Block(block) => block.loc_length(),
            BlockItem::Field(field) => field.loc_length(),
        }
    }

    fn into_loc(self) -> LocStack {
        match self {
            BlockItem::Value(token) => token.into_loc(),
            BlockItem::Block(block) => block.into_loc(),
            BlockItem::Field(field) => field.into_loc(),
        }
    }
}

impl ErrorLoc for Field {
    fn into_loc(self) -> LocStack {
        self.into_key().into_loc()
    }
}

impl ErrorLoc for &Field {
    fn into_loc(self) -> LocStack {
        self.key().into_loc()
    }
}

impl ErrorLoc for BV {
    fn loc_length(&self) -> usize {
        match self {
            BV::Value(token) => token.loc_length(),
            BV::Block(block) => block.loc_length(),
        }
    }

    fn into_loc(self) -> LocStack {
        match self {
            BV::Value(token) => token.into_loc(),
            BV::Block(block) => block.into_loc(),
        }
    }
}

impl ErrorLoc for &BV {
    fn loc_length(&self) -> usize {
        match self {
            BV::Value(token) => token.loc_length(),
            BV::Block(block) => block.loc_length(),
        }
    }

    fn into_loc(self) -> LocStack {
        match self {
            BV::Value(t) => t.into_loc(),
            BV::Block(s) => s.into_loc(),
        }
    }
}

impl ErrorLoc for FileEntry {
    fn into_loc(self) -> LocStack {
        LocStack::from(&self)
    }
}

impl ErrorLoc for &FileEntry {
    fn into_loc(self) -> LocStack {
        LocStack::from(self)
    }
}

impl ErrorLoc for LocStack {
    fn into_loc(self) -> LocStack {
        self
    }
}

impl ErrorLoc for Token {
    fn loc_length(&self) -> usize {
        self.as_str().chars().count().max(1)
    }

    fn into_loc(self) -> LocStack {
        self.loc
    }
}

impl ErrorLoc for &Token {
    fn loc_length(&self) -> usize {
        self.as_str().chars().count().max(1)
    }

    fn into_loc(self) -> LocStack {
        self.loc
    }
}

impl ErrorLoc for Block {
    fn into_loc(self) -> LocStack {
        self.loc
    }
}

impl ErrorLoc for &Block {
    fn into_loc(self) -> LocStack {
        self.loc
    }
}

impl ErrorLoc for Part {
    fn into_loc(self) -> LocStack {
        match self {
            Part::Token(t) | Part::TokenArgument(t, _) => t.loc,
        }
    }

    fn loc_length(&self) -> usize {
        match self {
            Part::Token(t) => t.loc_length(),
            Part::TokenArgument(func, arg) => func.loc_length() + arg.loc_length() + 2,
        }
    }
}

impl ErrorLoc for &Part {
    fn into_loc(self) -> LocStack {
        match self {
            Part::Token(t) | Part::TokenArgument(t, _) => t.loc,
        }
    }

    fn loc_length(&self) -> usize {
        match self {
            Part::Token(t) => t.loc_length(),
            Part::TokenArgument(func, arg) => func.loc_length() + arg.loc_length() + 2,
        }
    }
}
