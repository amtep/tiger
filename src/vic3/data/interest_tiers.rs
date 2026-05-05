use crate::block::Block;
use crate::context::ScopeContext;
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
pub struct InterestTierType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::InterestTierType, InterestTierType::add)
}

impl InterestTierType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::InterestTierType, key, block, Box::new(Self {}));
    }
}

impl DbKind for InterestTierType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        // TODO: Validation expects contiguous ranges: previous.max == current.min
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("icon", Item::File);
        vd.field_item("small_icon", Item::File);
        vd.field_integer("rank");
        vd.field_numeric("min_involvement");
        vd.field_numeric("max_involvement");

        vd.field_validated_block("state_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::State, vd);
        });

        vd.field_bool("colonization");

        for field in &["on_activate", "on_deactivate"] {
            vd.field_effect_builder(field, Tooltipped::Yes, |key| {
                let mut sc = ScopeContext::new(Scopes::Country, key);
                sc.define_name("target_region", Scopes::StrategicRegion, key);
                sc
            });
        }

        // undocumented

        vd.field_numeric("trade_advantage_exports");
        vd.field_numeric("max_involvement");
    }
}
