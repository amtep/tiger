use std::sync::LazyLock;

use crate::helpers::TigerHashSet;

pub(crate) static BUILTIN_MACROS_EU5: LazyLock<TigerHashSet<&'static str>> =
    LazyLock::new(|| BUILTIN_MACROS.iter().copied().collect());

// The table entries were collected by analyzing tiger's own output.
const BUILTIN_MACROS: &[&str] = &[
    // TODO: EU5 fill in table
];
