use crate::block::Block;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct PuppetType {}
#[derive(Clone, Debug)]
pub struct PuppetAction {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::PuppetType, PuppetType::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::PuppetAction, PuppetAction::add)
}

impl PuppetType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PuppetType, key, block, Box::new(Self {}));
    }
}
impl PuppetAction {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::PuppetAction, key, block, Box::new(Self {}));
    }
}

impl DbKind for PuppetType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("puppet", Scopes::Character, key);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("interaction", Item::CharacterInteraction);
        vd.field_trigger_rooted("can_have", Tooltipped::No, Scopes::Character);
        vd.field_trigger("is_valid", Tooltipped::No, &mut sc);
        vd.field_script_value_rooted("priority", Scopes::Character);
        vd.field_list_items("actions", Item::PuppetAction);

        // The docs say these need a fallback entry without a trigger
        for field in ["asset", "animation"] {
            let blocks = block.get_field_blocks(field);
            if blocks.iter().all(|block| block.has_key("trigger")) {
                let msg = format!("`{field}` needs a fallback entry without `trigger`");
                err(ErrorKey::FieldMissing).msg(msg).loc(key).push();
            }
        }
        vd.multi_field_validated_block("asset", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger("trigger", Tooltipped::No, &mut sc.clone());
            vd.field_item("icon", Item::File);
            vd.field_item("background", Item::File);
        });
        vd.multi_field_validated_block("animation", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger("trigger", Tooltipped::No, &mut sc.clone());
            vd.req_field("reference");
            vd.field_item("reference", Item::PortraitAnimation);
        });
        vd.field_validated_block("color", validate_color);
    }
}

impl DbKind for PuppetAction {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("puppet", Scopes::Character, key);

        vd.field_choice("type", &["character_interaction", "decision", "great_project", "trigger"]);
        let action_type =
            block.get_field_value("type").map_or("character_interaction", Token::as_str);

        if action_type == "character_interaction" || action_type == "trigger" {
            let loca = format!("puppet_action_description_{key}");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if action_type == "character_interaction" {
            vd.req_field("interaction");
            vd.field_item("interaction", Item::CharacterInteraction);
            vd.field_bool("is_important");
        } else {
            vd.ban_field("interaction", || "type = character_interaction");
            vd.ban_field("is_important", || "type = character_interaction");
        }
        if action_type == "decision" {
            vd.req_field("decision");
            vd.field_item("decision", Item::Decision);
        } else {
            vd.ban_field("decision", || "type = decision");
        }
        if action_type == "great_project" {
            vd.req_field("great_project");
            vd.field_item("great_project", Item::GreatProjectType);
        } else {
            vd.ban_field("great_project", || "type = great_project");
        }

        if action_type == "trigger" {
            vd.req_field("is_enabled");
        }
        vd.field_trigger("is_enabled", Tooltipped::No, &mut sc);
    }
}
