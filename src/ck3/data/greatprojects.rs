use crate::block::{BV, Block};
use crate::ck3::tables::misc::PROVINCE_FILTERS;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GreatProjectType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::GreatProjectType, GreatProjectType::add)
}

impl GreatProjectType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GreatProjectType, key, block, Box::new(Self {}));
    }
}

impl DbKind for GreatProjectType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("project_contributions") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::ProjectContribution, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca = format!("great_project_type_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("great_project_type_tooltip_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("great_project_name_{key}");
        data.localization.suggest(&loca, key);
        let loca = format!("great_project_name_possessive_{key}");
        data.localization.suggest(&loca, key);

        let mut vd = Validator::new(block, data);

        if !block.has_key("icon") {
            data.verify_icon("NGameIcons|GREAT_PROJECT_TYPE_ICON_PATH", key, ".dds");
        }

        vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(value) => {
                data.verify_exists(Item::File, value);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_trigger_builder("trigger", Tooltipped::No, |key| {
                    let mut sc = ScopeContext::new(Scopes::GreatProject, key);
                    sc.define_name("province", Scopes::Province, key);
                    sc.define_name("great_project", Scopes::GreatProject, key);
                    sc.define_name("owner", Scopes::Character, key);
                    sc.define_name("founder", Scopes::Character, key);
                    sc
                });
                vd.field_item("reference", Item::File);
            }
        });

        vd.multi_field_validated_block("illustration", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger_builder("trigger", Tooltipped::No, |key| {
                let mut sc = ScopeContext::new(Scopes::GreatProject, key);
                sc.define_name("province", Scopes::Province, key);
                sc.define_name("great_project", Scopes::GreatProject, key);
                sc.define_name("owner", Scopes::Character, key);
                sc.define_name("founder", Scopes::Character, key);
                sc
            });
            vd.field_item("reference", Item::File);
        });

        // docs say it's like the previous 2 fields, but it seems to be a normal triggered desc.
        vd.field_validated_key("name", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_desc(bv, data, &mut sc);
        });

        vd.field_trigger_builder("is_shown", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("province", Scopes::Province, key);
            sc.define_name("founder", Scopes::Character, key);
            sc
        });

        vd.field_choice("province_filter", PROVINCE_FILTERS);
        if let Some(filter) = block.get_field_value("province_filter") {
            if filter.is("landed_title") {
                vd.field_item("province_filter_target", Item::Title);
            } else if filter.is("geographical_region") {
                vd.field_item("province_filter_target", Item::Region);
            } else {
                vd.ban_field(
                    "province_filter_target",
                    || "`landed_title` or `geographical_region` filters",
                );
            }
        }

        vd.field_trigger_builder("can_start_planning", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("founder", Scopes::Character, key);
            sc
        });
        vd.field_trigger_rooted("can_cancel", Tooltipped::Yes, Scopes::Character);
        vd.field_trigger_builder("is_location_valid", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("province", Scopes::Province, key);
            sc.define_name("founder", Scopes::Character, key);
            sc
        });

        vd.field_choice(
            "owner",
            &[
                "province_owner_top_liege",
                "province_owner",
                "founder_primary_title_owner",
                "founder_top_liege_title_owner",
            ],
        );
        vd.field_trigger_builder("is_valid", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::GreatProject, key);
            sc.define_name("province", Scopes::Province, key);
            sc.define_name("great_project", Scopes::GreatProject, key);
            sc.define_name("owner", Scopes::Character, key);
            sc.define_name("founder", Scopes::Character, key);
            sc
        });
        vd.field_validated_key_block("cost", |key, block, data| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            validate_cost(block, data, &mut sc);
        });

        vd.field_script_value_builder("construction_time", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("great_project", Scopes::GreatProject, key);
            sc
        });
        vd.field_numeric("contribution_threshold");
        vd.advice_field(
            "investor_cooldown",
            "docs say `investor_cooldown` but it's `contributor_cooldown`",
        );
        vd.field_script_value_builder("contributor_cooldown", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("great_project", Scopes::GreatProject, key);
            sc
        });

        for field in
            &["on_complete", "on_cancel", "on_plan_build", "on_start_build", "on_invalidated"]
        {
            vd.field_effect_builder(field, Tooltipped::Yes, |key| {
                let mut sc = ScopeContext::new(Scopes::GreatProject, key);
                sc.define_name("province", Scopes::Province, key);
                sc.define_name("great_project", Scopes::GreatProject, key);
                sc.define_name("owner", Scopes::Character, key);
                sc.define_name("founder", Scopes::Character, key);
                sc
            });
        }

        // TODO: the interaction must use the request_great_project_contribution special interaction type
        vd.field_item("invite_interaction", Item::CharacterInteraction);

        vd.field_validated_block("allowed_contributor_filter", validate_contributor_filter);

        vd.field_validated_block("project_contributions", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|k, block| {
                validate_contribution(k, block, data, key);
            });
        });

        vd.field_script_value_no_breakdown_rooted("ai_will_do", Scopes::Character);
        vd.field_choice("ai_province_filter", PROVINCE_FILTERS);
        vd.field_integer("ai_check_interval");
        vd.field_validated_key_block("ai_check_interval_by_tier", |key, b, data| {
            let mut vd = Validator::new(b, data);
            for tier in &["barony", "county", "duchy", "kingdom", "empire", "hegemony"] {
                vd.req_field(tier);
                vd.field_integer(tier);
            }
            if block.has_key("ai_check_interval") {
                let msg = "must not have both `ai_check_interval` and `ai_check_interval_by_tier`";
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        });

        vd.field_validated_block("ai_target_quick_trigger", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("adult");
            vd.field_choice(
                "rank",
                &["barony", "county", "duchy", "kingdom", "empire", "hegemony"],
            );
            vd.field_list_items("government_type", Item::GovernmentType);
        });

        vd.field_bool("show_in_list");
        vd.field_bool("is_important");
        vd.field_choice(
            "target_title_tier",
            &["barony", "county", "duchy", "kingdom", "empire", "hegemony"],
        );

        // TODO: figure out valid values for this choice type
        vd.field_value("group");

        vd.field_item("completion_sound_effect", Item::Sound);
    }
}

fn validate_contributor_filter(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("vassals");
    vd.field_bool("tributaries");
    vd.field_bool("liege");
    vd.field_bool("top_liege");
    vd.field_bool("owner");
    vd.field_bool("allies");
}

fn validate_contribution(key: &Token, block: &Block, data: &Everything, gp_key: &Token) {
    let loca = format!("great_project_type_{gp_key}_contribution_{key}");
    data.verify_exists_implied(Item::Localization, &loca, key);
    let loca = format!("great_project_type_{gp_key}_contribution_{key}_desc");
    data.verify_exists_implied(Item::Localization, &loca, key);
    let loca = format!("great_project_type_{gp_key}_contribution_name_{key}");
    data.localization.suggest(&loca, key);

    let sc_builder = |key: &Token| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("province", Scopes::Province, key);
        sc.define_name("great_project", Scopes::GreatProject, key);
        sc.define_name("founder", Scopes::Character, key);
        sc.define_name("owner", Scopes::Character, key);
        sc
    };

    let mut vd = Validator::new(block, data);
    vd.field_trigger_builder("is_shown", Tooltipped::No, sc_builder);
    vd.field_bool("show_in_planning_phase");
    // TODO: cryptic doc "Note that we print this in an unevaluated format without scope, so
    // trigger_ifs are not supported without custom descs"
    vd.field_trigger_builder("contributor_is_valid", Tooltipped::Yes, sc_builder);
    vd.field_validated_block("allowed_contributor_filter", validate_contributor_filter);
    vd.field_trigger_builder("context_allows_contributions", Tooltipped::Yes, sc_builder);
    vd.field_validated_key_block("cost", |key, block, data| {
        let mut sc = sc_builder(key);
        validate_cost(block, data, &mut sc);
    });
    vd.field_script_value_builder("contributor_cooldown", sc_builder);
    vd.field_bool("is_required");
    vd.field_effect_builder("on_contribution_funded", Tooltipped::Yes, sc_builder);
    vd.field_effect_builder("on_complete", Tooltipped::Yes, sc_builder);
    vd.field_script_value_builder("ai_will_do", sc_builder);
    vd.field_integer("ai_check_interval");
    vd.field_validated_key_block("ai_check_interval_by_tier", |key, b, data| {
        let mut vd = Validator::new(b, data);
        for tier in &["barony", "county", "duchy", "kingdom", "empire", "hegemony"] {
            vd.req_field(tier);
            vd.field_integer(tier);
        }
        if block.has_key("ai_check_interval") {
            let msg = "must not have both `ai_check_interval` and `ai_check_interval_by_tier`";
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
        }
    });
}
