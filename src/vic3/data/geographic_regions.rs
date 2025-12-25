use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct GeographicRegion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::GeographicRegion, GeographicRegion::add)
}

impl GeographicRegion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::GeographicRegion, key, block, Box::new(Self {}));
    }
}

impl DbKind for GeographicRegion {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.req_field("short_key");
        vd.field_identifier("short_key", "geographic region short key");

        // TODO: validate this list of scopes (sr:some_strategic_region)
        vd.field_list("strategic_regions");

        vd.field_list_items("state_regions", Item::StateRegion);
    }
}
