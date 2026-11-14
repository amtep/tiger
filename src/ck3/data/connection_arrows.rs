use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err, warn};
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
        vd.req_field("arrow_type");
        if let Some(token) = vd.field_value("arrow_type") {
            data.verify_exists(Item::Asset, token);
            // The docs say this one is used on the flat map
            let flat = format!("{token}FlatMap");
            data.verify_exists_implied(Item::Asset, &flat, token);
        }
        vd.req_field("provinces");
        vd.field_validated_key_block("provinces", |key, block, data| {
            let mut vd = Validator::new(block, data);
            let mut seen: TigerHashSet<&Token> = TigerHashSet::default();
            let mut count = 0;
            for token in vd.values() {
                count += 1;
                data.verify_exists(Item::Province, token);
                if let Some(&other) = seen.get(token) {
                    let msg = format!("duplicate province {token}");
                    err(ErrorKey::Conflict).msg(msg).loc(token).loc_msg(other, "other").push();
                } else {
                    seen.insert(token);
                }
            }
            if count <= 2 {
                let msg = "connection arrows need more than 2 provinces";
                let info = "otherwise no arrow is drawn";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
            }
        });
    }
}
