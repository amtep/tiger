use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MenuScene {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::MenuScene, MenuScene::add)
}

impl MenuScene {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MenuScene, key, block, Box::new(Self {}));
    }
}

impl DbKind for MenuScene {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("dlc_feature", Item::DlcFeature);
        for slot in ["main", "heir", "secondary"] {
            vd.req_field(slot);
            vd.field_item(slot, Item::BookmarkPortrait);
            vd.field_item(&format!("animation_{slot}"), Item::PortraitAnimation);
        }
        vd.req_field("background");
        vd.field_validated_value("background", |_, mut vd| {
            let pathname = format!("gfx/interface/illustrations/{}", vd.value());
            data.verify_exists_implied(Item::File, &pathname, vd.value());
            vd.accept();
        });
        vd.field_integer("priority");
    }
}
