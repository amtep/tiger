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
pub struct FrontGraphics {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::FrontGraphics, FrontGraphics::add)
}

impl FrontGraphics {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FrontGraphics, key, block, Box::new(Self {}));
    }
}

impl DbKind for FrontGraphics {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let sc_builder = |key: &Token| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("target", Scopes::Country, key);
            sc
        };
        let sc_builder2 = |key: &Token| {
            let mut sc = sc_builder(key);
            sc.define_name("front", Scopes::Front, key);
            sc.define_name("battle", Scopes::Battle, key);
            sc
        };

        vd.field_choice("type", &["battle", "after_battle", "on_battle_ended"]);
        vd.field_trigger_builder("possible", Tooltipped::No, sc_builder);
        vd.field_script_value_no_breakdown_builder("weight", sc_builder);
        vd.field_validated_block("participant_entities", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                let mut vd = Validator::new(block, data);
                vd.field_item("name", Item::Entity);
                vd.field_trigger_builder("trigger", Tooltipped::No, sc_builder2);
            });
        });

        vd.field_validated_block("environment_entity", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Entity);
            vd.field_bool("one_time_action");
        });
    }
}
