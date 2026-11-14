use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err, untidy};
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct OpinionModifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::OpinionModifier, OpinionModifier::add)
}

impl OpinionModifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::OpinionModifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for OpinionModifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // TODO: figure out when it is necessary to localize an opinion.
        // Opinions given from traits don't need to be localized, for one.
        // Maybe only ones used with add_opinion
        // data.verify_exists(Item::Localization, key);
        data.mark_used(Item::Localization, key.as_str());

        vd.field_integer("opinion");

        vd.field_bool("decaying");
        let decaying = block.get_field_bool("decaying").unwrap_or(false);
        for field in &["delay_days", "delay_weeks", "delay_months", "delay_years"] {
            if decaying {
                vd.field_integer(field);
            } else {
                vd.ban_field(field, || "decaying opinions");
            }
        }

        let mut duration = None;
        for field in &["days", "weeks", "months", "years"] {
            duration = duration.or_else(|| block.get_key(field));
            vd.field_integer(field);
        }

        // The docs give the rules below
        if decaying && !block.has_key("monthly_change") && duration.is_none() {
            let msg = "decaying opinion needs `monthly_change` or a duration";
            err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
        }
        // The docs say negative values are clamped to 0
        vd.field_numeric_range("monthly_change", 0.0..);
        if let Some(key) = block.get_key("monthly_change") {
            if let Some(duration) = duration {
                let msg = "`monthly_change` and a duration are mutually exclusive";
                err(ErrorKey::Validation).msg(msg).loc(key).loc_msg(duration, "duration").push();
            }
            if !decaying && !block.get_field_bool("growing").unwrap_or(false) {
                let msg = "`monthly_change` does nothing without `decaying` or `growing`";
                untidy(ErrorKey::Unneeded).msg(msg).loc(key).push();
            }
        }

        vd.field_bool("growing");
        if let Some(key) = block.get_key("growing")
            && block.get_field_bool("growing").unwrap_or(false)
        {
            if decaying {
                let msg = "`growing` and `decaying` are mutually exclusive";
                err(ErrorKey::Validation).msg(msg).loc(key).push();
            }
            if !block.has_key("monthly_change") {
                let msg = "growing opinions need `monthly_change`";
                let info = "the docs say growing by duration is not implemented";
                err(ErrorKey::FieldMissing).msg(msg).info(info).loc(key).push();
            }
        }
        vd.field_bool("stacking");

        vd.field_integer("min");
        vd.field_integer("max");
        if !block.get_field_bool("stacking").unwrap_or(false) {
            for field in &["min", "max"] {
                if let Some(key) = block.get_key(field) {
                    let msg = format!("`{field}` does nothing without `stacking = yes`");
                    untidy(ErrorKey::Unneeded).msg(msg).loc(key).push();
                }
            }
        }
        if let Some(key) = block.get_key("min")
            && let Some(min) = block.get_field_integer("min")
            && let Some(max) = block.get_field_integer("max")
            && min > max
        {
            let msg = "`min` must not be higher than `max`";
            err(ErrorKey::Range).msg(msg).loc(key).push();
        }

        vd.field_bool("imprisonment_reason");
        vd.field_bool("banish_reason");
        vd.field_bool("execute_reason");
        vd.field_bool("revoke_title_reason");
        vd.field_bool("divorce_reason");

        vd.field_bool("obedient");

        vd.advice_field("disable_non_aggression_pacts", "removed in 1.12");
        vd.advice_field("non_aggression_pact", "removed in 1.12");
        vd.advice_field("non_interference", "removed in 1.12");
    }
}
