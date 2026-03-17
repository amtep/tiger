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
pub struct FlatMapStyle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::FlatMapStyle, FlatMapStyle::add)
}

impl FlatMapStyle {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FlatMapStyle, key, block, Box::new(Self {}));
    }
}

impl DbKind for FlatMapStyle {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_trigger_rooted("is_shown", Tooltipped::No, Scopes::Character);
        vd.field_item("dlc_feature", Item::DlcFeature);
        vd.field_integer("priority");
        vd.field_bool("default");
        if let Some(token) = vd.field_value("texture") {
            let pathname = format!("gfx/map/terrain/flat_maps/{token}");
            data.verify_exists_implied(Item::File, &pathname, token);
        }
    }
}
