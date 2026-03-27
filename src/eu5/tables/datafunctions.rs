#![allow(unused_imports)] // TODO EU5: remove this when ready
use std::sync::LazyLock;

use tiger_tables::datatype::*;

use crate::datatype::CaseInsensitiveStr;
use crate::helpers::{BiTigerHashMap, TigerHashMap, TigerHashSet};
use crate::scopes::Scopes;

use Datatype::*;
use Eu5Datatype::*;

pub static LOWERCASE_DATATYPE_SET: LazyLock<TigerHashSet<CaseInsensitiveStr>> =
    LazyLock::new(|| {
        let mut set = TigerHashSet::default();

        for (name, _, _) in GLOBAL_PROMOTES_EU5.iter().copied() {
            set.insert(CaseInsensitiveStr(name));
        }

        for (name, _, _) in GLOBAL_FUNCTIONS_EU5.iter().copied() {
            set.insert(CaseInsensitiveStr(name));
        }

        for (name, _, _, _) in PROMOTES_EU5.iter().copied() {
            set.insert(CaseInsensitiveStr(name));
        }

        for (name, _, _, _) in FUNCTIONS_EU5.iter().copied() {
            set.insert(CaseInsensitiveStr(name));
        }
        set
    });

pub static DATATYPE_AND_SCOPE_MAP: LazyLock<BiTigerHashMap<Datatype, Scopes>> =
    LazyLock::new(|| {
        let mut map = BiTigerHashMap::default();
        for (datatype, scope) in DATATYPE_AND_SCOPE.iter().copied() {
            map.insert(datatype, scope);
        }
        map
    });

pub static GLOBAL_PROMOTES_MAP: LazyLock<TigerHashMap<&'static str, (Args, Datatype)>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::default();
        for (name, args, datatype) in GLOBAL_PROMOTES_EU5.iter().copied() {
            map.insert(name, (args, datatype));
        }
        map
    });

pub static GLOBAL_FUNCTIONS_MAP: LazyLock<TigerHashMap<&'static str, (Args, Datatype)>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::default();
        for (name, args, datatype) in GLOBAL_FUNCTIONS_EU5.iter().copied() {
            map.insert(name, (args, datatype));
        }
        map
    });

#[allow(clippy::type_complexity)]
pub static PROMOTES_MAP: LazyLock<TigerHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in PROMOTES_EU5.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });

#[allow(clippy::type_complexity)]
pub static FUNCTIONS_MAP: LazyLock<TigerHashMap<&'static str, Vec<(Datatype, Args, Datatype)>>> =
    LazyLock::new(|| {
        let mut map = TigerHashMap::<&'static str, Vec<(Datatype, Args, Datatype)>>::default();
        for (name, from, args, to) in FUNCTIONS_EU5.iter().copied() {
            map.entry(name).or_default().push((from, args, to));
        }
        map
    });

// TODO: find the right datatypes for the commented out ones
const DATATYPE_AND_SCOPE: &[(Datatype, Scopes)] = &[
    // TODO EU5: fill in good guesses
];
