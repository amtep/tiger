use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::ck3::tables::misc::GOVERNMENT_RULES;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Government {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::GovernmentType, Government::add)
}

impl Government {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GovernmentType, key, block, Box::new(Self {}));
    }
}

impl DbKind for Government {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(vec) = block.get_field_list("flags") {
            for token in vec {
                db.add_flag(Item::GovernmentFlag, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // let modif = format!("{key}_levy_contribution_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_levy_contribution_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_tax_contribution_add");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_tax_contribution_mult");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_vassal_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);
        // let modif = format!("{key}_opinion_same_faith");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_adjective");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_realm");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
        if block.has_key("vassal_contract") {
            let loca = format!("{key}_vassals_label");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_validated_block("government_rules", |block, data| {
            let mut vd = Validator::new(block, data);
            for field in GOVERNMENT_RULES {
                vd.field_bool(field);
            }
        });

        // deprecated
        for field in GOVERNMENT_RULES {
            vd.field_bool(field);
        }

        vd.field_choice(
            "mechanic_type",
            &[
                "feudal",
                "mercenary",
                "holy_order",
                "clan",
                "theocracy",
                "administrative",
                "landless_adventurer",
                "herder",
                "nomad",
                "mandala",
            ],
        );
        // TODO: "All government types listed above should have one default"
        vd.field_bool("is_mechanic_type_default");

        vd.field_integer("fallback");

        vd.field_trigger_rooted("can_get_government", Tooltipped::No, Scopes::Character);
        vd.field_trigger_rooted("can_move_realm_capital", Tooltipped::Yes, Scopes::Character);

        vd.field_item("primary_holding", Item::HoldingType);
        vd.field_list_items("valid_holdings", Item::HoldingType);
        vd.field_list_items("required_county_holdings", Item::HoldingType);
        vd.field_item("generated_character_template", Item::CharacterTemplate);

        vd.field_list_items("primary_heritages", Item::CultureHeritage);
        vd.field_list_items("preferred_religions", Item::Religion);
        // TODO: test whether this was removed in 1.13
        vd.field_list_items("primary_cultures", Item::Culture);

        vd.field_bool("court_generate_spouses");
        if let Some(token) = vd.field_value("court_generate_commanders") {
            if !token.is("yes") && !token.is("no") {
                token.expect_number();
            }
        }
        vd.field_numeric("supply_limit_mult_for_others");

        vd.field_validated_block("prestige_opinion_override", |block, data| {
            let mut vd = Validator::new(block, data);
            for token in vd.values() {
                token.expect_number();
            }
        });

        vd.field_choice("royal_court", &["none", "any", "top_liege"]);
        vd.field_list_items("blocked_subject_courts", Item::GovernmentType);
        vd.field_choice("main_administrative_tier", &["county", "duchy", "kingdom"]);
        vd.field_choice("min_appointment_tier", &["county", "duchy", "kingdom"]);
        vd.field_choice("minimum_provincial_maa_tier", &["county", "duchy", "kingdom"]);
        vd.field_choice(
            "title_maa_setup",
            &[
                "main_administrative_tier_and_top_liege",
                "vassals_and_top_liege",
                "top_vassals_and_top_liege",
            ],
        );

        vd.field_integer("max_dread");

        vd.advice_field(
            "vassal_contract",
            "docs say vassal_contract but it's vassal_contract_group",
        );
        vd.field_item("vassal_contract_group", Item::SubjectContractGroup);
        vd.field_item("house_unity", Item::HouseUnity);
        vd.field_item("domicile_type", Item::DomicileType);

        vd.field_script_value_builder("opinion_of_liege", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("vassal", Scopes::Character, key);
            sc.define_name("liege", Scopes::Character, key);
            sc
        });
        vd.field_validated_key("opinion_of_liege_desc", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("vassal", Scopes::Character, key);
            sc.define_name("liege", Scopes::Character, key);
            validate_desc(bv, data, &mut sc);
        });
        vd.field_script_value_builder("opinion_of_suzerain", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("suzerain", Scopes::Character, key);
            sc.define_name("tributary", Scopes::Character, key);
            sc
        });
        vd.field_validated_key("opinion_of_suzerain_desc", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("suzerain", Scopes::Character, key);
            sc.define_name("tributary", Scopes::Character, key);
            validate_desc(bv, data, &mut sc);
        });
        vd.field_script_value_builder("opinion_of_overlord", |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("overlord", Scopes::Character, key);
            sc.define_name("subject", Scopes::Character, key);
            sc
        });
        vd.field_validated_key("opinion_of_overlord_desc", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("overlord", Scopes::Character, key);
            sc.define_name("subject", Scopes::Character, key);
            validate_desc(bv, data, &mut sc);
        });

        vd.field_validated_block("currency_levels_cap", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("piety");
            vd.field_integer("prestige");
            vd.field_integer("influence");
            vd.field_integer("merit");
        });

        vd.field_list_items("compatible_government_type_succession", Item::GovernmentType);

        vd.field_validated_block("ai", validate_ai);
        vd.multi_field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("top_liege_character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("color", validate_color);

        vd.field_script_value_no_breakdown_rooted(
            "ai_ruler_desired_kingdom_titles",
            Scopes::Character,
        );
        vd.field_script_value_no_breakdown_rooted(
            "ai_ruler_desired_empire_titles",
            Scopes::Character,
        );
        vd.field_trigger_rooted(
            "ai_can_reassign_council_positions",
            Tooltipped::No,
            Scopes::Character,
        );

        vd.advice_field("flag", "replaced with `flags` list in 1.18");
        vd.field_list("flags");

        // undocumented

        vd.field_item("tax_slot_type", Item::TaxSlotType);
    }
}

fn validate_ai(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_bool("use_lifestyle");
    vd.field_bool("arrange_marriage");
    vd.field_bool("use_goals");
    vd.field_bool("use_decisions");
    vd.field_bool("use_scripted_guis");
    vd.field_bool("use_legends");
    vd.field_bool("perform_religious_reformation");
    vd.field_bool("use_great_projects");
    // TODO: test whether this was removed in 1.13
    vd.field_bool("imprison");
    // TODO: test whether this was removed in 1.13
    vd.field_bool("start_murders");
}
