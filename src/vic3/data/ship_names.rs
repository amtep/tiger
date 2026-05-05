use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ShipNameDefinition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ShipNameDefinition, ShipNameDefinition::add)
}

impl ShipNameDefinition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipNameDefinition, key, block, Box::new(Self {}));
    }
}

impl DbKind for ShipNameDefinition {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("name", Item::Localization);
        vd.field_numeric("selection_weight");
        vd.field_item("country", Item::Country);

        vd.field_list_items("allowed_ship_types", Item::ShipType);
        vd.field_validated_block("properties", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                let mut vd = Validator::new(block, data);
                vd.field_choice("type", &["custom_text", "name_list"]);
                vd.field_item("key", Item::Localization);
                vd.field_item("custom_text", Item::Localization);
                vd.multi_field_item("quick_trigger_required_law", Item::LawType);
                vd.field_bool("quick_trigger_country_leader_female");
                vd.field_list_items("name_list", Item::Localization);
            }
        });
    }
}
