//! Utility code for managing the in-game defines

use crate::block::BV;
use crate::datatype::Datatype;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

/// The expected value of a define in `common/defines`.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum DefineType {
    Integer,
    Number,
    String,
    Item(Item),
    UnknownList,
    IntegerList,
    NumberList,
    StringList,
    ItemList(Item),
    /// Color is a list of 4 numbers, presumably in RGBA format.
    Color,
}

impl From<DefineType> for Datatype {
    fn from(dt: DefineType) -> Self {
        match dt {
            DefineType::Integer | DefineType::Number => Datatype::CFixedPoint,
            DefineType::String | DefineType::Item(_) => Datatype::CString,
            DefineType::UnknownList
            | DefineType::IntegerList
            | DefineType::NumberList
            | DefineType::StringList
            | DefineType::ItemList(_) => Datatype::Unknown,
            DefineType::Color => Datatype::CVector4f,
        }
    }
}

impl DefineType {
    pub fn validate(self, bv: &BV, data: &Everything) {
        match self {
            DefineType::Integer => {
                bv.expect_value().map(Token::expect_integer);
            }
            DefineType::Number => {
                bv.expect_value().map(Token::expect_precise_number);
            }
            DefineType::String => {
                bv.expect_value();
            }
            DefineType::Item(itype) => {
                if let Some(token) = bv.expect_value() {
                    data.verify_exists(itype, token);
                }
            }
            DefineType::UnknownList | DefineType::StringList => {
                bv.expect_block();
            }
            DefineType::IntegerList => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    for value in vd.values() {
                        value.expect_integer();
                    }
                }
            }
            DefineType::NumberList => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    for value in vd.values() {
                        value.expect_precise_number();
                    }
                }
            }
            DefineType::ItemList(itype) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    for value in vd.values() {
                        data.verify_exists(itype, value);
                    }
                }
            }
            DefineType::Color => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    let mut count = 0;
                    for value in vd.values() {
                        value.expect_precise_number();
                        count += 1;
                    }
                    if count != 4 {
                        let msg = "expected exactly 4 values for color";
                        warn(ErrorKey::Colors).msg(msg).loc(block).push();
                    }
                }
            }
        }
    }
}
