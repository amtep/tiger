use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct WarGoalType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::WarGoalType, WarGoalType::add)
}

impl WarGoalType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::WarGoalType, key, block, Box::new(Self {}));
    }
}

impl DbKind for WarGoalType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists_implied(Item::Localization, &format!("war_goal_{key}"), key);
        data.verify_exists_implied(Item::Localization, &format!("war_goal_{key}_desc"), key);

        vd.req_field("icon");
        vd.field_item("icon", Item::File);

        vd.req_field("kind");
        vd.field_choice(
            "kind",
            &[
                "annex_country",
                "ban_slavery",
                "colonization_rights",
                "conquer_state",
                "contain_threat",
                "enforce_treaty_article",
                "force_nationalization",
                "foreign_investment_rights",
                "humiliation",
                "increase_autonomy",
                "independence",
                "join_power_bloc",
                "leave_power_bloc",
                "liberate_country",
                "liberate_subject",
                "make_dominion",
                "make_protectorate",
                "make_tributary",
                "open_market",
                "reduce_autonomy",
                "regime_change",
                "return_state",
                "revoke_all_claims",
                "revoke_claim",
                "secession",
                "take_treaty_port",
                "transfer_subject",
                "unification",
                "unification_leadership",
                "custom",
            ],
        );

        let settings = &[
            "require_target_be_part_of_war",
            "can_add_for_other_country",
            "annexes_entire_state",
            "annexes_entire_country",
            "country_creation",
            "overlord_is_stakeholder",
            "can_target_decentralized",
            "has_other_stakeholder",
            "turns_into_subject",
            "skip_build_list",
            "targets_enemy_subject",
            "targets_enemy_claims",
            "requires_interest",
            "debug",
            "validate_subject_relation",
            "validate_formation_candidate_self",
            "validate_formation_candidate_target",
            "validate_sole_formation_candidate",
            "validate_target_not_treaty_port",
            "validate_join_power_bloc",
            "validate_colonization_rights",
            "validate_force_nationalization",
            "validate_foreign_investment_rights",
            "validate_regime_change",
            "validate_contain_threat",
            "validate_revoke_claims",
            "validate_increase_autonomy",
            "validate_take_treaty_port",
            "validate_independence",
            "validate_conflicts_war_goals_holder",
            "validate_conflicts_war_goals_all",
            "validate_conflicts_conquer_state",
            "validate_conflicts_annex_country",
            "validate_conflicts_make_subject",
            "validate_conflicts_existing_subject",
            "conflicts_with_make_subject",
            "conflicts_with_country_creation",
            "conflicts_with_annex_country",
            "conflicts_with_annex_state",
            "conflicts_with_existing_subject",
            // undocumented
            "preserve_on_switching_sides",
        ];
        vd.field_list_choice("settings", settings);

        vd.field_numeric("execution_priority");

        vd.field_choice(
            "contestion_type",
            &[
                "control_target_state",
                "control_target_country_capital",
                "control_any_target_country_state",
                "control_any_target_incorporated_state",
                "control_own_state",
                "control_own_capital",
                "control_all_own_states",
                "control_all_target_country_claims",
                "control_any_releasable_state",
            ],
        );

        vd.field_choice("target_type", &["country", "state", "treaty_article"]);

        for condition in &["possible", "valid"] {
            vd.field_trigger_builder(condition, Tooltipped::Yes, &build_sc);
        }

        for calculation in &["maneuvers", "infamy"] {
            vd.field_script_value_builder(calculation, &build_sc);
        }

        vd.field_effect_builder("on_enforced", Tooltipped::Yes, &build_sc);

        vd.multi_field_validated_block("ai", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_bool("is_significant_demand");
        });
    }
}

fn build_sc(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Country, key);
    sc.define_name("creator_country", Scopes::Country, key);
    sc.define_name("diplomatic_play", Scopes::DiplomaticPlay, key);
    sc.define_name("target_country", Scopes::Country, key);
    sc.define_name("target_state", Scopes::State, key);
    sc.define_name("stakeholder", Scopes::Country, key);
    sc.define_name("target_region", Scopes::StateRegion, key);
    sc.define_name("article_options", Scopes::TreatyArticleOptions, key);
    sc
}
