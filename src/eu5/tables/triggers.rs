#![allow(unused_imports)] // TODO EU5: remove this when ready
use std::sync::LazyLock;

use crate::eu5::tables::misc::*;
use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use crate::trigger::Trigger;

use Trigger::*;

pub fn scope_trigger(name: &Token, _data: &Everything) -> Option<(Scopes, Trigger)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    TRIGGER_MAP.get(&*name_lc).copied()
}

static TRIGGER_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Trigger)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, trigger) in TRIGGER.iter().copied() {
        hash.insert(s, (from, trigger));
    }
    hash
});

/// See `triggers.log` from the game data dumps
/// A key ends with '(' if it is the version that takes a parenthesized argument in script.
const TRIGGER: &[(Scopes, &str, Trigger)] = &[
    // TODO: EU5 fill in table.
];

#[inline]
pub fn scope_trigger_complex(name: &str) -> Option<(Scopes, ArgumentValue, Scopes)> {
    TRIGGER_COMPLEX_MAP.get(name).copied()
}

static TRIGGER_COMPLEX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, ArgumentValue, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, trigger, outscopes) in TRIGGER_COMPLEX.iter().copied() {
            hash.insert(s, (from, trigger, outscopes));
        }
        hash
    });

/// LAST UPDATED VIC3 VERSION 1.8.4
/// See `triggers.log` from the game data dumps
/// `(inscopes, trigger name, argtype, outscopes)`
/// Currently only works with single argument triggers
// TODO Update argtype when vic3 updated to 1.5+
const TRIGGER_COMPLEX: &[(Scopes, &str, ArgumentValue, Scopes)] = {
    use crate::item::Item;
    use ArgumentValue::*;
    &[
    // TODO: EU5 fill in table.
    ]
};
