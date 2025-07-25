use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
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
        vd.field_item("government", Item::GovernmentType);
        // Undocumented
        vd.field_item("default_obligation", Item::TaxSlotObligation);
        // Documented erroneously as `vassal_contracts` in _tax_slot_type.info
        vd.field_list_items("obligations", Item::TaxSlotObligation);

        vd.field_script_value_builder("tax_slot_vassal_limit", |key| {
            let mut sc = ScopeContext::new(Scopes::TaxSlot, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("tax_collector", Scopes::Character, key);
            sc
        });

        vd.field_trigger_builder("is_valid_tax_collector", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Character, key);
            sc.define_name("liege", Scopes::Character, key);
            sc
        });

        vd.field_list("aptitude_level_breakpoints");
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

        // Documented `vassal_opinion` does not work
        vd.field_numeric("tax_factor");
        vd.field_numeric("levies_factor");

        // Undocumented
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

        // Undocumented; may have more scopes available
        vd.field_validated_key("ai_will_do", |key, bv, data| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("is_current_obligation", Scopes::Bool, key);
            sc.define_name("liege", Scopes::Character, key);
            sc.define_name("num_slots_with_obligation", Scopes::Value, key);
            sc.define_name("num_vassal_slots", Scopes::Value, key);
            validate_script_value_no_breakdown(bv, data, &mut sc);
        });
    }
}
