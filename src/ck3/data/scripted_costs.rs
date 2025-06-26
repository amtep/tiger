use crate::block::Block;
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
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
            "hybridize_culture" => {
                sc.define_name("culture", Scopes::Culture, key);
            }
            "reforge_artifact" | "repair_artifact" => {
                sc = ScopeContext::new(Scopes::None, key);
                sc.define_name("artifact", Scopes::Artifact, key);
            }
            "travel_leader" => {
                sc.define_name("speed_aptitude", Scopes::Value, key);
                sc.define_name("safety_aptitude", Scopes::Value, key);
            }
            "deactivate_accolade" => {
                sc = ScopeContext::new(Scopes::Accolade, key);
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
            }
            _ => {
                sc.set_strict_scopes(false);
            }
        }

        validate_cost(block, data, &mut sc);
    }
}
