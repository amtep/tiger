use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::hoi4::modif::ModifKinds;
use crate::hoi4::validate::validate_rules;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{err, ErrorKey};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct IdeologyGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::IdeologyGroup, IdeologyGroup::add)
}

impl IdeologyGroup {
    #[allow(clippy::needless_pass_by_value)]
    pub fn add(db: &mut Db, key: Token, mut block: Block) {
        if key.is("ideologies") {
            for (key, block) in block.drain_definitions_warn() {
                db.add(Item::IdeologyGroup, key, block, Box::new(Self {}));
            }
        } else {
            let msg = "unexpected key";
            let info = "expected only `ideologies`";
            err(ErrorKey::UnknownField).msg(msg).info(info).loc(&key).push();
        }
    }
}

impl DbKind for IdeologyGroup {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("types") {
            for (key, _) in block.iter_definitions() {
                db.add_flag(Item::Ideology, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_noun");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let sprite = format!("GFX_ideology_{key}_group");
        data.verify_exists_implied(Item::Sprite, &sprite, key);

        vd.field_validated_block("types", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|key, block| {
                data.verify_exists(Item::Localization, key);
                let loca = format!("{key}_desc");
                data.verify_exists_implied(Item::Localization, &loca, key);

                let sprite = format!("GFX_ideology_{key}");
                data.mark_used(Item::Sprite, &sprite);

                let mut vd = Validator::new(block, data);
                vd.field_bool("can_be_randomly_selected");
                vd.field_validated_block("color", validate_color);
            });
        });

        vd.field_list_items("dynamic_faction_names", Item::Localization);
        vd.field_validated_block("color", validate_color);

        vd.field_validated_block("rules", validate_rules);

        vd.field_bool("can_host_government_in_exile");
        vd.field_numeric("war_impact_on_world_tension");
        vd.field_numeric("faction_impact_on_world_tension");

        for field in &["modifiers", "faction_modifiers"] {
            vd.field_validated_block(field, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Country, vd);
            });
        }

        vd.field_bool("can_be_boosted");
        vd.field_bool("can_collaborate");

        vd.field_bool("ai_democratic");
        vd.field_bool("ai_communist");
        vd.field_bool("ai_fascist");
        vd.field_bool("ai_neutral");

        vd.field_numeric("ai_ideology_wanted_units_factor");
        vd.field_numeric("ai_give_core_state_control_threshold");
    }
}
