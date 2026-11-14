use crate::block::Block;
use crate::ck3::data::titles::Tier;
use crate::ck3::modif::ModifKinds;
use crate::ck3::validate::validate_parameter_value;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, fatal, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct HolySite {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HolySite, HolySite::add)
}

impl HolySite {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HolySite, key, block, Box::new(Self {}));
    }
}

impl DbKind for HolySite {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("parameters") {
            for token in block.iter_values() {
                db.add_flag(Item::HolySiteParameter, token.clone());
            }
            for (key, _) in block.iter_assignments() {
                db.add_flag(Item::HolySiteParameter, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("holy_site_{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);
        // The docs say this is only used if it exists
        let loca = format!("holy_site_{key}_effects");
        data.mark_used(Item::Localization, &loca);

        // A dynamic holy site is a general type without a location
        let dynamic = block.field_value_is("is_dynamic", "yes");
        vd.field_bool("is_dynamic");
        if dynamic {
            let loca = format!("holy_site_{key}_dynamic_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
            vd.ban_field("county", || "holy sites without `is_dynamic = yes`");
            vd.ban_field("barony", || "holy sites without `is_dynamic = yes`");
        } else {
            vd.req_field("county");
            vd.field_item("county", Item::Title);
            vd.field_item("barony", Item::Title);
        }

        if !dynamic && let Some(county) = block.get_field_value("county") {
            if Tier::try_from(county) != Ok(Tier::County) {
                warn(ErrorKey::TitleTier).msg("must be a county").loc(county).push();
            }
            if let Some(barony) = block.get_field_value("barony") {
                if Tier::try_from(barony) != Ok(Tier::Barony) {
                    warn(ErrorKey::TitleTier).msg("must be a barony").loc(barony).push();
                }
                if let Some(title) = data.titles.get(barony.as_str())
                    && title.parent != Some(county.as_str())
                {
                    let msg = format!("barony not in specified county {county}");
                    fatal(ErrorKey::Crash).strong().msg(msg).loc(barony).push();
                }
            }
        }

        vd.field_validated_block("parameters", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.values();
            // The docs say the `parameter = yes` form is still accepted, ignoring the value
            vd.unknown_value_fields(validate_parameter_value);
        });

        vd.replaced_field("character_modifier", "faith_character_modifier");
        vd.field_validated_block("faith_character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("faith_character_modifier_scaled", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("scale");
            vd.field_script_value_rooted("scale", Scopes::HolySite);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("county_holder_character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.multi_field_validated_block("county_modifier_scaled", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("scale");
            vd.field_script_value_rooted("scale", Scopes::HolySite);
            validate_modifs(block, data, ModifKinds::County, vd);
        });

        // undocumented

        vd.field_bool("is_active");
    }
}
