#![allow(non_upper_case_globals)]
#![allow(unused_imports)] // TODO EU5: remove this when ready

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::eu5::modif::ModifKinds;
use crate::everything::Everything;
use crate::helpers::{TigerHashMap, TigerHashMapExt};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, Severity, report, untidy};
use crate::scopes::Scopes;
use crate::token::Token;

pub fn lookup_modif(name: &Token, data: &Everything, warn: Option<Severity>) -> Option<ModifKinds> {
    let name_lc = Lowercase::new(name.as_str());

    // TODO: check if EU5 behaves like Vic3, with the modifier type having to exist.
    lookup_engine_modif(name, &name_lc, data, warn)
}

/// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
/// Returns None otherwise.
pub fn lookup_engine_modif(
    name: &Token,
    name_lc: &Lowercase,
    _data: &Everything,
    warn: Option<Severity>,
) -> Option<ModifKinds> {
    if let result @ Some(_) = MODIF_MAP.get(name_lc).copied() {
        return result;
    }

    if let Some(info) = MODIF_REMOVED_MAP.get(name_lc).copied() {
        if let Some(sev) = warn {
            let msg = format!("{name} has been removed");
            report(ErrorKey::Removed, sev).msg(msg).info(info).loc(name).push();
        }
        return None;
    }

    // Look up generated modifs, in a careful order because of possibly overlapping suffixes.

    // TODO: EU5 probably need a lot of code here

    None
}

fn maybe_warn(itype: Item, s: &Lowercase, name: &Token, data: &Everything, warn: Option<Severity>) {
    if let Some(sev) = warn {
        if !data.item_exists_lc(itype, s) {
            let msg = format!("could not find {itype} {s}");
            let info = format!("so the modifier {name} will have no effect");
            report(ErrorKey::MissingItem, sev).strong().msg(msg).info(info).loc(name).push();
        }
    }
}

static MODIF_MAP: LazyLock<TigerHashMap<Lowercase<'static>, ModifKinds>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (s, kind) in MODIF_TABLE.iter().copied() {
        hash.insert(Lowercase::new_unchecked(s), kind);
    }
    hash
});

/// See `modifiers.log` from the game data dumps.
/// Modifiers that follow patterns (such as containing culture names) should be handled in
/// `lookup_engine_modif`.
/// A `modif` is my name for the things that modifiers modify.
const MODIF_TABLE: &[(&str, ModifKinds)] = &[
    // TODO: EU5 fill in table
];

static MODIF_REMOVED_MAP: LazyLock<TigerHashMap<Lowercase<'static>, &'static str>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (s, info) in MODIF_REMOVED_TABLE.iter().copied() {
            hash.insert(Lowercase::new_unchecked(s), info);
        }
        hash
    });

const MODIF_REMOVED_TABLE: &[(&str, &str)] = &[];
