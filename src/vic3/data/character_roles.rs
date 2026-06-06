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
use crate::vic3::tables::misc::CHARACTER_ARCHETYPES;

#[derive(Clone, Debug)]
pub struct CharacterRole {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::CharacterRole, CharacterRole::add)
}

impl CharacterRole {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterRole, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterRole {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_item("texture", Item::File);
        vd.field_choice("type", CHARACTER_ARCHETYPES);
        vd.field_numeric("priority");
        vd.field_bool("auto_assigned");
        vd.field_list_numeric_exactly("career_length", 2);
        vd.field_effect_rooted("on_career_end", Tooltipped::No, Scopes::Character);
        vd.field_bool("should_use_title_in_name");
        vd.field_choice("title_format", &["from_name", "ruler", "heir", "custom"]);
        if block.field_value_is("title_format", "custom") {
            let loca = format!("{key}_custom_loc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        vd.field_item("title_preposition", Item::Localization);
        vd.field_bool("spawn_characters_to_pool");
        vd.field_script_value_no_breakdown_rooted(
            "pool_spawn_target_powerful",
            Scopes::InterestGroup,
        );
        vd.field_script_value_no_breakdown_rooted(
            "pool_spawn_target_influential",
            Scopes::InterestGroup,
        );
        vd.field_script_value_no_breakdown_rooted(
            "pool_spawn_target_marginal",
            Scopes::InterestGroup,
        );
        // TODO: this one is incompatible with the three above
        vd.field_script_value_no_breakdown_rooted(
            "pool_spawn_target_country_wide",
            Scopes::Country,
        );
        vd.field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });
        vd.field_validated_block("character_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        // undocumented
        vd.field_validated_block("holding_scores", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.unknown_block_fields(|_, block| {
                let mut vd = Validator::new(block, data);
                vd.field_item("type", Item::BuildingType);
                vd.field_script_value_no_breakdown_builder("score", |key| {
                    let mut sc = ScopeContext::new(Scopes::Country, key);
                    sc.define_name("is_monarch", Scopes::Bool, key);
                    sc.define_name("interest_group", Scopes::InterestGroup, key);
                    sc.define_name("state", Scopes::State, key);
                    sc
                });
            });
        });
    }
}
