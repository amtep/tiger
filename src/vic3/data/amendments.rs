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
pub struct Amendment {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Amendment, Amendment::add)
}

impl Amendment {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Amendment, key, block, Box::new(Self {}));
    }
}

impl DbKind for Amendment {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        data.verify_exists_implied(Item::Localization, &format!("{key}_desc"), key);

        vd.field_item("parent", Item::LawType);

        vd.field_list_items("allowed_laws", Item::LawType);

        vd.field_validated_block("modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_validated_block("sponsor_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::InterestGroup, vd);
        });

        for tax_modifier in &[
            "tax_modifier_very_low",
            "tax_modifier_low",
            "tax_modifier_medium",
            "tax_modifier_high",
            "tax_modifier_very_high",
        ] {
            vd.field_validated_block(tax_modifier, |block, data| {
                let vd = Validator::new(block, data);
                validate_modifs(block, data, ModifKinds::Country, vd);
            });
        }

        vd.field_item("institution", Item::Institution);
        vd.field_validated_block("institution_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Country, vd);
        });

        vd.field_trigger_rooted("possible", Tooltipped::No, Scopes::Country);

        vd.field_trigger_rooted("can_repeal", Tooltipped::Yes, Scopes::Country);

        vd.field_trigger_rooted("would_sponsor", Tooltipped::Yes, Scopes::InterestGroup);

        vd.field_trigger_rooted("ai_will_revoke", Tooltipped::Yes, Scopes::Country);

        vd.field_numeric("amendment_activism_multiplier");
    }
}
