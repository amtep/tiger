use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::data::events::Event;
use crate::desc::validate_desc;
use crate::effect::{validate_effect, validate_effect_internal};
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{ListType, validate_ai_chance, validate_modifiers_with_base};
use crate::validator::Validator;

const EVENT_TYPES: &[&str] = &[
    "character_event",
    "country_event",
    "location_event",
    "unit_event",
    "exploration_event",
    "age_event",
];

const EVENT_OUTCOMES: &[&str] = &["positive", "neutral", "negative"];

const EVENT_CATEGORY: &[&str] =
    &["disaster_event", "situation_event", "international_organization_event"];

pub fn get_event_scope(key: &Token, block: &Block) -> (Scopes, Token) {
    if let Some(event_type) = block.get_field_value("type") {
        match event_type.as_str() {
            "character_event" => (Scopes::Character, event_type.clone()),
            "country_event" => (Scopes::Country, event_type.clone()),
            "location_event" => (Scopes::Location, event_type.clone()),
            "unit_event" => (Scopes::Unit, event_type.clone()),
            "exploration_event" => (Scopes::Exploration, event_type.clone()),
            "age_event" => (Scopes::Age, event_type.clone()),
            _ => (Scopes::Country, key.clone()),
        }
    } else {
        (Scopes::Country, key.clone())
    }
}

pub fn validate_event(event: &Event, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(&event.block, data);

    let mut tooltipped_immediate = Tooltipped::Past;
    let mut tooltipped = Tooltipped::Yes;

    // TODO: should character_event always be hidden?
    vd.field_choice("type", EVENT_TYPES);

    vd.field_bool("orphan");
    vd.field_bool("hidden");
    let hidden = event.block.field_value_is("hidden", "yes");
    if hidden {
        tooltipped_immediate = Tooltipped::No;
        tooltipped = Tooltipped::No;
    }

    vd.field_trigger("trigger", Tooltipped::No, sc);
    vd.field_trigger("major_trigger", Tooltipped::No, sc);
    vd.field_effect("on_trigger_fail", Tooltipped::No, sc);
    vd.field_validated_block_sc("weight_multiplier", sc, validate_modifiers_with_base);
    vd.field_effect("immediate", tooltipped_immediate, sc);
    vd.field_item("image", Item::File);

    vd.multi_field_validated_block("dynamic_historical_event", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("monthly_chance");
        vd.field_numeric("monthly_chance");

        vd.field_date("from");
        vd.field_date("to");

        vd.field_item("tag", Item::Country);
    });

    vd.field_bool("major");
    vd.field_bool("fire_only_once");
    vd.field_bool("interface_lock");
    vd.field_bool("hide_portraits");

    vd.field_list("illustration_tags");

    vd.field_choice("outcome", EVENT_OUTCOMES);
    vd.field_choice("category", EVENT_CATEGORY);

    vd.field_validated_sc("title", sc, validate_desc);
    vd.field_validated_sc("desc", sc, validate_desc);
    vd.field_validated_sc("historical_info", sc, validate_desc);

    if !hidden {
        vd.req_field("option");
    }
    let mut has_options = false;
    vd.multi_field_validated_block("option", |block, data| {
        has_options = true;
        validate_event_option(block, data, sc, tooltipped);
    });
    vd.field_validated_key_block("after", |key, block, data| {
        if !has_options {
            let msg = "`after` effect will not run if there are no `option` blocks";
            let info = "you can put it in `immediate` instead";
            err(ErrorKey::Logic).msg(msg).info(info).loc(key).push();
        }
        validate_effect(block, data, sc, tooltipped);
    });
}

fn validate_event_option(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    tooltipped: Tooltipped,
) {
    // TODO: warn if they use desc, first_valid, random_valid, or triggered_desc directly
    // in the name or tooltip.

    let mut vd = Validator::new(block, data);
    vd.multi_field_validated("name", |bv, data| match bv {
        BV::Value(t) => {
            data.verify_exists(Item::Localization, t);
        }
        BV::Block(b) => {
            let mut vd = Validator::new(b, data);
            vd.req_field("text");
            vd.field_trigger("trigger", Tooltipped::No, sc);
            vd.field_validated_sc("text", sc, validate_desc);
        }
    });

    vd.field_trigger("trigger", Tooltipped::No, sc);
    // undocumented
    vd.field_trigger("show_as_unavailable", Tooltipped::No, sc);

    vd.field_bool("historical_option");
    vd.field_bool("exclusive");
    vd.field_bool("evil_option");
    vd.field_bool("high_risk_option");
    vd.field_bool("high_reward_option");

    vd.field_bool("default_option");
    vd.field_bool("highlighted_option");
    vd.field_bool("fallback");
    vd.field_validated_sc("ai_will_select", sc, validate_ai_chance);
    vd.field_validated_sc("ai_chance", sc, validate_ai_chance);
    validate_effect_internal(
        &Lowercase::new_unchecked("option"),
        ListType::None,
        block,
        data,
        sc,
        &mut vd,
        tooltipped,
    );
}
