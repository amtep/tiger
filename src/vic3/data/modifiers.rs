use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, ModifKinds as _};
use crate::token::Token;
use crate::validator::Validator;
use crate::vic3::modif::ModifKinds;
use crate::vic3::tables::modifs::modif_scope_kind;

#[derive(Clone, Debug)]
pub struct Modifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::Modifier, Modifier::add)
}

impl Modifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add_exact_dup_ok(Item::Modifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for Modifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // The standard of living defines for cultures are required but don't need localizations
        if !key.as_str().ends_with("_standard_of_living_modifier_positive")
            && !key.as_str().ends_with("standard_of_living_modifier_negative")
        {
            data.verify_exists(Item::Localization, key);
        }
        vd.field_item("icon", Item::File);

        validate_modifs(block, data, ModifKinds::all(), vd);
    }

    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        from: &Token,
        _from_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        let mut vd = Validator::new(block, data);

        // Mark as known field
        vd.field("icon");

        // Ensure contained modifs are valid at this location
        let (scopes, scope_reason) = sc.scopes_reason();
        let scope_kinds = modif_scope_kind(scopes);
        vd.unknown_fields(|key, _| {
            if let Some(kind) = ModifKinds::lookup_modif(key, data, None) {
                scope_kinds.require_from(
                    kind,
                    key,
                    Some((from, scopes, scope_reason)),
                    crate::Severity::Warning,
                );
            }
        });
    }
}
