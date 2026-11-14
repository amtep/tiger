use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct MorphemeStripRule {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::MorphemeStripRule, MorphemeStripRule::add)
}

impl MorphemeStripRule {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::MorphemeStripRule, key, block, Box::new(Self {}));
    }
}

impl DbKind for MorphemeStripRule {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("seek_from", &["head", "tail"]);
        vd.multi_field_validated_block("rule", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("match");
            vd.multi_field_validated_list("match", |token, _| {
                // Each match block is a set of single characters
                if token.as_str().chars().count() != 1 {
                    let msg = "expected a single character";
                    warn(ErrorKey::Validation).msg(msg).loc(token).push();
                }
            });
            vd.req_field("strip");
            vd.field_integer_range("strip", 0..);
        });
    }
}
