//! Europa Universalis 5 specific [`Item`] functions

use crate::item::Item;

/// Returns whether an item type uses the REPLACE/INJECT/CREATE prefixes.
pub fn injectable_eu5(itype: Item) -> bool {
    // TODO: EU5 fill in injectable item types
    matches!(itype, Item::ScriptValue | Item::ScriptedEffect | Item::ScriptedTrigger)
}
