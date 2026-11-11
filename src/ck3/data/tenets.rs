use crate::block::{BV, Block};
use crate::ck3::data::doctrines::{
    is_unreformed, validate_special_parameters, verify_parameter_loca,
};
use crate::ck3::modif::ModifKinds;
use crate::ck3::validate::{validate_parameter_value, validate_traits};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Tenet {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Tenet, Tenet::add)
}

impl Tenet {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Tenet, key, block, Box::new(Self {}));
    }
}

impl DbKind for Tenet {
    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        property == "unreformed" && is_unreformed(block)
    }

    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        // Tenet parameters are plain flags that can be checked like doctrine parameters
        if let Some(block) = block.get_field_block("parameters") {
            for token in block.iter_values() {
                db.add_flag(Item::DoctrineParameter, token.clone());
            }
            for (key, _) in block.iter_assignments() {
                db.add_flag(Item::DoctrineParameter, key.clone());
            }
        }
        if let Some(block) = block.get_field_block("personal_tenet_parameters") {
            for token in block.iter_values() {
                db.add_flag(Item::PersonalTenetParameter, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // `selected_doctrines` holds the selected core tenets and doctrines
        let selected = Scopes::Doctrine | Scopes::Tenet;
        let mut faith_sc = ScopeContext::new(Scopes::Faith, key);
        let mut can_pick_sc = ScopeContext::new(Scopes::Faith, key);
        can_pick_sc.define_name("actor", Scopes::Character, key);
        can_pick_sc.define_list("selected_doctrines", selected, key);
        let mut rite_sc = ScopeContext::new(Scopes::Rite, key);
        let mut cost_sc = ScopeContext::new(Scopes::Rite, key);
        // TODO: verify. Undocumented for piety_cost, but one vanilla tenet uses it there.
        cost_sc.define_list("selected_doctrines", selected, key);

        // Documented as a triggered list, but vanilla uses plain icon names
        if !vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(token) => data.verify_icon("NGameIcons|TENET_TYPE_ICON_PATH", token, ".dds"),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Faith);
                vd.req_field("reference");
                vd.field_icon("reference", "NGameIcons|TENET_TYPE_ICON_PATH", ".dds");
            }
        }) {
            data.verify_icon("NGameIcons|TENET_TYPE_ICON_PATH", key, ".dds");
        }

        if !vd.field_validated_sc("name", &mut rite_sc, validate_desc) {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        if !vd.field_validated_sc("desc", &mut rite_sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        vd.field_validated_sc("historical_glossary", &mut rite_sc, validate_desc);

        vd.field_bool("visible");
        vd.field_item("requires_dlc_flag", Item::DlcFeature);
        vd.field_validated_block("parameters", |block, data| {
            let mut vd = Validator::new(block, data);
            for name in vd.values() {
                verify_parameter_loca(name, data);
            }
            // TODO: verify. Vanilla uses `name = yes` only in doctrine parameters.
            vd.unknown_value_fields(|key, value| {
                verify_parameter_loca(key, data);
                validate_parameter_value(key, value);
            });
        });
        vd.field_script_value("piety_cost", &mut cost_sc);
        vd.field_trigger("is_shown", Tooltipped::No, &mut faith_sc);
        vd.field_trigger("can_pick", Tooltipped::Yes, &mut can_pick_sc);
        // TODO: verify tooltip
        vd.field_trigger_rooted("can_pick_as_personal_tenet", Tooltipped::Yes, Scopes::Character);
        vd.field_numeric("divergence_multiplier");

        vd.field_validated_block("character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization); // undocumented
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("clergy_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("county_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::County, vd);
        });
        vd.field_validated_block("traits", validate_traits);
        vd.field_validated_block("special_parameters", validate_special_parameters);
        vd.field_validated_block("personal_tenet_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_list("personal_tenet_parameters", |flag, data| {
            // The docs say they are localized with the doctrine parameter prefix
            let loca = format!("doctrine_parameter_{flag}");
            data.verify_exists_implied(Item::Localization, &loca, flag);
        });
    }
}
