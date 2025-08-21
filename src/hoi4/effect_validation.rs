use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::effect::validate_effect_control;
use crate::everything::Everything;
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::report::{ErrorKey, err, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::{Validator, ValueValidator};

pub fn validate_add_ace(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("name");
    vd.field_value("name");
    vd.req_field("surname");
    vd.field_value("surname");
    vd.req_field("callsign");
    vd.field_value("callsign");
    vd.field_item("type", Item::AceModifier);
}

pub fn validate_add_advisor_role(
    _key: &Token,
    _block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    if !sc.scopes().contains(Scopes::Character) {
        vd.req_field("character");
    }
    // TODO: if scope is a country literal, check that this character belongs to it.
    vd.field_item("character", Item::Character);
    vd.field_bool("activate");

    vd.field_validated_block("advisor", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("slot", Item::AdvisorSlot);
        vd.field_numeric("cost");
        vd.field_bool("can_be_fired");
        vd.field_value("idea_token"); // TODO does this need to be registered or validated
        vd.field_list_items("traits", Item::CountryLeaderTrait);
        vd.field_trigger("allowed", Tooltipped::No, sc);
    });
}

pub fn validate_add_ai_strategy(
    _key: &Token,
    block: &Block,
    _data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    vd.req_field("type");
    vd.field_item("type", Item::AiStrategyType);
    if let Some(strategy) = block.get_field_value("type") {
        match strategy.as_str() {
            "dont_defend_ally_borders"
            | "declare_war"
            | "invade"
            | "equipment_market_trade_desire"
            | "raid_target_country" => {
                vd.req_field("id");
                vd.field_target("id", sc, Scopes::Country);
                vd.field_integer("value");
            }
            "force_concentration_front_factor"
            | "force_concentration_target_weight"
            | "front_control"
            | "front_unit_request"
            | "invasion_unit_request" => {
                vd.multi_field_target("tag", sc, Scopes::Country);
                vd.multi_field_item("state", Item::State);
                vd.multi_field_item("strategic_region", Item::StrategicRegion);
                vd.multi_field_item("area", Item::AiArea);
                let sc_country = |key: &Token| {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    sc.push_as_from(Scopes::Country, key);
                    sc
                };
                vd.field_trigger_builder("country_trigger", Tooltipped::No, sc_country);
                let sc_state = |key: &Token| {
                    let mut sc = ScopeContext::new(Scopes::State, key);
                    sc.push_as_from(Scopes::Country, key);
                    sc.push_as_from(Scopes::Country, key);
                    sc
                };
                vd.field_trigger_builder("state_trigger", Tooltipped::No, sc_state);
                vd.field_numeric("ratio");
                if strategy.is("front_control") {
                    vd.field_integer("priority");
                    vd.field_choice("ordertype", &["front", "invasion"]);
                    vd.field_choice(
                        "execution_type",
                        &["careful", "balanced", "rush", "rush_weak"],
                    );
                    vd.field_bool("execute_order");
                    vd.field_bool("manual_attack");
                } else {
                    vd.field_integer("value");
                }
            }
            "put_unit_buffers" => {
                vd.field_numeric("ratio");
                vd.field_integer("order_id");
                vd.field_list_items("states", Item::State);
                vd.multi_field_item("area", Item::AiArea);
                vd.field_bool("subtract_invasions_from_need");
                vd.field_bool("subtract_fronts_from_need");
            }
            "avoid_starting_wars"
            | "force_concentration_factor"
            | "naval_invasion_supremacy_weight"
            | "intelligence_agency_usable_factories"
            | "equipment_market_spend_factories"
            | "become_spymaster" => {
                vd.field_integer("value");
            }
            "theatre_distribution_demand_increase" => {
                vd.req_field("id");
                vd.field_item("id", Item::State);
                vd.field_integer("value");
            }
            "intelligence_agency_branch_desire_factor" => {
                vd.req_field("id");
                vd.field_item("id", Item::IntelligenceAgencyBranch);
                vd.field_integer("value");
            }
            "operative_mission" => {
                vd.field_item("mission", Item::Mission);
                vd.field_target("mission_target", sc, Scopes::Country);
                vd.multi_field_item("state", Item::State);
                vd.field_integer("priority");
                vd.field_integer("value");
            }
            "operative_operation" => {
                vd.field_item("operation", Item::Operation);
                vd.field_target("operation_target", sc, Scopes::Country);
                vd.multi_field_item("state", Item::State);
                vd.multi_field_item("region", Item::StrategicRegion);
                vd.field_integer("priority");
                vd.field_integer("value");
            }
            "build_building" => {
                vd.req_field("id");
                vd.field_item("id", Item::Building);
                // TODO: state id if state building, province id if province building
                vd.field_integer("target");
                vd.field_integer("value");
            }
            "building_target" => {
                vd.req_field("id");
                vd.field_item("id", Item::Building);
                vd.field_integer("value");
            }
            "equipment_production_factor"
            | "equipment_production_surplus_management"
            | "equipment_production_min_factories" => {
                vd.req_field("id");
                vd.field_item("id", Item::EquipmentCategory);
                vd.field_integer("value");
            }
            "equipment_production_min_factories_archetype"
            | "equipment_market_for_sale_threshold"
            | "equipment_market_for_sale_factor"
            | "equipment_market_max_for_sale"
            | "equipment_market_min_for_sale"
            | "equipment_market_buying_threshold" => {
                vd.req_field("id");
                vd.field_item("id", Item::Equipment);
                vd.field_integer("value");
            }
            "equipment_market_buy" => {
                vd.field_item("equipment_type", Item::Equipment);
                vd.field_target("seller", sc, Scopes::Country);
                vd.field_integer("value");
            }
            "research_tech" | "research_weight_factor" => {
                vd.req_field("id");
                vd.field_item("id", Item::Technology);
                vd.field_integer("value");
            }
            "unit_ratio" => {
                vd.req_field("id");
                vd.field_item("id", Item::SubUnit);
                vd.field_integer("value");
            }
            "land_xp_spend_priority" | "air_xp_spend_priority" | "navy_xp_spend_priority" => {
                vd.req_field("id");
                let mut choices = vec![
                    "division_template",
                    "unlock_doctrine",
                    "equipment_variant",
                    "upgrade_xp_cutoff",
                ];
                if strategy.is("land_xp_spend_priority") {
                    choices.push("army_spirit");
                } else if strategy.is("air_xp_spend_priority") {
                    choices.push("air_spirit");
                } else if strategy.is("navy_xp_spend_priority") {
                    choices.push("navy_spirit");
                }
                vd.field_choice("id", &choices);
                vd.field_integer("value");
            }
            "strategic_air_importance" => {
                vd.req_field("id");
                vd.field_item("id", Item::StrategicRegion);
                vd.field_integer("value");
            }
            // undocumented
            "befriend" | "alliance" | "antagonize" | "protect" | "conquer" => {
                // TODO: figure out whether it's id or target. Vanilla uses both.
                vd.field_target("id", sc, Scopes::Country);
                vd.field_target("target", sc, Scopes::Country);
                vd.field_integer("value");
            }
            _ => {
                vd.field_value("id");
                vd.field_integer("value");
            }
        }
    }
}

pub fn validate_flag_name(name: &Token) {
    let v = name.split('@');
    #[allow(clippy::comparison_chain)]
    if v.len() > 2 {
        let msg = "too many tags in flag name";
        let info = "each flag may only have one @-tag";
        err(ErrorKey::Validation).msg(msg).info(info).loc(name).push();
    } else if v.len() == 2 {
        let sfx = &v[1];
        if !(sfx.starts_with("ROOT")
            || sfx.starts_with("PREV")
            || sfx.starts_with("FROM")
            || sfx.starts_with("THIS"))
        {
            let msg = "invalid tag in flag name";
            let info = "must be @ROOT, @PREV, @FROM, or @THIS";
            err(ErrorKey::Validation).msg(msg).info(info).loc(name).push();
        }
    }
}

pub fn validate_clr_flag(
    key: &Token,
    mut vd: ValueValidator,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    if key.is("clr_unit_leader_flag") {
        let msg = "deprecated in favor of clr_character_flag";
        warn(ErrorKey::Deprecated).msg(msg).loc(key).push();
    }

    validate_flag_name(vd.value());
    vd.accept();
}

pub fn validate_modify_flag(
    key: &Token,
    _block: &Block,
    _data: &Everything,
    _sc: &mut ScopeContext,
    mut vd: Validator,
    _tooltipped: Tooltipped,
) {
    if key.is("modify_unit_leader_flag") {
        let msg = "deprecated in favor of modify_character_flag";
        warn(ErrorKey::Deprecated).msg(msg).loc(key).push();
    }

    vd.req_field("flag");
    vd.field_value("flag").map(validate_flag_name);
    vd.field_integer("value");
}

pub fn validate_set_flag(
    key: &Token,
    bv: &BV,
    data: &Everything,
    _sc: &mut ScopeContext,
    _tooltipped: Tooltipped,
) {
    if key.is("set_unit_leader_flag") {
        let msg = "deprecated in favor of set_character_flag";
        warn(ErrorKey::Deprecated).msg(msg).loc(key).push();
    }

    match bv {
        BV::Value(name) => {
            validate_flag_name(name);
        }
        BV::Block(block) => {
            let mut vd = Validator::new(block, data);
            vd.req_field("flag");
            vd.field_value("flag").map(validate_flag_name);
            vd.field_integer("days");
            vd.field_integer("value");
        }
    }
}

/// A specific validator for the `random_list` effect, which has a unique syntax.
/// This one is for the hoi4 version, which is different from the jomini games.
pub fn validate_random_list(
    key: &Token,
    _block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    mut vd: Validator,
    tooltipped: Tooltipped,
) {
    let caller = Lowercase::new(key.as_str());
    vd.field_bool("log");
    // TODO: validate variable expression if not 'const' or 'random'
    vd.field_value("seed"); // var_name/const/random
    vd.unknown_block_fields(|key, block| {
        // TODO: validate variable expression in else branch
        if let Some(n) = key.get_number() {
            if n < 0.0 {
                let msg = "negative weights make the whole `random_list` fail";
                err(ErrorKey::Range).strong().msg(msg).loc(key).push();
            }
        }
        validate_effect_control(&caller, block, data, sc, tooltipped);
    });
}
