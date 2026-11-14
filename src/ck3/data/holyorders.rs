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
pub struct HolyOrderType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HolyOrderType, HolyOrderType::add)
}

impl HolyOrderType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HolyOrderType, key, block, Box::new(Self {}));
    }
}

impl DbKind for HolyOrderType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // The trigger and weight have an empty scope
        let build_sc = |key: &Token| {
            let mut sc = ScopeContext::new(Scopes::None, key);
            sc.define_name("leader", Scopes::Character, key);
            sc.define_name("founder", Scopes::Character, key);
            sc.define_name("province", Scopes::Province, key);
            sc
        };

        vd.req_field("type");
        vd.field_choice("type", &["monastic", "military"]);
        vd.field_item("government", Item::GovernmentType);
        vd.field_trigger_builder("trigger", Tooltipped::No, build_sc);
        vd.field_script_value_builder("weight", build_sc);

        vd.field_item("title", Item::Title);
        if !vd.field_item("name", Item::Localization) && !block.has_key("title") {
            data.verify_exists(Item::Localization, key);
        }
        vd.field_item("coat_of_arms", Item::Coa);
        if block.field_value_is("type", "military") {
            vd.field_list_items("men_at_arms", Item::MenAtArms);
        } else {
            vd.ban_field("men_at_arms", || "military holy orders");
        }
        vd.field_item("tenet", Item::Tenet);

        let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
        sc.define_name("leader", Scopes::Character, key);
        sc.define_name("founder", Scopes::Character, key);
        sc.define_name("province", Scopes::Province, key);
        sc.define_name("title", Scopes::LandedTitle, key);
        sc.define_name("new_holy_order", Scopes::HolyOrder, key);
        sc.define_name("rite", Scopes::Rite, key);
        vd.field_effect("effect", Tooltipped::No, &mut sc);
    }
}
