use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct BuildingCategory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Eu5, Item::BuildingCategory, BuildingCategory::add)
}

impl BuildingCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BuildingCategory, key, block, Box::new(Self {}));
    }
}

impl DbKind for BuildingCategory {
    fn validate(&self, key: &Token, _block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);
    }
}
