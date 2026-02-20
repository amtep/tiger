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
pub struct ArtistType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Eu5, Item::ArtistType, ArtistType::add)
}

impl ArtistType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtistType, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtistType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let loca_name = format!("ARTIST_TYPE_NAME_{key}");
        data.verify_exists_implied(Item::Localization, &loca_name, key);
        let loca_desc = format!("ARTIST_TYPE_DESC_{key}");
        data.verify_exists_implied(Item::Localization, &loca_desc, key);

        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        vd.field_trigger("potential", Tooltipped::No, &mut sc);
    }
}
