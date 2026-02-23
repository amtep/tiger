use crate::block::Block;
use crate::ck3::validate::{
    validate_random_culture, validate_random_faith, validate_random_traits_list,
};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_target_ok_this;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CharacterTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CharacterTemplate, CharacterTemplate::add)
}

impl CharacterTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CharacterTemplate, key, block, Box::new(Self {}));
    }
}

impl DbKind for CharacterTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut sc = ScopeContext::new_unrooted(Scopes::all(), key);
        sc.set_strict_scopes(false);
        let b = Block::new(key.loc);
        self.validate_call(key, block, key, &b, data, &mut sc);
    }

    // TODO: lots of duplication between this and "create_character" effect
    fn validate_call(
        &self,
        _key: &Token,
        block: &Block,
        _from: &Token,
        from_block: &Block,
        data: &Everything,
        sc: &mut ScopeContext,
    ) {
        // `from_block` is used to suppress warnings about targets that won't be used
        let mut vd = Validator::new(block, data);
        if from_block.has_key("name") {
            vd.field_value("name");
        } else {
            vd.field_item("name", Item::Localization);
        }
        for field in &[
            "age",
            "health",
            "diplomacy",
            "intrigue",
            "learning",
            "martial",
            "prowess",
            "stewardship",
        ] {
            if from_block.has_key(field) {
                vd.field(field);
            } else {
                vd.field_script_value(field, sc);
            }
        }
        if from_block.has_key("gender") {
            vd.field_value("gender");
            vd.field("gender_female_chance");
        } else {
            if let Some(token) = vd.field_value("gender") {
                if !token.is("male") && !token.is("female") {
                    validate_target_ok_this(token, data, sc, Scopes::Character);
                }
            }
            if from_block.has_key("gender_female_chance") {
                vd.field("gender_female_chance");
            } else {
                vd.field_script_value("gender_female_chance", sc);
            }
        }
        vd.multi_field_item("trait", Item::Trait);
        vd.multi_field_validated_block_sc("random_traits_list", sc, validate_random_traits_list);
        vd.field_bool("random_traits");
        if from_block.has_key("culture") {
            vd.field_value("culture");
            vd.multi_field_block("random_culture");
        } else {
            vd.field_target("culture", sc, Scopes::Culture);
            vd.multi_field_validated_block_sc("random_culture", sc, validate_random_culture);
        }
        if from_block.has_key("faith") {
            vd.field_value("faith");
            vd.multi_field_block("random_faith");
        } else {
            vd.field_target("faith", sc, Scopes::Faith);
            vd.multi_field_validated_block_sc("random_faith", sc, validate_random_faith);
        }
        if from_block.has_key("dynasty_house") {
            vd.field_value("dynasty_house");
        } else {
            vd.field_target("dynasty_house", sc, Scopes::DynastyHouse);
        }
        if from_block.has_key("dynasty") {
            vd.field_value("dynasty");
        } else {
            vd.field_choice("dynasty", &["generate", "inherit", "none"]);
        }
        if from_block.has_key("after_creation") {
            vd.field_block("after_creation");
        } else {
            vd.field_validated_key_block("after_creation", |key, block, data| {
                sc.open_scope(Scopes::Character, key.clone());
                validate_effect(block, data, sc, Tooltipped::No);
                sc.close();
            });
        }
    }
}
