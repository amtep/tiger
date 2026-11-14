use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PoolSelector {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::PoolSelector, PoolSelector::add)
}

impl PoolSelector {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PoolSelector, key, block, Box::new(Self {}));
    }
}

impl DbKind for PoolSelector {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("base", Scopes::Character, key);
        if key.is("auto_generated_title_holder")
            || key.is("auto_generated_local_title_holder")
            || key.is("pool_theocratic_succession")
        {
            sc.define_name("province", Scopes::Province, key);
        }

        vd.field_trigger("valid_character", Tooltipped::No, &mut sc);
        vd.field_validated_block_sc("character_score", &mut sc, validate_modifiers_with_base);
        vd.field_validated_block("config", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("background", Item::CharacterBackground);
            vd.field_list_integers_exactly("age", 2);
        });

        vd.field_integer("selection_count");

        vd.field_choice("pool", &["faith", "heritage"]);
        vd.field_bool("court");
        vd.field_choice(
            "gender",
            &["random", "male", "female", "clergy", "commander", "gender_law"],
        );
        vd.field_choice("culture", &["character", "province", "company"]);
    }
}

#[derive(Clone, Debug)]
pub struct CharacterBackground {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CharacterBackground, CharacterBackground::add)
}

impl CharacterBackground {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterBackground, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterBackground {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.multi_field_item("trait", Item::Trait);
    }
}
