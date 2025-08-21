use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::ck3::tables::misc::{AGENT_SLOT_CONTRIBUTION_TYPE, AI_TARGETS};
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_duration, validate_modifiers_with_base};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Scheme {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Scheme, Scheme::add)
}

impl Scheme {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Scheme, key, block, Box::new(Self {}));
    }
}

impl DbKind for Scheme {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        fn sc_secrecy(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Scheme, key);
            let target_scopes =
                Scopes::Character | Scopes::LandedTitle | Scopes::Culture | Scopes::Faith;
            sc.define_name("target", target_scopes, key);
            sc.define_name("owner", Scopes::Character, key);
            sc.define_name("exposed", Scopes::Bool, key);
            sc
        }

        let mut vd = Validator::new(block, data);
        let target_scopes =
            Scopes::Character | Scopes::LandedTitle | Scopes::Culture | Scopes::Faith;
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("scheme", Scopes::Scheme, key);
        sc.define_name("target", target_scopes, key);
        sc.define_name("owner", Scopes::Character, key);
        sc.define_name("exposed", Scopes::Bool, key);

        // let modif = format!("max_{key}_schemes_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_power_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_power_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_resistance_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_scheme_resistance_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_action");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("desc");
        vd.field_validated_sc("desc", &mut sc, validate_desc);
        vd.req_field("success_desc");
        vd.field_validated_sc("success_desc", &mut sc, validate_desc); // undocumented
        vd.field_validated_sc("discovery_desc", &mut sc, validate_desc); // undocumented

        vd.req_field("skill");
        vd.field_item("skill", Item::Skill);

        // "political" is undocumented
        vd.field_choice("category", &["personal", "contract", "hostile", "political"]);
        vd.field_choice("target_type", &["character", "title", "culture", "faith", "nothing"]);

        vd.advice_field("hostile", "deprecated; replaced with `category`");
        vd.field_bool("hostile");

        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|SCHEME_TYPE_ICON_PATH", icon, ".dds");
        vd.field_item("illustration", Item::File);

        vd.field_trigger("allow", Tooltipped::Yes, &mut sc);
        vd.field_trigger("valid", Tooltipped::No, &mut sc);

        vd.field_integer("agent_join_threshold");
        vd.field_integer("agent_leave_threshold");
        vd.field_bool("uses_resistance");

        vd.field_bool("is_basic");

        vd.field_trigger("valid_agent", Tooltipped::No, &mut sc);

        vd.field_list_choice("agent_groups_owner_perspective", AI_TARGETS);
        vd.field_list_choice("agent_groups_target_character_perspective", AI_TARGETS);

        vd.field_script_value("odds_prediction", &mut sc);

        vd.field_validated_key_block("agent_join_chance", |key, block, data| {
            let mut sc = sc.clone();
            sc.define_name("gift", Scopes::Bool, key);
            validate_modifiers_with_base(block, data, &mut sc);
        });
        vd.field_validated_block_sc("agent_success_chance", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block_sc("base_success_chance", &mut sc, validate_modifiers_with_base);
        vd.advice_field(
            "base_maximum_success_chance",
            "docs say `base_maximum_success_chance` but it's `base_maximum_success`",
        );
        vd.field_script_value("base_maximum_success", &mut sc);
        vd.advice_field("maximum_success", "Replaced with `base_maximum_success`");
        vd.field_integer_range("minimum_success", 0..=100);
        vd.field_integer_range("maximum_secrecy", 0..=100);
        vd.field_integer_range("minimum_secrecy", 0..=100);
        vd.advice_field("maximum_progress_chance", "Removed in 1.13");
        vd.advice_field("minimum_progress_chance", "Removed in 1.13");
        vd.field_script_value("base_progress_goal", &mut sc);
        vd.field_integer("maximum_breaches");

        vd.advice_field("power_per_skill_point", "Replaced with `speed_per_skill_point`");
        vd.advice_field("resistance_per_skill_point", "Removed in 1.13");
        vd.advice_field("power_per_agent_skill_point", "Removed in 1.13");
        vd.advice_field(
            "spymaster_power_per_skill_point",
            "Replaced with `spymaster_speed_per_skill_point`",
        );
        vd.advice_field("spymaster_resistance_per_skill_point", "Removed in 1.13");
        vd.advice_field("tier_resistance", "Removed in 1.13");
        vd.advice_field("uses_agents", "Removed in 1.13");

        vd.field_validated_block("pulse_actions", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("entries", Item::SchemePulseAction);
            vd.field_validated("chance_of_no_event", validate_non_dynamic_script_value);
        });

        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_bool("is_secret");
        vd.field_trigger("use_secrecy", Tooltipped::No, &mut sc);
        vd.field_script_value_no_breakdown_builder("base_secrecy", sc_secrecy);

        // on_start is undocumented
        for field in &[
            "on_start",
            "on_phase_completed",
            "on_hud_click",
            "on_monthly",
            "on_semiyearly",
            "on_invalidated",
        ] {
            vd.field_effect_rooted(field, Tooltipped::No, Scopes::Scheme);
        }
        vd.advice_field("on_ready", "Replaced with `on_phase_completed`");
        vd.advice_field("on_agent_join", "Removed in 1.13");
        vd.advice_field("on_agent_leave", "Removed in 1.13");
        vd.advice_field("on_agent_exposed", "Removed in 1.13");

        vd.field_bool("freeze_scheme_when_traveling");
        vd.field_bool("freeze_scheme_when_traveling_target");
        vd.field_bool("cancel_scheme_when_traveling");
        vd.field_bool("cancel_scheme_when_traveling_target");

        vd.field_script_value("speed_per_skill_point", &mut sc);
        vd.field_script_value("speed_per_target_skill_point", &mut sc);
        vd.field_script_value("success_chance_growth_per_skill_point", &mut sc);
        vd.field_script_value("spymaster_speed_per_skill_point", &mut sc);
        vd.field_script_value("target_spymaster_speed_per_skill_point", &mut sc);
        vd.field_integer("tier_speed");

        // undocumented

        vd.field_script_value("phases_per_agent_charge", &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct AgentType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::AgentType, AgentType::add)
}

impl AgentType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AgentType, key, block, Box::new(Self {}));
    }
}

impl DbKind for AgentType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        fn sc_builder(key: &Token) -> ScopeContext {
            let target_scopes =
                Scopes::Character | Scopes::LandedTitle | Scopes::Culture | Scopes::Faith;
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("owner", Scopes::Character, key);
            sc.define_name("scheme", Scopes::Scheme, key);
            sc.define_name("target", target_scopes, key);
            sc
        }

        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_i");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_trigger_builder("valid_agent_for_slot", Tooltipped::Yes, |key| {
            let target_scopes =
                Scopes::Character | Scopes::LandedTitle | Scopes::Culture | Scopes::Faith;
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("owner", Scopes::Character, key);
            sc.define_name("target", target_scopes, key);
            sc
        });
        vd.field_choice("contribution_type", AGENT_SLOT_CONTRIBUTION_TYPE);

        vd.field_script_value_builder("contribution", sc_builder);
    }
}

#[derive(Clone, Debug)]
pub struct SchemePulseAction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SchemePulseAction, SchemePulseAction::add)
}

impl SchemePulseAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SchemePulseAction, key, block, Box::new(Self {}));
    }
}

impl DbKind for SchemePulseAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        fn sc_builder(key: &Token) -> ScopeContext {
            let mut sc = ScopeContext::new(Scopes::Scheme, key);
            sc.define_name("scheme", Scopes::Scheme, key); // docs say "activity"
            sc.define_name("owner", Scopes::Character, key);
            sc
        }

        let mut vd = Validator::new(block, data);

        let icon = vd.field_value("icon").unwrap_or(key);
        data.verify_icon("NGameIcons|STATICMODIFIER_ICON_PATH", icon, ".dds");

        vd.field_localization("hud_text", &mut sc_builder(key));

        vd.field_trigger_builder("is_valid", Tooltipped::No, sc_builder);
        vd.field_script_value_no_breakdown_builder("weight", sc_builder);
        vd.field_effect_builder("effect", Tooltipped::No, sc_builder);
    }
}

#[derive(Clone, Debug)]
pub struct Countermeasure {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Countermeasure, Countermeasure::add)
}

impl Countermeasure {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Countermeasure, key, block, Box::new(Self {}));
    }
}

impl DbKind for Countermeasure {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(parameters) = block.get_field_block("parameters") {
            for (key, _) in parameters.iter_assignments() {
                db.add_flag(Item::CountermeasureParameter, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("scheme_countermeasure_type_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("scheme_countermeasure_type_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        data.verify_icon("NGameIcons|SCHEME_COUNTERMEASURE_TYPE_ICON_PATH", key, ".dds");

        vd.field_trigger_rooted("is_shown", Tooltipped::No, Scopes::Character);
        vd.field_trigger_rooted(
            "is_valid_showing_failures_only",
            Tooltipped::FailuresOnly,
            Scopes::Character,
        );
        vd.field_effect_rooted("on_activate", Tooltipped::Yes, Scopes::Character);

        let mut sc = ScopeContext::new(Scopes::Character, key);
        vd.field_validated_block_sc("activation_cost", &mut sc, validate_cost);

        vd.field_validated_block("owner_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_script_value_no_breakdown_rooted("ai_will_do", Scopes::Character);

        vd.field_validated_block("parameters", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|_, value| {
                if !value.is("yes") {
                    let msg = "only `yes` currently makes sense here";
                    warn(ErrorKey::Validation).msg(msg).loc(value).push();
                }
            });
        });

        // undocumented

        vd.field_item("frame", Item::File);
    }
}
