use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::imperator::modif::ModifKinds;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct LawGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Imperator, Item::LawGroup, LawGroup::add)
}

impl LawGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::LawGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for LawGroup {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for (key, block) in block.iter_definitions() {
            if !key.is("potential") {
                db.add(Item::Law, key.clone(), block.clone(), Box::new(Law {}));
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);

        vd.field_trigger("potential", Tooltipped::No, &mut sc);

        // The laws. They are validated in the Law class.
        vd.unknown_block_fields(|_, _| ());
    }
}

#[derive(Clone, Debug)]
pub struct Law {}

impl DbKind for Law {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Country, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_choice(
            "succession",
            &[
                "elective_monarchy",
                "old_egyptian_succession",
                "agnatic",
                "cognatic",
                "agnatic_seniority",
            ],
        );

        vd.field_trigger("allow", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_enact", Tooltipped::Yes, &mut sc);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });
    }
}
