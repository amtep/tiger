use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct AiStrategicRegionStanceType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::AiStrategicRegionStanceType, AiStrategicRegionStanceType::add)
}

impl AiStrategicRegionStanceType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::AiStrategicRegionStanceType, key, block, Box::new(Self {}));
    }
}

impl DbKind for AiStrategicRegionStanceType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // There's no UI for these yet.
        data.mark_used(Item::Localization, key.as_str());

        vd.field_item("icon", Item::File);
        vd.field_bool("can_send_fleets");

        // undocumented

        vd.field_bool("add_diplomatic_relevance");
    }
}
