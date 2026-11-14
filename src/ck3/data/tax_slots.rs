use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::script_value::validate_script_value_no_breakdown;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TaxSlotType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::TaxSlotType, TaxSlotType::add)
}

impl TaxSlotType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TaxSlotType, key, block, Box::new(Self {}));
    }
}

impl DbKind for TaxSlotType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        vd.field_item("default_obligation", Item::TaxSlotObligation);
        vd.field_list_items("obligations", Item::TaxSlotObligation);

        vd.field_script_value_builder("tax_slot_vassal_limit", |key| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("tax_slot", Scopes::TaxSlot, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("tax_collector", Scopes::Character, key);
            sc
        });

        vd.field_trigger_builder("is_valid_tax_collector", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("liege", Scopes::Character, key);
            sc
        });

        vd.field_validated_block("aptitude_level_breakpoints", |block, data| {
            let mut vd = Validator::new(block, data);
            let mut prev = None;
            for token in vd.values() {
                if let Some(value) = token.expect_number() {
                    if prev.is_some_and(|prev| value < prev) {
                        let msg = "breakpoints must be in ascending order";
                        err(ErrorKey::Validation).msg(msg).loc(token).push();
                    }
                    prev = Some(value);
                }
            }
        });
        vd.field_script_value_rooted("tax_collector_aptitude", Scopes::Character);

        for field in &["on_tax_collector_hired", "on_tax_collector_fired"] {
            vd.field_effect_builder(field, Tooltipped::No, |key| {
                let mut sc = ScopeContext::new(Scopes::None, key);
                sc.define_name("liege", Scopes::Character, key);
                sc.define_name("tax_slot", Scopes::TaxSlot, key);
                sc.define_name("tax_collector", Scopes::Character, key);
                sc
            });
        }

        for field in &["on_vassal_assigned", "on_vassal_removed"] {
            vd.field_effect_builder(field, Tooltipped::No, |key| {
                let mut sc = ScopeContext::new(Scopes::None, key);
                sc.define_name("liege", Scopes::Character, key);
                sc.define_name("tax_slot", Scopes::TaxSlot, key);
                sc.define_name("tax_collector", Scopes::Character, key);
                sc.define_name("vassal", Scopes::Character, key);
                sc
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct TaxSlotObligation {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::TaxSlotObligation, TaxSlotObligation::add)
}

impl TaxSlotObligation {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TaxSlotObligation, key, block, Box::new(Self {}));
    }
}

impl DbKind for TaxSlotObligation {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for token in block.get_field_values("flag") {
            // TODO: not 100% sure of this.
            db.add_flag(Item::SubjectContractFlag, token.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_flavor_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        data.verify_icon("NGameIcons|TAX_SLOT_OBLIGATION_TYPE_PATH", key, ".dds");

        let mut vd = Validator::new(block, data);
        vd.field_trigger_builder("is_shown", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("tax_slot", Scopes::TaxSlot, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("tax_collector", Scopes::Character, key);
            sc
        });

        vd.field_trigger_builder("is_valid", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("tax_slot", Scopes::TaxSlot, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("tax_collector", Scopes::Character, key);
            sc
        });

        vd.field_trigger_builder("is_vassal_valid", Tooltipped::Yes, |key| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("tax_slot", Scopes::TaxSlot, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("vassal", Scopes::Character, key);
            sc.define_name("tax_collector", Scopes::Character, key);
            sc
        });

        // The docs say these have the same format as the subject contract obligation levels
        // TODO: verify the scopes, which the docs don't give
        let mut sc = ScopeContext::new(Scopes::None, key);
        sc.define_name("tax_slot", Scopes::TaxSlot, key);
        sc.define_name("liege", Scopes::Character, key);
        sc.define_name("tax_collector", Scopes::Character, key);
        sc.define_name("vassal", Scopes::Character, key);
        for currency in &["tax", "levies", "herd", "prestige", "piety", "barter_goods"] {
            vd.field_script_value(currency, &mut sc);
            vd.field_script_value(&format!("min_{currency}"), &mut sc);
            vd.field_script_value(&format!("{currency}_factor"), &mut sc);
            vd.field_item(&format!("{currency}_contribution_postfix"), Item::Localization);
        }
        vd.field_validated_sc("contribution_desc", &mut sc, validate_desc);
        vd.field_item("unclamped_contribution_label", Item::Localization);
        vd.field_item("min_contribution_label", Item::Localization);

        vd.field_validated_block("liege_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.advice_field("vassal_modifier", "was changed to subject_modifier in 1.16");
        vd.field_validated_block("subject_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.multi_field_value("flag");

        vd.field_validated_key("ai_will_do", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("tax_slot", Scopes::TaxSlot, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("tax_collector", Scopes::Character, key);
            sc.define_name("is_current_obligation", Scopes::Bool, key);
            sc.define_name("num_slots_with_obligation", Scopes::Value, key);
            sc.define_name("num_vassal_slots", Scopes::Value, key);
            validate_script_value_no_breakdown(bv, data, &mut sc);
        });

        for field in &["on_added", "on_removed"] {
            vd.field_effect_builder(field, Tooltipped::No, |key| {
                let mut sc = ScopeContext::new(Scopes::None, key);
                sc.define_name("tax_slot", Scopes::TaxSlot, key);
                sc.define_name("liege", Scopes::Character, key);
                sc.define_name("tax_collector", Scopes::Character, key);
                sc
            });
        }
    }
}
