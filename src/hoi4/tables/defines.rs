use std::sync::LazyLock;

use crate::defines::DefineType;
use crate::helpers::TigerHashMap;

/// A hashed version of [`DEFINES`], for quick lookup
pub static DEFINES_MAP: LazyLock<TigerHashMap<&'static str, DefineType>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (key, dt) in DEFINES.iter().copied() {
        hash.insert(key, dt);
    }
    hash
});

// LAST UPDATED HOI4 VERSION
// See common/defines.
const DEFINES: &[(&str, DefineType)] = &[];
