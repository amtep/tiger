//! Validator for `modifs` which is our name for the basic things that modifiers modify.
//!
//! The main entry points are the [`validate_modifs`] function and the [`ModifKinds`] type.

use std::fmt::Display;

use bitflags::Flags;

use crate::block::Block;
use crate::everything::Everything;
use crate::game::Game;
#[cfg(feature = "hoi4")]
use crate::hoi4::tables::modifs::modif_loc_hoi4;
#[cfg(any(feature = "ck3", feature = "vic3", feature = "hoi4"))]
use crate::item::Item;
use crate::report::{ErrorKey, Severity, err};
#[cfg(feature = "jomini")]
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::validator::Validator;
#[cfg(feature = "vic3")]
use crate::vic3::tables::modifs::modif_loc_vic3;

/// All the things a modif can apply to.
/// Many modifs are for multiple things, so this is a bitflags type.
///
/// This trait is used to warn when a modif is used inappropriately.
/// For Imperator it is not yet known how important this is.
pub trait ModifKinds: Display + Flags + Copy {
    fn require(self, other: Self, token: &Token) {
        if !self.intersects(other) {
            let msg = format!("`{token}` is a modifier for {other} but expected {self}");
            err(ErrorKey::Modifiers).msg(msg).loc(token).push();
        }
    }

    /// Returns Some(kinds) if the token is a valid modif or *could* be a valid modif if the appropriate item existed.
    /// Returns None otherwise.
    fn lookup_modif(name: &Token, data: &Everything, warn: Option<Severity>) -> Option<Self>;
}

pub fn validate_modifs<'a, MK: ModifKinds>(
    _block: &Block,
    data: &'a Everything,
    kinds: MK,
    mut vd: Validator<'a>,
) {
    #[cfg(feature = "hoi4")]
    vd.field_validated_block("hidden_modifier", |block, data| {
        let mut vd = Validator::new(block, data);
        // Same as below, but with no loca check
        vd.unknown_fields(|key, bv| {
            if let Some(mk) = MK::lookup_modif(key, data, Some(Severity::Error)) {
                kinds.require(mk, key);

                // TODO HOI4
                let _ = &bv;
            } else {
                let msg = format!("unknown modifier `{key}`");
                err(ErrorKey::UnknownField).msg(msg).loc(key).push();
            }
        });
    });

    #[cfg(feature = "hoi4")]
    vd.field_item("custom_modifier_tooltip", Item::Localization);

    vd.unknown_fields(|key, bv| {
        #[cfg(feature = "hoi4")]
        if Game::is_hoi4()
            && (key.is("fort") || key.is("river") || data.item_exists(Item::Terrain, key.as_str()))
        {
            if let Some(block) = bv.expect_block() {
                let mut vd = Validator::new(block, data);
                vd.field_numeric("attack");
                vd.field_numeric("movement");
                vd.field_numeric("defence");
            }
            return;
        }

        if let Some(mk) = MK::lookup_modif(key, data, Some(Severity::Error)) {
            kinds.require(mk, key);
            if Game::is_jomini() {
                #[cfg(feature = "jomini")]
                validate_non_dynamic_script_value(bv, data);
            } else {
                // TODO HOI4
                let _ = &bv;
            }
            #[cfg(feature = "ck3")]
            if Game::is_ck3()
                && !key.is("health")
                && !key.is("elderly_health")
                && !key.is("child_health")
                && !key.is("negate_health_penalty_add")
            {
                data.verify_exists(Item::ModifierFormat, key);
            }
            #[cfg(feature = "vic3")]
            if Game::is_vic3() {
                // The loca should exist.
                let (loca_key, loca_desc_key) = modif_loc_vic3(key, data);
                data.verify_exists_implied(Item::Localization, &loca_key, key);
                data.verify_exists_implied(Item::Localization, &loca_desc_key, key);
            }
            #[cfg(feature = "hoi4")]
            if Game::is_hoi4() {
                let loca_key = modif_loc_hoi4(key, data);
                data.verify_exists_implied(Item::Localization, &loca_key, key);
            }
        }
        // All modifiers are potentially valid in vic3
        else if !Game::is_vic3() {
            let msg = format!("unknown modifier `{key}`");
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    });
}

pub fn verify_modif_exists<MK: ModifKinds>(
    key: &Token,
    data: &Everything,
    kinds: MK,
    sev: Severity,
) {
    if let Some(mk) = MK::lookup_modif(key, data, Some(sev)) {
        kinds.require(mk, key);
    }
    // All modifiers are potentially valid in vic3
    else if !Game::is_vic3() {
        let msg = format!("unknown modifier `{key}`");
        err(ErrorKey::UnknownField).msg(msg).loc(key).push();
    }
}
