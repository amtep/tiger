use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{warn, ErrorKey};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::vic3::tables::misc::TREATY_ARTICLE_CATEGORIES;

#[derive(Clone, Debug)]
pub struct AiStrategy {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::AiStrategy, AiStrategy::add)
}

impl AiStrategy {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiStrategy, key, block, Box::new(Self {}));
    }
}

impl DbKind for AiStrategy {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        fn sc_builder_support(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("country", Scopes::Country, key);
            sc.define_name("enemy_country", Scopes::Country, key);
            sc.define_name("diplomatic_play_type", Scopes::DiplomaticPlayType, key);
            sc.define_name("is_initiator", Scopes::Bool, key);
            sc
        }

        fn sc_builder_plays(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("diplomatic_play", Scopes::DiplomaticPlay, key);
            sc
        }

        fn sc_builder_conscripts(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("military_formation", Scopes::MilitaryFormation, key);
            sc
        }

        let mut vd = Validator::new(block, data);

        if !key.is("ai_strategy_default") {
            data.verify_exists(Item::Localization, key);
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_choice("type", &["administrative", "diplomatic", "political"]);

        vd.field_item("icon", Item::File);

        // TODO verify scope type
        vd.field_trigger_rooted("will_form_power_bloc", Tooltipped::No, Scopes::Country);

        vd.field_item("desired_tax_level", Item::Level);
        vd.field_item("max_tax_level", Item::Level);
        vd.field_item("min_tax_level", Item::Level);

        // TODO verify scope type
        vd.field_script_value_rooted("undesirable_infamy_level", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("unacceptable_infamy_level", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("ideological_opinion_effect_mult", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("revolution_aversion", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("min_law_chance_to_pass", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("max_progressiveness", Scopes::Country);
        // TODO verify scope type
        vd.field_script_value_rooted("max_regressiveness", Scopes::Country);

        // TODO verify scopes
        vd.field_script_value_builder("diplomatic_play_support", sc_builder_support);
        vd.field_script_value_no_breakdown_builder("diplomatic_play_neutrality", sc_builder_plays);
        vd.field_script_value_no_breakdown_builder("diplomatic_play_boldness", sc_builder_plays);
        vd.field_validated_key("wargoal_maneuvers_fraction", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("enemy_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_script_value_rooted("change_law_chance", Scopes::Country);

        vd.field_list_items("pro_interest_groups", Item::InterestGroup);
        vd.field_list_items("anti_interest_groups", Item::InterestGroup);
        vd.field_list_items("pro_movements", Item::PoliticalMovement);
        vd.field_list_items("anti_movements", Item::PoliticalMovement);
        vd.field_validated_key_block("institution_scores", validate_institution_scores);

        vd.field_validated_key("obligation_value", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_script_value_no_breakdown_builder("interest_group_government_weight", |key| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("interest_group", Scopes::InterestGroup, key);
            sc
        });
        vd.field_validated_key("state_value", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_state", Scopes::State, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_validated_key("treaty_port_value", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_state", Scopes::State, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_validated_key("subject_value", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_validated_key("become_subject_value", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("overlord", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_validated_key("recklessness", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_validated_key("aggression", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target_country", Scopes::Country, key);
            validate_script_value(bv, data, &mut sc);
        });
        vd.field_script_value_rooted("wanted_construction_output", Scopes::Country);
        vd.replaced_field("wanted_construction_sector_levels", "wanted_construction_output");
        vd.field_script_value_rooted("wanted_army_size", Scopes::Country);
        vd.field_script_value_rooted("wanted_navy_size", Scopes::Country);

        vd.field_validated_key_block(
            "combat_unit_group_weights",
            validate_combat_unit_group_weights,
        );

        vd.field_script_value_no_breakdown_builder(
            "conscript_battalion_ratio",
            sc_builder_conscripts,
        );

        vd.field_script_value_no_breakdown_rooted("nationalization_desire", Scopes::Country);

        vd.field_validated_key_block("building_group_weights", validate_building_group_weights);

        vd.field_validated_key_block("subsidies", validate_subsidies);
        vd.field_validated_key_block("war_subsidies", validate_subsidies);
        vd.field_validated_key_block("goods_stances", validate_goods_stances);

        vd.field_script_value_rooted("colonial_interest_ratio", Scopes::Country);
        vd.field_validated_block("liberate_country_scores", validate_liberate_country_scores);
        vd.field_validated_key_block("strategic_region_scores", validate_strategic_region_scores);
        vd.field_validated_key_block("secret_goal_scores", validate_secret_goal_scores);
        vd.field_validated_key_block("secret_goal_weights", validate_secret_goal_weights);
        vd.field_validated_key_block("treaty_category_scores", validate_treaty_category_scores);
        vd.field_validated_key_block("wargoal_scores", validate_wargoal_scores);
        vd.field_validated_key_block("wargoal_weights", validate_wargoal_weights);

        vd.field_trigger_rooted("possible", Tooltipped::No, Scopes::Country); // TODO scope type
        vd.field_script_value_rooted("weight", Scopes::Country);

        vd.field_script_value_rooted("economic_vs_government_spending_balance", Scopes::Country);
    }
}

fn validate_institution_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key); // TODO verify scope type
    vd.unknown_fields(|key, bv| {
        data.verify_exists(Item::Institution, key);
        validate_script_value(bv, data, &mut sc);
    });
}

fn validate_combat_unit_group_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_fields(|key, bv| {
        data.verify_exists(Item::CombatUnitGroup, key);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("military_formation", Scopes::MilitaryFormation, key);
        validate_script_value(bv, data, &mut sc);
    });
}

fn validate_building_group_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, token| {
        data.verify_exists(Item::BuildingGroup, key);
        token.expect_number();
    });
}

// TODO what other options?
const SUBSIDIES_TYPES: &[&str] = &["should_have", "wants_to_have", "must_have", "nice_to_have"];
fn validate_subsidies(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, token| {
        data.verify_exists(Item::BuildingType, key);
        if !SUBSIDIES_TYPES.contains(&token.as_str()) {
            warn(ErrorKey::Validation).weak().msg("unknown subsidy type").loc(token).push();
        }
    });
}

fn validate_goods_stances(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::Goods, key);
        let mut vd = Validator::new(block, data);
        vd.req_field("stance");
        vd.field_choice("stance", &["wants_high_supply", "wants_export", "does_not_want"]);
        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
    });
}

fn validate_strategic_region_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    vd.unknown_fields(|key, bv| {
        data.verify_exists(Item::StrategicRegion, key);
        validate_script_value(bv, data, &mut sc);
    });
}

fn validate_secret_goal_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("target_country", Scopes::Country, key);
    vd.unknown_block_fields(|key, block| {
        data.verify_exists(Item::SecretGoal, key);
        let mut vd = Validator::new(block, data);
        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
        vd.field_script_value_no_breakdown("score", &mut sc);
    });
}

fn validate_secret_goal_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, token| {
        data.verify_exists(Item::SecretGoal, key);
        token.expect_number();
    });
}

fn validate_treaty_category_scores(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for category in TREATY_ARTICLE_CATEGORIES {
        vd.field_script_value_rooted(category, Scopes::Country);
    }
    vd.field_script_value_rooted("none", Scopes::Country);
}

fn validate_wargoal_scores(key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("target_country", Scopes::Country, key);
    sc.define_name("target_state", Scopes::State, key); // might not be set
    vd.unknown_fields(|key, bv| {
        data.verify_exists(Item::Wargoal, key);
        validate_script_value(bv, data, &mut sc);
    });
}

fn validate_wargoal_weights(_key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_value_fields(|key, token| {
        data.verify_exists(Item::Wargoal, key);
        token.expect_number();
    });
}

fn validate_liberate_country_scores(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.unknown_fields(|key, bv| {
        data.verify_exists(Item::Country, key);
        let mut sc = ScopeContext::new(Scopes::Country, key);
        sc.define_name("target_country", Scopes::Country, key);
        validate_script_value(bv, data, &mut sc);
    });
}
