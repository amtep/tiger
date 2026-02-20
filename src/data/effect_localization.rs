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
pub struct EffectLocalization {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::jomini(), Item::EffectLocalization, EffectLocalization::add)
}

impl EffectLocalization {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::EffectLocalization, key, block, Box::new(Self {}));
    }

    pub fn validate_use(
        key: &Token,
        block: &Block,
        data: &Everything,
        caller: &Token,
        tooltipped: Tooltipped,
    ) {
        match tooltipped {
            Tooltipped::No => (),
            Tooltipped::Yes | Tooltipped::FailuresOnly => {
                for field in &["global", "first", "third"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                let msg = "missing present perspective";
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).loc_msg(key, "here").push();
            }
            Tooltipped::Past => {
                for field in &["global_past", "first_past", "third_past"] {
                    if block.has_key(field) {
                        return;
                    }
                }
                for field in &["global", "first", "third"] {
                    if let Some(token) = block.get_field_value(field) {
                        // TODO: check if these are auto-guessed if _past key is missing
                        let loca = format!("{token}_PAST");
                        if data.item_exists(Item::Localization, &loca) {
                            return;
                        }
                    }
                }
                let msg = "missing `_past` perspective";
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).loc_msg(key, "here").push();
            }
            #[cfg(feature = "hoi4")]
            Tooltipped::Inner => unimplemented!(),
        }
    }
}

impl DbKind for EffectLocalization {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        #[cfg(feature = "eu5")]
        vd.field_item("none", Item::Localization);
        #[cfg(feature = "eu5")]
        vd.field_item("none_past", Item::Localization);
        vd.field_item("global", Item::Localization);
        vd.field_item("global_past", Item::Localization);
        vd.field_item("global_neg", Item::Localization);
        vd.field_item("global_past_neg", Item::Localization);
        vd.field_item("first", Item::Localization);
        vd.field_item("first_past", Item::Localization);
        vd.field_item("first_neg", Item::Localization);
        vd.field_item("first_past_neg", Item::Localization);
        vd.field_item("third", Item::Localization);
        vd.field_item("third_past", Item::Localization);
        vd.field_item("third_neg", Item::Localization);
        vd.field_item("third_past_neg", Item::Localization);
    }
}

pub fn validate_effect_localization(caller: &Token, data: &Everything, tooltipped: Tooltipped) {
    if let Some((key, block)) = data.get_key_block(Item::EffectLocalization, caller.as_str()) {
        EffectLocalization::validate_use(key, block, data, caller, tooltipped);
        return;
    }

    // As of CK3 1.18, effect localizations don't have to be defined and can just be present as
    // localizations.
    if Game::is_ck3() {
        match tooltipped {
            Tooltipped::No => (),
            Tooltipped::Yes | Tooltipped::FailuresOnly => {
                if data.item_exists(Item::Localization, caller.as_str()) {
                    return;
                }
                for sfx in &["global", "first", "third"] {
                    let loca = format!("{caller}_{sfx}");
                    if data.item_exists(Item::Localization, &loca) {
                        return;
                    }
                }
                let msg = "missing present perspective";
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).push();
            }
            Tooltipped::Past => {
                for sfx in &["global_part", "first_part", "third_past"] {
                    let loca = format!("{caller}_{sfx}");
                    if data.item_exists(Item::Localization, &loca) {
                        return;
                    }
                }
                let msg = "missing past perspective";
                warn(ErrorKey::MissingPerspective).msg(msg).loc(caller).push();
            }
            #[cfg(feature = "hoi4")]
            Tooltipped::Inner => unimplemented!(),
        }
    } else {
        data.verify_exists(Item::EffectLocalization, caller);
    }
}
