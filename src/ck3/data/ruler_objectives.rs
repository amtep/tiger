use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct RulerObjectiveType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::RulerObjectiveType, RulerObjectiveType::add)
}

impl RulerObjectiveType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::RulerObjectiveType, key, block, Box::new(Self {}));
    }
}

impl DbKind for RulerObjectiveType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // Verified: fields in this item can set scopes for later fields
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("title", Scopes::LandedTitle, key);

        vd.field_list_items("decisions", Item::Decision);

        vd.field_trigger("is_valid_advice", Tooltipped::No, &mut sc);
        vd.field_trigger("is_doing", Tooltipped::No, &mut sc);
        sc.define_name("doing", Scopes::Bool, key);
        vd.field_trigger("is_valid_for_title", Tooltipped::No, &mut sc);
        vd.field_script_value_no_breakdown("relevance", &mut sc);

        vd.field_validated_sc("summary", &mut sc, validate_desc);
        vd.field_validated_sc("description", &mut sc, validate_desc);
    }
}
