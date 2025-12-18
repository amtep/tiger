use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GraphicalUnitType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::GraphicalUnitType, GraphicalUnitType::add)
}

impl GraphicalUnitType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GraphicalUnitType, key, block, Box::new(Self {}));
    }
}

impl DbKind for GraphicalUnitType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let mut count = 0;
        for define in &[
            "NGameIcons|GRAPHICAL_UNIT_TYPES_LEVIES_ILLUSTRATIONS_SMALL_PATH",
            "NGameIcons|GRAPHICAL_UNIT_TYPES_LEVIES_ILLUSTRATIONS_BIG_PATH",
            "NGameIcons|GRAPHICAL_UNIT_TYPES_KNIGHTS_ILLUSTRATIONS_SMALL_PATH",
            "NGameIcons|GRAPHICAL_UNIT_TYPES_KNIGHTS_ILLUSTRATIONS_BIG_PATH",
        ] {
            if let Some(icon_path) = data.get_defined_string_warn(key, define) {
                let pathname = format!("{icon_path}/{key}.dds");
                if data.item_exists(Item::File, &pathname) {
                    count += 1;
                }
            }
        }
        if count == 0 && !key.is("default") {
            let msg = "no knight or levy illustrations found for `{key}`";
            warn(ErrorKey::MissingFile).msg(msg).loc(key).push();
        }

        vd.field_list_items("graphical_cultures", Item::UnitGfx);
    }
}
