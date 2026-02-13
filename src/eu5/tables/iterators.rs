#![allow(unused_imports)] // TODO EU5: remove this when ready
use std::sync::LazyLock;

use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::token::Token;

#[inline]
pub fn iterator(
    name_lc: &Lowercase,
    _name: &Token,
    _data: &Everything,
) -> Option<(Scopes, Scopes)> {
    ITERATOR_MAP.get(name_lc.as_str()).copied()
}

static ITERATOR_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Scopes)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, to) in ITERATOR.iter().copied() {
        hash.insert(s, (from, to));
    }
    hash
});

/// See `effects.log` from the game data dumps
/// These are the list iterators. Every entry represents
/// a every_, ordered_, random_, and any_ version.
const ITERATOR: &[(Scopes, &str, Scopes)] = &[
    // TODO: EU5 fill in table
];

pub fn iterator_removed(name: &str) -> Option<(&'static str, &'static str)> {
    for (removed_name, version, explanation) in ITERATOR_REMOVED.iter().copied() {
        if name == removed_name {
            return Some((version, explanation));
        }
    }
    None
}

const ITERATOR_REMOVED: &[(&str, &str, &str)] = &[];
