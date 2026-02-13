#![allow(unused_imports)] // TODO EU5: remove this when ready
use std::sync::LazyLock;

use crate::defines::DefineType;
use crate::helpers::TigerHashMap;
use crate::item::Item;

/// A hashed version of [`DEFINES`], for quick lookup
pub static DEFINES_MAP: LazyLock<TigerHashMap<&'static str, DefineType>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (key, dt) in DEFINES.iter().copied() {
        hash.insert(key, dt);
    }
    hash
});

// See common/defines. Remember the ones in ../jomini/
const DEFINES: &[(&str, DefineType)] = &[
    // TODO: EU5 fill in table
];
