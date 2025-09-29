//! A general database of item types. Most game items go here.
//!
//! Items that need special handling are stored separately in the [`Everything`] type.

use std::any::Any;
use std::fmt::Debug;
use std::mem::take;

use as_any::AsAny;
use itertools::Itertools;
use rayon::prelude::*;
use strum::EnumCount;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::everything::Everything;
use crate::helpers::{
    dup_error, exact_dup_advice, exact_dup_error, DupReporter, TigerHashMap, TigerHashSet,
};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::token::Token;
use crate::variables::Variables;

pub type FlagValidator = fn(&Token, &Everything);

/// The main database of game items.
#[derive(Debug)]
pub struct Db {
    /// Items with full `DbEntries`, meaning a key and a block for each.
    /// The `Vec` is indexed with an `Item` discriminant.
    database: Vec<TigerHashMap<&'static str, DbEntry>>,
    /// Items generated as side effects of the full items in `database`.
    /// It allows only a limited form of validation.
    /// The `Vec` is indexed with an `Item` discriminant.
    flags: Vec<(TigerHashSet<Token>, Option<FlagValidator>)>,
    /// Items that have object identity but no usable name
    anonymous: Vec<DbEntry>,
    /// Lowercased registry of database items and flags, for case insensitive lookups
    items_lc: Vec<TigerHashMap<Lowercase<'static>, &'static str>>,
}

impl Default for Db {
    fn default() -> Self {
        Self {
            database: (0..Item::COUNT).map(|_| TigerHashMap::default()).collect(),
            flags: (0..Item::COUNT).map(|_| (TigerHashSet::default(), None)).collect(),
            anonymous: Vec::new(),
            items_lc: (0..Item::COUNT).map(|_| TigerHashMap::default()).collect(),
        }
    }
}

impl Db {
    pub fn add(&mut self, item: Item, key: Token, block: Block, kind: Box<dyn DbKind>) {
        self.add_with_reporter(item, key, block, kind, exact_dup_error);
    }

    #[allow(dead_code)]
    pub fn add_exact_dup_ok(
        &mut self,
        item: Item,
        key: Token,
        block: Block,
        kind: Box<dyn DbKind>,
    ) {
        self.add_with_reporter(item, key, block, kind, exact_dup_advice);
    }

    fn add_with_reporter(
        &mut self,
        item: Item,
        key: Token,
        block: Block,
        kind: Box<dyn DbKind>,
        exact_dup_reporter: DupReporter,
    ) {
        use std::collections::hash_map::Entry;

        match self.database[item as usize].entry(key.as_str()) {
            Entry::Occupied(mut occupied_entry) => {
                let new = DbEntry { key, block, kind };
                let existing = occupied_entry.get();

                let (keep, overwriten) =
                    // Just compare the top loc, not the whole stack
                    if new.key.loc.ptr > existing.key.loc.ptr {
                        (&new, existing)
                    } else {
                        (existing, &new)
                    };

                if overwriten.key.loc.ptr.kind >= keep.key.loc.ptr.kind {
                    if overwriten.block.equivalent(&keep.block) {
                        exact_dup_reporter(&keep.key, &overwriten.key, &item.to_string());
                    } else {
                        dup_error(&keep.key, &overwriten.key, &item.to_string());
                    }
                }
                // Update the db if the new entry is the one we're keeping
                if &raw const new == &raw const *keep {
                    self.items_lc[item as usize]
                        .insert(Lowercase::new(new.key.as_str()), new.key.as_str());
                    occupied_entry.insert(new);
                }
            }
            Entry::Vacant(vacant_entry) => {
                self.items_lc[item as usize].insert(Lowercase::new(key.as_str()), key.as_str());
                vacant_entry.insert(DbEntry { key, block, kind });
            }
        }
    }

    #[cfg(feature = "hoi4")]
    pub fn set_flag_validator(&mut self, item: Item, f: FlagValidator) {
        self.flags[item as usize].1 = Some(f);
    }

    pub fn add_flag(&mut self, item: Item, key: Token) {
        self.items_lc[item as usize].insert(Lowercase::new(key.as_str()), key.as_str());
        self.flags[item as usize].0.insert(key);
    }

    #[cfg(feature = "hoi4")]
    pub fn add_anonymous(&mut self, ident: Token, block: Block, kind: Box<dyn DbKind>) {
        self.anonymous.push(DbEntry { key: ident, block, kind });
    }

    pub fn add_subitems(&mut self) {
        for itype in 0..Item::COUNT {
            let queue = take(&mut self.database[itype]);
            for entry in queue.values().sorted_by_key(|e| e.key.loc) {
                entry.kind.add_subitems(&entry.key, &entry.block, self);
            }
            if self.database[itype].is_empty() {
                // The usual case. It should be extremely rare for `add_subitems` to add items of
                // the same item type as its parent.
                self.database[itype] = queue;
            } else {
                self.database[itype].extend(queue);
            }
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for map in &self.database {
            for entry in map.values() {
                registry.scan(&entry.block);
            }
        }
    }

    pub fn validate(&self, data: &Everything) {
        self.database.par_iter().for_each(|map| {
            map.par_iter().for_each(|(_, entry)| {
                entry.kind.validate(&entry.key, &entry.block, data);
            });
        });
        self.flags.par_iter().for_each(|(map, fv)| {
            if let Some(fv) = fv {
                map.par_iter().for_each(|flag| {
                    fv(flag, data);
                });
            }
        });
        self.anonymous.par_iter().for_each(|entry| {
            entry.kind.validate(&entry.key, &entry.block, data);
        });
    }

    pub fn exists(&self, item: Item, key: &str) -> bool {
        self.database[item as usize].contains_key(key) || self.flags[item as usize].0.contains(key)
    }

    pub fn exists_lc(&self, item: Item, key: &Lowercase) -> bool {
        self.items_lc[item as usize].contains_key(key)
    }

    #[allow(dead_code)]
    pub fn get_item<T: DbKind + Any>(&self, item: Item, key: &str) -> Option<(&Token, &Block, &T)> {
        if let Some(entry) = self.database[item as usize].get(key) {
            if let Some(kind) = (*entry.kind).as_any().downcast_ref::<T>() {
                return Some((&entry.key, &entry.block, kind));
            }
        }
        None
    }

    pub fn get_key_block(&self, item: Item, key: &str) -> Option<(&Token, &Block)> {
        self.database[item as usize].get(key).map(|entry| (&entry.key, &entry.block))
    }

    #[allow(dead_code)]
    pub fn has_property(&self, item: Item, key: &str, property: &str, data: &Everything) -> bool {
        if let Some(entry) = self.database[item as usize].get(key) {
            entry.kind.has_property(&entry.key, &entry.block, property, data)
        } else {
            false
        }
    }

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn lc_has_property(
        &self,
        item: Item,
        key: &Lowercase,
        property: &str,
        data: &Everything,
    ) -> bool {
        let real_key = self.items_lc[item as usize].get(key);
        if let Some(entry) = real_key.and_then(|key| self.database[item as usize].get(key)) {
            entry.kind.has_property(&entry.key, &entry.block, property, data)
        } else {
            false
        }
    }

    #[cfg(feature = "ck3")] // vic3 happens not to use
    pub fn set_property(&mut self, item: Item, key: &str, property: &str) {
        if let Some(entry) = self.database[item as usize].get_mut(key) {
            entry.kind.set_property(&entry.key, &entry.block, property);
        }
    }

    #[allow(dead_code)]
    pub fn validate_call(
        &self,
        item: Item,
        key: &Token,
        block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        if let Some(entry) = self.database[item as usize].get(key.as_str()) {
            entry.kind.validate_call(&entry.key, &entry.block, key, block, data, sc);
        }
    }

    #[allow(dead_code)]
    pub fn validate_use(&self, item: Item, key: &Token, block: &Block, data: &Everything) {
        if let Some(entry) = self.database[item as usize].get(key.as_str()) {
            entry.kind.validate_use(&entry.key, &entry.block, data, key, block);
        }
    }

    #[allow(dead_code)]
    pub fn validate_property_use(
        &self,
        item: Item,
        key: &Token,
        data: &Everything,
        property: &Token,
        caller: &str,
    ) {
        if let Some(entry) = self.database[item as usize].get(key.as_str()) {
            entry.kind.validate_property_use(&entry.key, &entry.block, property, caller, data);
        }
    }

    #[allow(dead_code)]
    pub fn iter_key_block(&self, itype: Item) -> impl Iterator<Item = (&Token, &Block)> {
        self.database[itype as usize].values().map(|entry| (&entry.key, &entry.block))
    }

    pub fn iter_keys(&self, itype: Item) -> impl Iterator<Item = &Token> {
        self.database[itype as usize]
            .values()
            .map(|entry| &entry.key)
            .chain(self.flags[itype as usize].0.iter())
    }
}

#[derive(Debug)]
pub struct DbEntry {
    key: Token,
    block: Block,
    kind: Box<dyn DbKind>,
}

#[allow(dead_code)]
pub trait DbKind: Debug + AsAny + Sync + Send {
    /// Add additional items that are implied by the current item, for example buildings that add
    /// `BuildingFlag` items. It's done in a separate pass so that items that were later overridden
    /// don't add their subitems.
    fn add_subitems(&self, _key: &Token, _block: &Block, _db: &mut Db) {}

    fn validate(&self, key: &Token, block: &Block, data: &Everything);
    fn has_property(
        &self,
        _key: &Token,
        _block: &Block,
        _property: &str,
        _data: &Everything,
    ) -> bool {
        false
    }
    fn validate_call(
        &self,
        _key: &Token,
        _block: &Block,
        _from: &Token,
        _from_block: &Block,
        _data: &Everything,
        _sc: &mut ScopeContext,
    ) {
    }

    fn validate_use(
        &self,
        _key: &Token,
        _block: &Block,
        _data: &Everything,
        _call_key: &Token,
        _call_block: &Block,
    ) {
    }

    fn validate_property_use(
        &self,
        _key: &Token,
        _block: &Block,
        _property: &Token,
        _caller: &str,
        _data: &Everything,
    ) {
    }

    fn set_property(&mut self, _key: &Token, _block: &Block, _property: &str) {}
}
