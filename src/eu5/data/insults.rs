use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct InsultType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Eu5, Item::InsultType, InsultType::add)
}

impl InsultType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::InsultType, key, block, Box::new(Self {}));
    }
}

impl DbKind for InsultType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        sc.define_name("actor", Scopes::Country, key);
        sc.define_name("recipient", Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
    }
}
