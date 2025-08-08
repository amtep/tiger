use crate::block::{BV, Block};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct NationalFocusTree {}
#[derive(Clone, Debug)]
pub struct NationalFocus {}
#[derive(Clone, Debug)]
pub struct NationalFocusStyle {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Hoi4, Item::NationalFocusTree, NationalFocusTree::add)
}

impl NationalFocusTree {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        if key.is("focus_tree") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::NationalFocusTree, id.clone(), block, Box::new(Self {}));
            } else {
                let msg = "focus tree without id";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else if key.is("shared_focus") || key.is("joint_focus") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::NationalFocus, id.clone(), block, Box::new(NationalFocus {}));
            } else {
                let msg = "focus without id";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else if key.is("style") {
            if let Some(name) = block.get_field_value("name") {
                db.add(
                    Item::NationalFocusStyle,
                    name.clone(),
                    block,
                    Box::new(NationalFocusStyle {}),
                );
            } else {
                let msg = "style without name";
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        } else {
            let msg = "unexpected key";
            err(ErrorKey::UnknownField).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for NationalFocusTree {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        for block in block.get_field_blocks("focus") {
            if let Some(id) = block.get_field_value("id") {
                db.add(Item::NationalFocus, id.clone(), block.clone(), Box::new(NationalFocus {}));
            } else {
                let msg = "focus without id";
                err(ErrorKey::FieldMissing).msg(msg).loc(block).push();
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("id");

        let mut sc = ScopeContext::new(Scopes::Country, key);
        vd.field_validated_block_sc("country", &mut sc, validate_modifiers_with_base);

        vd.field_bool("default");

        for field in &["initial_show_position", "continuous_focus_position"] {
            vd.field_validated_block(field, |block, data| {
                let mut vd = Validator::new(block, data);
                vd.field_integer("x");
                vd.field_integer("y");
            });
        }

        vd.multi_field_item("shared_focus", Item::NationalFocus);

        vd.multi_field("focus"); // validated by NationalFocus item
    }
}

impl DbKind for NationalFocus {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("id");
        vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(value) => {
                data.verify_exists(Item::Sprite, value);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                vd.field_item("value", Item::Sprite);
                vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Country);
            }
        });
        vd.field_item("text_icon", Item::NationalFocusStyle);

        vd.field_trigger_rooted("allow_branch", Tooltipped::Yes, Scopes::Country);
        vd.field_validated_block("mutually_exclusive", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_item("focus", Item::NationalFocus);
        });
        vd.multi_field_validated_block("prerequisite", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.multi_field_item("focus", Item::NationalFocus);
        });
        vd.field_integer("x");
        vd.field_integer("y");
        vd.field_item("relative_position_id", Item::NationalFocus);
        vd.multi_field_validated_block("offset", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_integer("x");
            vd.field_integer("y");
            vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Country);
        });
        vd.field_integer("cost");
        vd.field_trigger_rooted("bypass", Tooltipped::Yes, Scopes::Country);
        vd.field_trigger_rooted("available", Tooltipped::Yes, Scopes::Country);
        vd.field_effect_rooted("select_effect", Tooltipped::Yes, Scopes::Country);

        vd.field_bool("cancel_if_invalid");
        vd.field_bool("continue_if_invalid");
        vd.field_bool("available_if_capitulated");

        vd.field_validated_list("search_filters", |value, data| {
            data.verify_exists(Item::Localization, value);
            let sprite = format!("GFX_{value}");
            data.verify_exists_implied(Item::Sprite, &sprite, value);
        });
        vd.field_effect_rooted("complete_tooltip", Tooltipped::Yes, Scopes::Country);
        vd.field_effect_rooted("completion_reward", Tooltipped::Yes, Scopes::Country);

        let mut sc = ScopeContext::new(Scopes::Country, key);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);

        vd.field_bool("dynamic");
        vd.field_item("will_lead_to_war_with", Item::CountryTag);
    }
}

impl DbKind for NationalFocusStyle {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_value("name");
        vd.field_bool("default");
        vd.field_item("unavailable", Item::Sprite);
        vd.field_item("completed", Item::Sprite);
        vd.field_item("available", Item::Sprite);
        vd.field_item("current", Item::Sprite);
    }
}
