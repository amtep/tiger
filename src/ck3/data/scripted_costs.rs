use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;

#[derive(Clone, Debug)]
pub struct ScriptedCost {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ScriptedCost, ScriptedCost::add)
}

impl ScriptedCost {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedCost, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedCost {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        match key.as_str() {
            // The docs give these only the character root
            "diverge_culture" | "adopt_court_language" | "enact_treasury_budget" => (),
            "hybridize_culture" => {
                sc.define_name("culture", Scopes::Culture, key);
            }
            "reforge_artifact" | "repair_artifact" => {
                sc.define_name("artifact", Scopes::Artifact, key);
                sc.define_name("owner", Scopes::Character, key);
                sc.define_name("quality", Scopes::Value, key);
                sc.define_name("wealth", Scopes::Value, key);
            }
            "travel_leader" => {
                sc.define_name("speed_aptitude", Scopes::Value, key);
                sc.define_name("safety_aptitude", Scopes::Value, key);
            }
            "create_accolade" => {
                sc.define_name("owner", Scopes::Character, key);
            }
            "reassign_title_troops" => {
                sc = ScopeContext::new(Scopes::LandedTitle, key);
                sc.define_name("actor", Scopes::Character, key);
            }
            "reform_culture_ethos" | "reform_culture_language" | "reform_culture_martial" => {
                sc = ScopeContext::new(Scopes::Culture, key);
                sc.define_name("character", Scopes::Character, key);
            }
            "create_holy_site" => {
                sc.define_name("title", Scopes::LandedTitle, key);
                sc.define_name("holy_site_type", Scopes::HolySiteType, key);
                sc.define_name("faith", Scopes::Faith, key);
                sc.define_list("artifacts", Scopes::Artifact, key);
            }
            _ => {
                // The docs say the game only looks up the named costs
                let msg = format!("the game doesn't use a scripted cost named `{key}`");
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
                sc.set_strict_scopes(false);
            }
        }

        validate_cost(block, data, &mut sc);
    }
}
