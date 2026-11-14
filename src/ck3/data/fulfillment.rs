use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashMap;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, err, untidy, warn};
use crate::token::{Loc, Token};
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct SpiritualFulfillmentType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::SpiritualFulfillmentType, SpiritualFulfillmentType::add)
}

impl SpiritualFulfillmentType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::SpiritualFulfillmentType, key, block, Box::new(Self {}));
    }

    /// The docs say no two types may share a religion, since that is how characters get their
    /// type, and that one type should have an empty religion list as the fallback
    pub fn validate_all(db: &Db, _data: &Everything) {
        // Each religion with its type
        let mut seen: TigerHashMap<&str, (&Token, &Token)> = TigerHashMap::default();
        let mut fallback: Option<&Token> = None;
        // Go in load order, so that reports point at the later type
        let mut types: Vec<_> = db.iter_key_block(Item::SpiritualFulfillmentType).collect();
        types.sort_by_key(|(key, _)| key.loc);
        for (key, block) in types {
            let religions: Vec<&Token> = block
                .get_field_block("religions")
                .map(|block| block.iter_values().collect())
                .unwrap_or_default();
            if religions.is_empty() {
                if let Some(other) = fallback {
                    let msg = "more than one spiritual fulfillment type without religions";
                    warn(ErrorKey::Conflict).msg(msg).loc(key).loc_msg(other, "other type").push();
                } else {
                    fallback = Some(key);
                }
            }
            for religion in religions {
                match seen.get(religion.as_str()) {
                    Some(&(other_type, other)) if other_type == key => {
                        let msg = format!("religion {religion} is listed twice");
                        untidy(ErrorKey::DuplicateItem)
                            .msg(msg)
                            .loc(religion)
                            .loc_msg(other, "earlier listing")
                            .push();
                    }
                    Some(&(_, other)) => {
                        let msg = format!(
                            "religion {religion} is in more than one spiritual fulfillment type"
                        );
                        err(ErrorKey::Conflict)
                            .msg(msg)
                            .loc(religion)
                            .loc_msg(other, "other type")
                            .push();
                    }
                    None => {
                        seen.insert(religion.as_str(), (key, religion));
                    }
                }
            }
        }
    }
}

impl DbKind for SpiritualFulfillmentType {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for block in block.get_field_blocks("level") {
            if let Some(block) = block.get_field_block("flags") {
                for token in block.iter_values() {
                    db.add_flag(Item::FulfillmentParameter, token.clone());
                }
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_list_items("religions", Item::Religion);

        let min = data
            .get_defined_string("NReligion|MIN_SPIRITUAL_FULFILLMENT")
            .and_then(Token::get_number)
            .unwrap_or(-100.0);
        let max = data
            .get_defined_string("NReligion|MAX_SPIRITUAL_FULFILLMENT")
            .and_then(Token::get_number)
            .unwrap_or(100.0);

        vd.req_field("level");
        let mut levels = 0;
        let mut thresholds: TigerHashMap<u64, Loc> = TigerHashMap::default();
        vd.multi_field_validated_block("level", |block, data| {
            levels += 1;
            let mut vd = Validator::new(block, data);
            vd.field_numeric_range("threshold", min..=max);
            let threshold =
                block.get_field_value("threshold").and_then(Token::get_number).unwrap_or(min);
            // The docs say the thresholds must be unique. Adding 0.0 turns -0.0 into 0.0.
            let bits = (threshold + 0.0).to_bits();
            if let Some(&other) = thresholds.get(&bits) {
                let msg = "levels must have unique thresholds";
                err(ErrorKey::DuplicateField)
                    .msg(msg)
                    .loc(block)
                    .loc_msg(other, "other level")
                    .push();
            } else {
                thresholds.insert(bits, block.loc);
            }

            vd.field_validated_block("modifier", |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Character, vd);
            });
            vd.req_field("icon");
            vd.field_item("icon", Item::File);
            vd.field_validated_list("flags", |flag, data| {
                let loca = format!("fulfillment_level_parameter_{flag}");
                data.verify_exists_implied(Item::Localization, &loca, flag);
            });
        });

        // The docs say level names are `<key>_level_<index>`, falling back to
        // `default_fulfillment_level_<index>`, where the index is the position by threshold
        for index in 0..levels {
            let loca = format!("{key}_level_{index}");
            let fallback = format!("default_fulfillment_level_{index}");
            if data.item_exists(Item::Localization, &loca) {
                data.mark_used(Item::Localization, &loca);
            } else if data.item_exists(Item::Localization, &fallback) {
                data.mark_used(Item::Localization, &fallback);
            } else {
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }
    }
}
