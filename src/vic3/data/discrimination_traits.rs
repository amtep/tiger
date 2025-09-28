use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DiscriminationTrait {}
#[derive(Clone, Debug)]
pub struct DiscriminationTraitGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DiscriminationTrait, DiscriminationTrait::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::DiscriminationTraitGroup, DiscriminationTraitGroup::add)
}

impl DiscriminationTrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiscriminationTrait, key, block, Box::new(Self {}));
    }
}
impl DiscriminationTraitGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DiscriminationTraitGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for DiscriminationTrait {
    fn add_subitems(&self, key: &Token, block: &Block, db: &mut Db) {
        if let Some(token) = block.get_field_value("type") {
            if token.is("tradition") {
                db.add_flag(Item::TraditionTrait, key.clone());
            } else if token.is("language") {
                db.add_flag(Item::LanguageTrait, key.clone());
            } else if token.is("heritage") {
                db.add_flag(Item::HeritageTrait, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_choice("type", &["heritage", "language", "tradition"]);
        if block.get_field_value("type").is_some_and(|token| token.is("tradition")) {
            vd.ban_field("trait_group", || "language and tradition types");
        } else {
            // TODO: check that heritage traits have heritage trait groups and language traits have
            // language trait groups
            vd.field_item("trait_group", Item::DiscriminationTraitGroup);
        }
    }
}

impl DbKind for DiscriminationTraitGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_choice("type", &["heritage", "language"]);
    }
}
