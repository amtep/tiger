use crate::block::Block;
use crate::ck3::data::doctrines::validate_doctrine_groups;
use crate::date::Date;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct FaithHistory {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::FaithHistory, FaithHistory::add)
}

impl FaithHistory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FaithHistory, key, block, Box::new(Self {}));
    }
}

impl DbKind for FaithHistory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Faith, key);

        let mut vd = Validator::new(block, data);
        vd.validate_history_blocks(validate_history);
    }
}

fn validate_history(_date: Date, _key: &Token, block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    vd.field_bool("created");
    vd.field_item("main_rite", Item::RiteType);
    vd.field_item("religious_head", Item::Title);

    vd.field_list_items("known", Item::Tenet);
    vd.field_list_items("permitted", Item::Tenet);
    vd.field_list_items("prohibited", Item::Tenet);
    validate_tenet_status(block);

    vd.field_validated_block("popularity", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::Tenet, key);
            value.expect_number();
        });
    });

    vd.field_validated_block("rites", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_block_fields(|key, block| {
            // The key is only the rite if there is no `rite` field
            if !block.has_key("rite") {
                data.verify_exists(Item::RiteType, key);
            }
            validate_rite(block, data);
        });
    });
}

/// The docs say the game warns about tenets that are in more than one of these lists
fn validate_tenet_status(block: &Block) {
    let mut seen: TigerHashSet<Token> = TigerHashSet::default();
    for field in ["known", "permitted", "prohibited"] {
        for tenet in block.get_field_list(field).unwrap_or_default() {
            if let Some(other) = seen.get(&tenet) {
                let msg = format!("tenet {tenet} is listed twice");
                warn(ErrorKey::DuplicateItem)
                    .msg(msg)
                    .loc(&tenet)
                    .loc_msg(other, "earlier listing")
                    .push();
            } else {
                seen.insert(tenet);
            }
        }
    }
}

fn validate_rite(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);

    // An empty `rite` means the faith's current main rite
    vd.field_validated_value("rite", |_, mut vd| {
        if !vd.maybe_is("") {
            vd.item(Item::RiteType);
        }
    });
    vd.field_bool("enabled");
    vd.multi_field_validated_block("tenet_setup", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_list_items("dlc_feature", Item::DlcFeature);
        vd.req_field("tenets");
        vd.field_list_items("tenets", Item::Tenet);
    });
    vd.field_list_items("doctrines", Item::Doctrine);
    if let Some(doctrines) = block.get_field_list("doctrines") {
        validate_doctrine_groups(&doctrines, data);
    }
}
