use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct StraitDefinition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::StraitDefinition, StraitDefinition::add)
}

impl StraitDefinition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::StraitDefinition, key, block, Box::new(Self {}));
    }
}

impl DbKind for StraitDefinition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_choice("type", &["natural", "artificial"]);
        vd.field_bool("total_block_requires_full_control");
        vd.field_bool("military_block_requires_full_control");
        vd.field_item("first_land_endpoint", Item::Province);
        vd.field_item("second_land_endpoint", Item::Province);
        vd.field_item("first_sea_endpoint", Item::Province);
        vd.field_item("second_sea_endpoint", Item::Province);
    }
}
