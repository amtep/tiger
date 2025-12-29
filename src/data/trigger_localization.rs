use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct TriggerLocalization {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::jomini(), Item::TriggerLocalization, TriggerLocalization::add)
}

impl TriggerLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::TriggerLocalization, key, block, Box::new(Self {}));
    }

    pub fn validate_use(
        key: &Token,
        block: &Block,
        data: &Everything,
        caller: &Token,
        tooltipped: Tooltipped,
        negated: bool,
    ) {
        if tooltipped.is_tooltipped() {
            if negated {
                for field in &["global_not", "first_not", "third_not", "none_not"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                for field in &["global", "first", "third"] {
                    if let Some(token) = block.get_field_value(field) {
                        let loca = format!("NOT_{token}");
                        if data.item_exists(Item::Localization, &loca) {
                            return;
                        }
                    }
                }
                let msg = format!("missing `NOT_` perspective for {key}");
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).loc_msg(key, "here").push();
            } else {
                for field in &["global", "first", "third", "none"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                let msg = format!("missing positive perspective for {key}");
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).loc_msg(key, "here").push();
            }
        }
    }
}

impl DbKind for TriggerLocalization {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_item("global", Item::Localization);
        vd.field_item("global_not", Item::Localization);
        vd.field_item("first", Item::Localization);
        vd.field_item("first_not", Item::Localization);
        vd.field_item("third", Item::Localization);
        vd.field_item("third_not", Item::Localization);
        vd.field_item("none", Item::Localization);
        vd.field_item("none_not", Item::Localization);
    }
}

pub fn validate_trigger_localization(
    caller: &Token,
    data: &Everything,
    tooltipped: Tooltipped,
    negated: bool,
) {
    if let Some((key, block)) = data.get_key_block(Item::TriggerLocalization, caller.as_str()) {
        TriggerLocalization::validate_use(key, block, data, caller, tooltipped, negated);
        return;
    }

    // As of CK3 1.18, trigger localizations don't have to be defined and can just be present as
    // localizations.
    if Game::is_ck3() {
        if tooltipped.is_tooltipped() {
            if negated {
                for sfx in &["global_not", "first_not", "third_not", "none_not"] {
                    let loca = format!("{caller}_{sfx}");
                    if data.item_exists(Item::Localization, &loca) {
                        return;
                    }
                }
                let msg = format!("missing negated perspective for {caller}");
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).push();
            } else {
                if data.item_exists(Item::Localization, caller.as_str()) {
                    return;
                }
                for sfx in &["global", "first", "third", "none"] {
                    let loca = format!("{caller}_{sfx}");
                    if data.item_exists(Item::Localization, &loca) {
                        return;
                    }
                }
                let msg = format!("missing positive perspective for {caller}");
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).push();
            }
        }
    } else {
        data.verify_exists(Item::TriggerLocalization, caller);
    }
}
