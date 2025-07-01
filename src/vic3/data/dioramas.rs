use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::vic3::validate::validate_locators;

#[derive(Clone, Debug)]
pub struct ArmyDiorama {}
#[derive(Clone, Debug)]
pub struct FleetDiorama {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ArmyDiorama, ArmyDiorama::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::FleetDiorama, FleetDiorama::add)
}

impl ArmyDiorama {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArmyDiorama, key, block, Box::new(Self {}));
    }
}
impl FleetDiorama {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FleetDiorama, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArmyDiorama {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("battle_side", &["none", "attacker", "defender", "either"]);
        vd.field_bool("basecamp");
        validate_diorama(vd, true);
    }
}

impl DbKind for FleetDiorama {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_choice("group", &["fleet", "battle_side", "blockade"]);
        validate_diorama(vd, false);
    }
}

fn validate_diorama(mut vd: Validator, is_army: bool) {
    vd.field_validated_block("unit_composition", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.validate_item_key_values(Item::CombatUnitGroup, |_, mut vvd| {
            vvd.integer();
        });
    });

    vd.field_list_items("required_unit_types", Item::CombatUnit);

    vd.field_numeric("distance");

    let locator_names = validate_locators(&mut vd);
    vd.multi_field_validated_block("attach", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_field("locator");
        vd.field_choice("locator", &locator_names);
        vd.field_item("entity", Item::Entity);
        vd.field_item("combat_unit_group", Item::CombatUnitGroup);
        vd.field_value("entity_group"); // TODO
        vd.field_bool("can_be_defeated");
        if is_army {
            vd.field_bool("general");
            vd.field_bool("ignore_terrain");
        }
        vd.field_trigger_builder("is_visible", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Country, key);
            sc.define_name("num_units", Scopes::Value, key);
            sc
        });
    });
}
