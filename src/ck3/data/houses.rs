use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct House {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::House, House::add)
}

impl House {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::House, key, block, Box::new(Self {}));
    }

    pub fn get_dynasty<'a>(key: &str, data: &'a Everything) -> Option<&'a Token> {
        data.database
            .get_key_block(Item::House, key)
            .and_then(|(_, block)| block.get_field_value("dynasty"))
    }
}

impl DbKind for House {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("name");
        vd.req_field("dynasty");

        vd.field_item("name", Item::Localization);
        vd.field_item("prefix", Item::Localization);
        vd.field_item("motto", Item::Localization);
        vd.field_item("dynasty", Item::Dynasty);
        vd.field_value("forced_coa_religiongroup"); // TODO
    }
}

#[derive(Clone, Debug)]
pub struct HouseAspiration {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::HouseAspiration, HouseAspiration::add)
}

impl HouseAspiration {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::HouseAspiration, key, block, Box::new(Self {}));
    }
}

impl DbKind for HouseAspiration {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for block in block.get_field_blocks("level") {
            if let Some(block) = block.get_field_block("parameters") {
                for (key, value) in block.iter_assignments() {
                    if value.lowercase_is("yes") || value.lowercase_is("no") {
                        db.add_flag(Item::BooleanHousePowerParameter, key.clone());
                    }
                }
            }
            if let Some(block) = block.get_field_block("house_head_parameters") {
                for (key, value) in block.iter_assignments() {
                    if value.lowercase_is("yes") || value.lowercase_is("no") {
                        db.add_flag(Item::BooleanHouseHeadParameter, key.clone());
                    }
                }
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_house_power");
        data.verify_exists_implied(Item::Localization, &loca, key);

        data.verify_icon("NGameIcons|HOUSE_POWER_BONUS_ICON_PATH", key, ".dds");

        vd.field_bool("show_in_main_hud");
        vd.field_trigger_rooted("is_shown", Tooltipped::No, Scopes::DynastyHouse);

        vd.field_bool("is_default");

        vd.req_field("level");
        vd.multi_field_validated_block("level", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_key_block("cost", |key, block, data| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                validate_cost(block, data, &mut sc);
            });
            for field in &[
                "powerful_family_top_liege_modifier",
                "powerful_family_member_modifier",
                "any_house_member_modifier",
                "house_head_modifier",
            ] {
                vd.field_validated_block(field, |block, data| {
                    let vd = Validator::new(block, data);
                    validate_modifs(block, data, ModifKinds::Character, vd);
                });
            }

            vd.field_script_value_no_breakdown_rooted("ai_score", Scopes::Character);
            for field in &["parameters", "house_head_parameters"] {
                vd.field_validated_block(field, |block, data| {
                    let mut vd = Validator::new(block, data);
                    vd.unknown_value_fields(|key, value| {
                        let loca = format!("house_power_parameter_{key}");
                        data.verify_exists_implied(Item::Localization, &loca, key);

                        let mut vvd = ValueValidator::new(value, data);
                        vvd.bool();
                    });
                });
            }

            vd.field_bool("can_request_great_project_contributions_from_allies");
            vd.field_trigger_rooted("can_upgrade", Tooltipped::Yes, Scopes::Character);
        });

        vd.field_item("illustration", Item::File);

        // TODO: figure out if this takes script values, and if so, what's the scope context.
        vd.field_validated_block("cooldown", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("days");
            vd.field_integer("weeks");
            vd.field_integer("months");
            vd.field_integer("years");
        });

        for field in &["on_changed", "on_upgraded"] {
            vd.field_effect_builder(field, Tooltipped::Yes, |key| {
                let mut sc = ScopeContext::new(Scopes::Character, key);
                sc.define_name("house", Scopes::DynastyHouse, key);
                sc
            });
        }
    }
}
