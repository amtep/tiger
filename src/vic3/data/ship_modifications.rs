use crate::block::Block;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;
use crate::vic3::modif::ModifKinds;

#[derive(Clone, Debug)]
pub struct ShipModification {}
#[derive(Clone, Debug)]
pub struct ShipModificationSlot {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ShipModification, ShipModification::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::ShipModificationSlot, ShipModificationSlot::add)
}

impl ShipModification {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipModification, key, block, Box::new(Self {}));
    }
}
impl ShipModificationSlot {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ShipModificationSlot, key, block, Box::new(Self {}));
    }
}

impl DbKind for ShipModification {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Ship, vd);
        });

        vd.field_item("type", Item::ShipModificationSlot);
        vd.field_validated_block("construction_goods", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Goods, vd);
        });
        vd.field_validated_block("materiel_goods", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Goods, vd);
        });
        vd.field_list_items("unlocking_technologies", Item::Technology);
        vd.multi_field_item("incompatible_with", Item::ShipModification);

        vd.field_script_value_no_breakdown_rooted("ai_weight", Scopes::Country);

        // undocumented

        vd.field_item("icon", Item::File);
        vd.field_item("clicksound", Item::Sound);
    }
}

impl DbKind for ShipModificationSlot {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_botton_0");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_botton_1");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_botton_2");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_bool("utility");
    }
}
