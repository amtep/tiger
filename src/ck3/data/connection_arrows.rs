use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ConnectionArrow {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ConnectionArrow, ConnectionArrow::add)
}

impl ConnectionArrow {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ConnectionArrow, key, block, Box::new(Self {}));
    }
}

impl DbKind for ConnectionArrow {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_bool("is_primary");
        // TODO: process the arrowType assets and reference them here
        vd.field_value("arrow_type");
        vd.field_list_items("provinces", Item::Province);
    }
}
