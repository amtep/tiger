use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

const CONCEPT_FAMILIES: &[&str] = &[
    "good",
    "tax_base",
    "religion",
    "culture",
    "food",
    "integration_status",
    "combat",
    "dice",
    "cabinet",
    "market",
    "mercenary",
    "trait",
    "role",
    "location_rank",
    "ability",
];

#[derive(Clone, Debug)]
pub struct GameConcept {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Eu5, Item::GameConcept, GameConcept::add)
}

impl GameConcept {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GameConcept, key, block, Box::new(Self {}));
    }
}

impl DbKind for GameConcept {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(list) = block.get_field_list("alias") {
            for token in list {
                db.add_flag(Item::GameConcept, token);
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca_name = format!("game_concept_{key}");
        let loca_desc = format!("game_concept_{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca_name, key);
        data.verify_exists_implied(Item::Localization, &loca_desc, key);

        if let Some(token) = vd.field_value("texture") {
            let pathname = format!("gfx/interface/icons/{token}.dds");
            data.verify_exists_implied(Item::File, &pathname, token);
        }

        vd.multi_field_validated_key("alias", |key, _bv, data| {
            data.verify_exists_implied(Item::Localization, &loca_name, key);
        });

        vd.field_bool("shown_in_loading_screen");

        vd.field_choice("family", CONCEPT_FAMILIES);

        vd.field_item("tooltip_map_mode", Item::MapMode);

        vd.field_item("tutorial_lesson", Item::TutorialLesson);
    }
}
