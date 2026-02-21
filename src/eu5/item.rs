//! Europa Universalis 5 specific [`Item`] functions

use crate::item::Item;

impl Item {
    /// Returns whether an item type uses the REPLACE/INJECT/CREATE prefixes.
    pub fn injectable(self) -> bool {
        // TODO: EU5 fill in injectable item types
        matches!(self, Item::ScriptValue | Item::ScriptedEffect | Item::ScriptedTrigger)
    }
}
