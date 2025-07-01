use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct AirGraphics {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::AirGraphics, AirGraphics::add)
}

impl AirGraphics {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AirGraphics, key, block, Box::new(Self {}));
    }
}

impl DbKind for AirGraphics {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_block("entities", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                let mut vd = Validator::new(block, data);
                vd.field_item("name", Item::Entity);
                vd.field_trigger_rooted("possible", Tooltipped::No, Scopes::Country);
                vd.field_script_value_no_breakdown_rooted("weight", Scopes::Country);
            });
        });

        vd.field_trigger_rooted("possible", Tooltipped::No, Scopes::Country);
        vd.field_numeric("entity_length");
        vd.field_numeric("speed");
        vd.field_integer("max_count");
        vd.field_numeric("height");
    }
}
