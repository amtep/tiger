//! Validators for effects that are generic across multiple games.

use crate::block::{Block, Comparator, Eq::Single, BV};
use crate::context::ScopeContext;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_control};
use crate::everything::Everything;
#[cfg(feature = "jomini")]
use crate::game::Game;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{err, warn, ErrorKey, Severity};
use crate::scopes::Scopes;
#[cfg(feature = "jomini")]
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::{validate_target_ok_this, validate_trigger_key_bv};
use crate::validate::{validate_identifier, validate_optional_duration};
use crate::validator::{Validator, ValueValidator};

#[allow(dead_code)]
#[cfg(feature = "imperator")]
pub fn validate_add_to_list_imperator(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.identifier("list name");
    sc.define_or_expect_list(vd.value());
    vd.accept();
}

#[allow(dead_code)]
#[cfg(any(feature = "ck3", feature = "vic3"))]
pub fn validate_add_to_list(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(name) => {
            validate_identifier(name, "list name", Severity::Error);
            sc.define_or_expect_list(name);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_field("name");
            vd.req_field("value");
            if let Some(target) = vd.field_value("value").cloned() {
                if let Some(name) = vd.field_value("name") {
                    validate_identifier(name, "list name", Severity::Error);
                    let outscopes =
                        validate_target_ok_this(&target, data, sc, Scopes::all_but_none());
                    sc.open_scope(outscopes, target);
                    sc.define_or_expect_list(name);
                    sc.close();
                }
            }
        }
    }
}

/// A specific validator for the three `add_to_variable_list` effects (`global`, `local`, and default).
#[cfg(feature = "jomini")]
pub fn validate_add_to_variable_list(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("target");
    vd.field_identifier("name", "list name");
    vd.field_target_ok_this("target", sc, Scopes::all_but_none());
    if Game::is_ck3() || Game::is_vic3() {
        validate_optional_duration(&mut vd, sc);
    }
}

/// A specific validator for the three `change_variable` effects (`global`, `local`, and default).
#[cfg(feature = "jomini")]
pub fn validate_change_variable(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_identifier("name", "list name");
    vd.field_script_value("add", sc);
    vd.field_script_value("subtract", sc);
    vd.field_script_value("multiply", sc);
    vd.field_script_value("divide", sc);
    vd.field_script_value("modulo", sc);
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
}

/// A specific validator for the three `clamp_variable` effects (`global`, `local`, and default).
#[cfg(feature = "jomini")]
pub fn validate_clamp_variable(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_identifier("name", "list name");
    vd.field_script_value("min", sc);
    vd.field_script_value("max", sc);
}

/// A specific validator for the `random_list` effect, which has a unique syntax.
#[cfg(feature = "jomini")]
pub fn validate_random_list(
    key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    vd.field_integer("pick");
    vd.field_bool("unique"); // don't know what this does
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.unknown_block_fields(|key, block| {
        if let Some(n) = key.expect_number() {
            if n < 0.0 {
                let msg = "negative weights make the whole `random_list` fail";
                err(ErrorKey::Range).strong().msg(msg).loc(key).push();
            } else if n > 0.0 && n < 1.0 {
                let msg = "fractional weights are treated as just 0 in `random_list`";
                err(ErrorKey::Range).strong().msg(msg).loc(key).push();
            } else if n.fract() != 0.0 {
                let msg = "fractions are discarded in `random_list` weights";
                warn(ErrorKey::Range).strong().msg(msg).loc(key).push();
            }
            validate_effect_control(&caller, block, data, sc, tooltipped);
        }
    });
}

#[cfg(feature = "jomini")]
pub fn validate_remove_from_list(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.identifier("list name");
    sc.expect_list(vd.value());
    vd.accept();
}

/// A specific validator for the three `round_variable` effects (`global`, `local`, and default).
#[cfg(feature = "jomini")]
pub fn validate_round_variable(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("nearest");
    vd.field_identifier("name", "variable name");
    vd.field_script_value("nearest", sc);
}

#[cfg(feature = "jomini")]
pub fn validate_save_scope(
    _key: &Token,
    mut vd: ValueValidator,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    vd.identifier("scope name");
    sc.save_current_scope(vd.value().as_str());
    vd.accept();
}

/// A specific validator for the `save_scope_value` effect.
#[cfg(feature = "jomini")]
pub fn validate_save_scope_value(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.req_field("value");
    if let Some(name) = vd.field_identifier_or_flag("name", sc) {
        // TODO: examine `value` field to check its real scope type
        sc.define_name_token(name.as_str(), Scopes::primitive(), name);
    }
    vd.field_script_value_or_flag("value", sc);
}

/// A specific validator for the three `set_variable` effects (`global`, `local`, and default).
#[cfg(feature = "jomini")]
pub fn validate_set_variable(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            validate_identifier(token, "variable name", Severity::Error);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.req_field("name");
            vd.field_identifier("name", "variable name");
            vd.field_validated("value", |bv, data| match bv {
                BV::Value(token) => {
                    validate_target_ok_this(token, data, sc, Scopes::all_but_none());
                }
                BV::Block(_) => {
                    #[cfg(feature = "jomini")]
                    if Game::is_jomini() {
                        validate_script_value(bv, data, sc);
                    }
                    // TODO HOI4
                }
            });
            validate_optional_duration(&mut vd, sc);
        }
    }
}

/// A specific validator for the `switch` effect, which has a unique syntax.
#[cfg(feature = "jomini")]
pub fn validate_switch(
    key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    vd.set_case_sensitive(true);
    vd.req_field("trigger");
    if let Some(target) = vd.field_value("trigger") {
        // clone to avoid calling vd again while target is still borrowed
        let target = target.clone();
        let mut count = 0;
        vd.set_allow_questionmark_equals(true);
        vd.unknown_block_fields(|key, block| {
            count += 1;
            if !key.is("fallback") {
                // Pretend the switch was written as a series of trigger = key lines
                let synthetic_bv = BV::Value(key.clone());
                validate_trigger_key_bv(
                    &target,
                    Comparator::Equals(Single),
                    &synthetic_bv,
                    data,
                    sc,
                    tooltipped,
                    false,
                    Severity::Error,
                );
            }

            validate_effect(block, data, sc, tooltipped);
        });
        if count == 0 {
            let msg = "switch with no branches";
            err(ErrorKey::Logic).msg(msg).loc(key).push();
        }
    }
}

#[cfg(feature = "jomini")]
pub fn validate_trigger_event(
    _key: &Token,
    bv: &BV,
    data: &Everything,
    sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    match bv {
        BV::Value(token) => {
            data.verify_exists(Item::Event, token);
            data.event_check_scope(token, sc);
            if let Some(mut event_sc) = sc.root_for_event(token) {
                data.event_validate_call(token, &mut event_sc);
            }
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.set_case_sensitive(false);
            vd.field_event("id", sc);
            vd.field_action("on_action", sc);
            #[cfg(feature = "ck3")]
            if Game::is_ck3() {
                vd.field_target("saved_event_id", sc, Scopes::Flag);
                vd.field_date("trigger_on_next_date");
                vd.field_bool("delayed");
            }
            #[cfg(feature = "vic3")]
            if Game::is_vic3() {
                vd.field_bool("popup");
            }
            validate_optional_duration(&mut vd, sc);
        }
    }
}
