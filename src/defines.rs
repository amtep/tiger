//! Utility code for managing the in-game defines

use crate::block::BV;
use crate::datatype::Datatype;
use crate::everything::Everything;
use crate::item::Item;
use crate::report::{err, warn, ErrorKey};
use crate::token::Token;
use crate::validator::Validator;

/// The expected value of a define in `common/defines`.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum DefineType {
    Boolean,
    Integer,
    Number,
    Date,
    String,
    Item(Item),
    SingleQuotedItem(Item),
    Choice(&'static [&'static str]),
    UnknownList,
    IntegerList,
    NumberList,
    StringList,
    ItemList(Item),
    ItemOrEmptyList(Item),
    /// Color is a list of 4 numbers, presumably in RGBA format.
    Color,
    /// Color3 is a list of 3 numbers, presumably in RGB format.
    Color3,
}

impl From<DefineType> for Datatype {
    fn from(dt: DefineType) -> Self {
        match dt {
            DefineType::Boolean => Datatype::bool,
            DefineType::Integer | DefineType::Number => Datatype::CFixedPoint,
            DefineType::Date => Datatype::Date,
            DefineType::String
            | DefineType::Item(_)
            | DefineType::SingleQuotedItem(_)
            | DefineType::Choice(_) => Datatype::CString,
            DefineType::UnknownList
            | DefineType::IntegerList
            | DefineType::NumberList
            | DefineType::StringList
            | DefineType::ItemList(_)
            | DefineType::ItemOrEmptyList(_) => Datatype::Unknown,
            DefineType::Color3 => Datatype::CVector3f,
            DefineType::Color => Datatype::CVector4f,
        }
    }
}

impl DefineType {
    pub fn validate(self, bv: &BV, data: &Everything) {
        match self {
            DefineType::Boolean => {
                if let Some(token) = bv.expect_value() {
                    if !token.is("yes") && !token.is("no") {
                        let msg = "expected `yes` or `no`";
                        err(ErrorKey::Validation).msg(msg).loc(token).push();
                    }
                }
            }
            DefineType::Integer => {
                bv.expect_value().map(Token::expect_integer);
            }
            DefineType::Number => {
                bv.expect_value().map(Token::expect_precise_number);
            }
            DefineType::Date => {
                bv.expect_value().map(Token::expect_date);
            }
            DefineType::String => {
                bv.expect_value();
            }
            DefineType::Item(itype) => {
                if let Some(token) = bv.expect_value() {
                    if !(itype == Item::Sound && token.as_str().is_empty()) {
                        data.verify_exists(itype, token);
                    }
                }
            }
            DefineType::SingleQuotedItem(itype) => {
                if let Some(token) = bv.expect_value() {
                    if let Some(sfx) = token.strip_prefix("'") {
                        if let Some(bare) = sfx.strip_suffix("'") {
                            data.verify_exists(itype, &bare);
                        }
                    }
                }
            }
            DefineType::Choice(choices) => {
                if let Some(token) = bv.expect_value() {
                    if !choices.contains(&token.as_str()) {
                        let msg = format!("expected one of {}", choices.join(", "));
                        err(ErrorKey::Choice).msg(msg).loc(token).push();
                    }
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
            DefineType::ItemOrEmptyList(itype) => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    for value in vd.values() {
                        if !value.as_str().is_empty() {
                            data.verify_exists(itype, value);
                        }
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
            DefineType::Color3 => {
                if let Some(block) = bv.expect_block() {
                    let mut vd = Validator::new(block, data);
                    let mut count = 0;
                    for value in vd.values() {
                        value.expect_precise_number();
                        count += 1;
                    }
                    if count != 3 {
                        let msg = "expected exactly 3 values for this color";
                        warn(ErrorKey::Colors).msg(msg).loc(block).push();
                    }
                }
            }
        }
    }
}
