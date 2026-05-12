use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_color;
use crate::validator::Validator;
use crate::vic3::modif::ModifKinds;
use crate::vic3::tables::modifs::maybe_warn_modifiable_capitalization;

#[derive(Clone, Debug)]
pub struct ShipType {}
#[derive(Clone, Debug)]
pub struct ShipGroup {}
#[derive(Clone, Debug)]
pub struct ShipVeterancyLevel {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ShipType, ShipType::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ShipGroup, ShipGroup::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ShipVeterancyLevel, ShipVeterancyLevel::add)
}

impl ShipType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipType, key, block, Box::new(Self {}));
    }
}
impl ShipGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipGroup, key, block, Box::new(Self {}));
    }
}
impl ShipVeterancyLevel {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipVeterancyLevel, key, block, Box::new(Self {}));
    }
}

impl DbKind for ShipType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        maybe_warn_modifiable_capitalization(key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Ship, vd);
        });

        vd.field_item("ship_group", Item::ShipGroup);

        vd.field_validated_block("construction_goods", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Goods, vd);
        });
        vd.field_validated_block("materiel_goods", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Goods, vd);
        });
        vd.field_validated_block("distance_to_port_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Ship, vd);
        });

        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.field_bool("use_modificaitons");
        vd.field_numeric("base_construction_cost");
        vd.field_numeric("modification_construction_cost");
        vd.field_numeric("combat_power_multiplier");

        vd.field_item("icon", Item::File);

        vd.field_script_value_no_breakdown_rooted("ai_weight", Scopes::Country);

        vd.multi_field_validated_block("modifications", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                let mut vd = Validator::new(block, data);
                data.verify_exists(Item::ShipModificationSlot, key);
                for value in vd.values() {
                    data.verify_exists(Item::ShipModification, value);
                }
            });
        });

        vd.multi_field_validated_block("default_modifications", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                data.verify_exists(Item::ShipModificationSlot, key);
                data.verify_exists(Item::ShipModification, value);
            });
        });

        // undocumented

        vd.field_bool("can_be_flagship");
        vd.field_trigger_rooted("is_obsolete", Tooltipped::No, Scopes::Country);
        vd.field_trigger_rooted("is_very_obsolete", Tooltipped::No, Scopes::Country);
        vd.field_item("profile_texture", Item::File);
    }
}

impl DbKind for ShipGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        maybe_warn_modifiable_capitalization(key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_validated_block("color", validate_color);
        vd.field_script_value_no_breakdown_rooted("ai_weight", Scopes::Country);

        // undocumented

        vd.field_choice("category", &["capital_group", "cruiser_group", "torpedo_group", "supply"]);
    }
}

impl DbKind for ShipVeterancyLevel {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_item("icon", Item::File);
        vd.field_numeric("experience_threshold");
        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Ship, vd);
        });
    }
}
