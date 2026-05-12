use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;
use crate::vic3::modif::ModifKinds;

#[derive(Clone, Debug)]
pub struct NavalMissionType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::NavalMissionType, NavalMissionType::add)
}

impl NavalMissionType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::NavalMissionType, key, block, Box::new(Self {}));
    }
}

impl DbKind for NavalMissionType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_bool("intercept");
        vd.field_bool("project_power");
        vd.field_bool("blockade");
        vd.field_bool("raid_supply");
        vd.field_bool("protect_supply");
        vd.field_bool("piracy");
        vd.field_bool("port_bombardment");

        vd.field_numeric("experience");

        vd.field_list_choice("intercept_targets", &["none", "hostile", "piracy", "all"]);
        vd.field_list_choice(
            "piracy_targets",
            &["none", "hostile", "neutral", "allies", "non_allies", "all"],
        );

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::MilitaryFormation, vd);
        });
        vd.field_validated_block("command_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_trigger_rooted("potential", Tooltipped::No, Scopes::MilitaryFormation);
        vd.field_trigger_rooted("possible", Tooltipped::Yes, Scopes::MilitaryFormation);

        vd.field_trigger_rooted("ai_will_do", Tooltipped::No, Scopes::MilitaryFormation);
        vd.field_script_value_no_breakdown_rooted("ai_weight", Scopes::MilitaryFormation);

        // undocumented

        vd.field_item("clicksound", Item::Sound);
        vd.field_bool("ai_will_cancel_for_transportation");
        vd.field_numeric("ai_distance_penalty_multiplier");
    }
}
