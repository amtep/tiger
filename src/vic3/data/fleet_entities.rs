use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::vic3::validate::validate_locators;

#[derive(Clone, Debug)]
pub struct FleetEntity {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::FleetEntity, FleetEntity::add)
}

impl FleetEntity {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FleetEntity, key, block, Box::new(Self {}));
    }
}

impl DbKind for FleetEntity {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::ShipType, key);

        vd.field_bool("default");
        vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Country);

        vd.field_choice("weapon_type", &["none", "cannon", "turret", "torpedo"]);
        vd.field_numeric("weapon_fire_interval");

        vd.field_choice("hull_type", &["wood", "steel"]);
        vd.field_numeric("motion_intensity");

        vd.field_numeric("length");

        vd.field_item("entity", Item::Entity);

        validate_locators(&mut vd);

        vd.multi_field_validated_block("vfx_entity", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("entity", Item::Entity);
            vd.field_bool("attach");
            // TODO: this requires inspecting the ship model file
            vd.field_value("locator");
        });

        vd.field_validated_block("damage_decal_positions", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                let mut vd = Validator::new(block, data);
                let values = vd.values();
                if values.len() != 4 {
                    warn(ErrorKey::Validation).msg("expected exactly 4 numbers").loc(block).push();
                }
                for value in vd.values() {
                    value.expect_number();
                }
            }
        });

        vd.multi_field_validated_block("modification", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("name");
            vd.field_item("slot_type", Item::ShipModificationSlot);
            // TODO: The number of entities must be equal to the number of modification levels in the slot
            vd.multi_field_item("entity", Item::Entity);
        });

        vd.multi_field_validated_block("attach", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_value("locator");
            vd.field_item("slot_type", Item::ShipModificationSlot);
            // TODO: The number of entities must be equal to the number of modification levels in the slot
            vd.multi_field_validated_block("modification", |block, data| {
                let mut vd = Validator::new(block, data);
                // TODO: name of a modification specified above
                vd.field_value("name");
                // TODO: mutually exclusive with `name`
                vd.field_item("entity", Item::Entity);
                vd.field_bool("disabled");
                vd.field_numeric("fire_angle");
            });
        });
    }
}
