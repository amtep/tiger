use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, Severity, report};
use crate::token::Token;
use crate::validator::Validator;

const MODIFIER_CATEGORIES: &[&str] = &[
    "all",
    "none",
    "country",
    "location",
    "province",
    "character",
    "unit",
    "mercenary",
    "religion",
    "internationalorganization",
    "rebel",
];

const MODIFIER_FORMATS: &[&str] = &["FormatPopCaps", "FormatManPower", "FormatGold"];

const MODIFIER_BIAS: &[&str] = &["opinion", "trust", "voting"];

#[derive(Clone, Debug)]
pub struct ModifierTypeDefinition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Eu5, Item::ModifierTypeDefinition, ModifierTypeDefinition::add)
}

impl ModifierTypeDefinition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ModifierTypeDefinition, key, block, Box::new(Self {}));
    }
}

impl DbKind for ModifierTypeDefinition {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // Modifier type definitions must be lowercase
        if !key.is_lowercase() {
            report(ErrorKey::DefinitionName, Severity::Error)
                .msg("modifier type definition names must be lowercase")
                .loc(key)
                .push();
        }

        let loca_name = format!("MODIFIER_TYPE_NAME_{key}");
        data.verify_exists_implied(Item::Localization, &loca_name, key);
        let loca_desc = format!("MODIFIER_TYPE_DESC_{key}");
        data.verify_exists_implied(Item::Localization, &loca_desc, key);

        vd.field_integer("decimals");
        vd.field_choice("color", &["neutral", "good", "bad"]);
        vd.field_bool("percent");
        vd.field_bool("boolean");
        vd.field_bool("already_percent");
        vd.field_validated_block("game_data", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_choice("category", MODIFIER_CATEGORIES);
            vd.field_choice("format", MODIFIER_FORMATS);
            vd.multi_field_choice("bias_type", MODIFIER_BIAS);

            vd.field_numeric("min");
            vd.field_numeric("max");

            vd.field_bool("cap_zero_to_one");
            vd.field_bool("scale_with_pop");
            vd.field_bool("ai");
            vd.field_bool("should_show_in_modifiers_tab");
            vd.field_bool("is_societal_value_change");
        });
    }
}
