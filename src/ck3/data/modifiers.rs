#![allow(non_upper_case_globals)]

use crate::block::{BV, Block};
use crate::ck3::modif::ModifKinds;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::script_value::validate_script_value;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Modifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Modifier, Modifier::add)
}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // There are {key} and {key}_desc locas but both are optional
        data.mark_used(Item::Localization, key.as_str());
        let loca = format!("{key}_desc");
        data.mark_used(Item::Localization, &loca);

        vd.field_icon("icon", "NGameIcons|STATICMODIFIER_ICON_PATH", ".dds");

        vd.field_bool("stacking");
        vd.field_bool("hide_effects");
        // `scale` is validated in `validate_call`
        vd.field("scale");
        validate_modifs(block, data, ModifKinds::all(), vd);
    }

    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _from: &Token,
        _from_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let mut vd = Validator::new(block, data);
        // The docs say `this` is the object getting the modifier, and root and saved scopes are
        // those of the calling effect
        vd.field_validated("scale", |bv, data| match bv {
            BV::Value(_) => validate_script_value(bv, data, sc),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.req_field("value");
                vd.field_script_value("value", sc);
                vd.field_item("desc", Item::Localization);
                vd.field_choice("display_mode", &["default", "scaled", "unscaled"]);
            }
        });
        vd.no_warn_remaining();
    }

    fn validate_property_use(
        &self,
        _key: &Token,
        block: &Block,
        _property: &Token,
        caller: &str,
        data: &Everything,
    ) {
        let mut vd = Validator::new(block, data);
        // skip over the known fields
        vd.field("icon");
        vd.field("stacking");
        vd.field("hide_effects");
        vd.field("scale");

        // TODO: make validate_modifs explain why it expected this kind
        validate_modifs(block, data, get_modif_kinds(caller), vd);
    }
}

// LAST UPDATED CK3 VERSION 1.20.1
/// Get the modifier kinds from property name
/// See `effects.log` from the game data dumps.
fn get_modif_kinds(name: &str) -> ModifKinds {
    for substr in [
        "artifact_modifier",
        "character_modifier",
        "dynasty_modifier",
        "house_modifier",
        "house_unity_modifier",
        "legend_owner_modifier",
    ] {
        if name.contains(substr) {
            return ModifKinds::Character;
        }
    }
    if name.contains("county_modifier") {
        return ModifKinds::County;
    }
    if name.contains("province_modifier") {
        return ModifKinds::Province;
    }
    if name.contains("scheme_modifier") {
        return ModifKinds::Scheme;
    }
    if name.contains("travel_plan_modifier") {
        return ModifKinds::TravelPlan;
    }

    ModifKinds::empty()
}
