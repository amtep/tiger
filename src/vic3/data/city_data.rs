use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, Severity, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::{Validator, ValueValidator};
use crate::vic3::validate::validate_locators;

#[derive(Clone, Debug)]
pub struct CityBuildingVfx {}
#[derive(Clone, Debug)]
pub struct CityCenterpiece {}
#[derive(Clone, Debug)]
pub struct CityGraphicsType {}
#[derive(Clone, Debug)]
pub struct CityVfx {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CityBuildingVfx, CityBuildingVfx::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CityCenterpiece, CityCenterpiece::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CityGraphicsType, CityGraphicsType::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CityVfx, CityVfx::add)
}

impl CityBuildingVfx {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CityBuildingVfx, key, block, Box::new(Self {}));
    }
}
impl CityCenterpiece {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CityCenterpiece, key, block, Box::new(Self {}));
    }
}
impl CityGraphicsType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CityGraphicsType, key, block, Box::new(Self {}));
    }
}
impl CityVfx {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CityVfx, key, block, Box::new(Self {}));
    }
}

impl DbKind for CityBuildingVfx {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);

        if let Some(particle) = vd.field_value("particle") {
            let pathname = format!("gfx/particles/{particle}.particle2");
            data.verify_exists_implied(Item::File, &pathname, particle);
            let pathname = format!("gfx/particles/{particle}.editordata");
            data.verify_exists_implied(Item::File, &pathname, particle);
        }

        vd.field_numeric("max_visible");
        vd.field_numeric("max_distance");
    }
}

impl DbKind for CityCenterpiece {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);

        vd.req_field_one_of(&["city_type", "city_graphics_type"]);
        vd.field_choice("city_type", &["none", "city", "farm", "mine", "port", "wood"]);
        vd.multi_field_item("city_graphics_type", Item::CityGraphicsType);
        vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::StateRegion);
        vd.field_integer("priority");
        vd.field_bool("should_update_on_pm_change");

        vd.field_list_integers_exactly("grid_size", 2); // undocumented

        let locator_names = validate_locators(&mut vd);

        let mut composition_names = Vec::new();
        vd.field_validated_block("composition_group", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_list_items("building_types", Item::BuildingType);
            let n = block.get_field_list("building_types").map_or(0, |v| v.len());
            vd.field_validated_list("levels", |token, _| {
                token.expect_integer();
            });
            #[allow(clippy::cast_possible_wrap)]
            let m = block.get_field_list("levels").map_or(0, |v| v.len()) as i64;
            vd.multi_field_validated_block("composition", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_field("name");
                if let Some(name) = vd.field_value("name") {
                    if let Some(other) = composition_names.iter().find(|n| n == &name) {
                        warn(ErrorKey::DuplicateField)
                            .msg(format!("duplicate composition name `{name}`"))
                            .loc(name)
                            .loc_msg(other, "previous composition")
                            .push();
                    } else {
                        composition_names.push(name.clone());
                    }
                }
                vd.req_field_one_of(&["levels", "ratios", "trigger"]);

                vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::State);

                let mut count = 0;
                vd.field_validated_list("levels", |token, data| {
                    let mut vvd = ValueValidator::new(token, data);
                    vvd.set_max_severity(Severity::Warning);
                    vvd.integer_range(0..=m);
                    count += 1;
                });
                if let Some(levels) = block.get_key("levels") {
                    if count != n {
                        let msg = "length of `levels` list should be the same as `building_types`";
                        warn(ErrorKey::Validation).msg(msg).loc(levels).push();
                    }
                }

                let mut count = 0;
                vd.field_validated_list("ratios", |token, _| {
                    token.expect_integer();
                    count += 1;
                });
                if let Some(ratios) = block.get_key("ratios") {
                    if count != n {
                        let msg = "length of `ratios` list should be the same as `building_types`";
                        warn(ErrorKey::Validation).msg(msg).loc(ratios).push();
                    }
                }
            });
        });

        let composition_names: Vec<_> = composition_names.into_iter().map(|n| n.as_str()).collect();
        vd.multi_field_validated_block("attach", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.set_max_severity(Severity::Warning);
            vd.field_choice("locator", &locator_names);

            vd.multi_field_validated_block("variant", |block, data| {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);
                vd.req_field_one_of(&["entity", "is_power_bloc_statue"]);
                vd.field_item("entity", Item::Entity);
                vd.field_bool("is_power_bloc_statue");
                vd.field_item("building_type", Item::BuildingType);
                vd.field_list_choice("compositions", &composition_names);
                vd.field_block("attach"); // TODO what is attach_body
            });
        });
    }
}

impl DbKind for CityGraphicsType {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);

        vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::StateRegion);
        vd.field_numeric("weight");

        vd.field_value("graphical_culture");
        vd.field_choice("city_type", &["none", "city", "farm", "mine", "port", "wood"]);

        vd.field_integer("min_residential_buildings");
        vd.field_integer("max_residential_buildings");
        vd.field_integer("max_residence_points");

        for field in &["rich_building_meshes", "mid_building_meshes", "poor_building_meshes"] {
            vd.field_list_items(field, Item::Pdxmesh);
        }

        for field in &[
            "rich_building_offsets",
            "mid_building_offsets",
            "poor_building_offsets",
            "building_offsets",
        ] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.set_max_severity(Severity::Warning);

                vd.field_numeric("position");
                vd.field_numeric("rotation");
            });
        }

        vd.validate_item_key_blocks(Item::BuildingType, |_, block, data| {
            let mut vd = Validator::new(block, data);
            vd.set_max_severity(Severity::Warning);

            for value in vd.values() {
                data.verify_exists_max_sev(Item::Pdxmesh, value, Severity::Warning);
            }
        });
    }
}

impl DbKind for CityVfx {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.set_max_severity(Severity::Warning);
        vd.field_trigger_rooted("visible", Tooltipped::No, Scopes::State);
        vd.field_item("entity", Item::Entity);
    }
}
