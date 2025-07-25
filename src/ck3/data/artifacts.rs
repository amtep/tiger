use crate::block::{Block, BV};
use crate::ck3::modif::ModifKinds;
use crate::ck3::tables::misc::{ARTIFACT_RARITIES, SUPPORT_TYPES};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::modif::{validate_modifs, verify_modif_exists};
use crate::report::{warn, ErrorKey, Severity};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ArtifactSlot {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactSlot, ArtifactSlot::add)
}

impl ArtifactSlot {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactSlot, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactSlot {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(slot_type) = block.get_field_value("type") {
            db.add_flag(Item::ArtifactSlotType, slot_type.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        data.verify_exists(Item::Localization, key);
        vd.field_item("type", Item::ArtifactSlotType);
        vd.field_choice("category", &["inventory", "court"]);
        if let Some(category) = block.get_field_value("category") {
            // TODO: this can probably be simplified
            if category.is("inventory") {
                let icon = vd.field_value("icon").unwrap_or(key);
                data.verify_icon("NGameIcons|INVENTORY_SLOT_ICON_PATH", icon, ".dds");
            } else if let Some(icon) = vd.field_value("icon") {
                data.verify_icon("NGameIcons|INVENTORY_SLOT_ICON_PATH", icon, ".dds");
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactType {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactType, ArtifactType::add)
}

impl ArtifactType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactType, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let loca = format!("artifact_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("slot", Item::ArtifactSlotType);
        vd.field_list_items("required_features", Item::ArtifactFeatureGroup);
        vd.field_list_items("optional_features", Item::ArtifactFeatureGroup);
        vd.field_bool("can_reforge");
        vd.field_item("default_visuals", Item::ArtifactVisual);
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactTemplate {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactTemplate, ArtifactTemplate::add)
}

impl ArtifactTemplate {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactTemplate, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactTemplate {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("artifact", Scopes::Artifact, key);

        vd.field_trigger("can_equip", Tooltipped::Yes, &mut sc);
        vd.field_trigger("can_benefit", Tooltipped::Yes, &mut sc);
        vd.field_trigger("can_reforge", Tooltipped::Yes, &mut sc);
        vd.field_trigger("can_repair", Tooltipped::Yes, &mut sc);

        vd.field_validated_block("fallback", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_script_value("ai_score", &mut sc);
        vd.field_bool("unique");
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactVisual {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactVisual, ArtifactVisual::add)
}

impl ArtifactVisual {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactVisual, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactVisual {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("artifact", Scopes::Artifact, key);

        vd.field_value("default_type"); // unused

        // These two are undocumented
        vd.field_choice("pedestal", SUPPORT_TYPES);
        vd.field_choice("support_type", SUPPORT_TYPES);

        let mut unconditional = false;
        vd.multi_field_validated("icon", |bv, data| match bv {
            BV::Value(icon) => {
                unconditional = true;
                data.verify_icon("NGameIcons|ARTIFACT_ICON_PATH", icon, "");
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                if !block.has_key("trigger") {
                    unconditional = true;
                }
                vd.field_trigger("trigger", Tooltipped::No, &mut sc);
                vd.field_icon("reference", "NGameIcons|ARTIFACT_ICON_PATH", "");
            }
        });
        if !unconditional {
            let msg = "needs one icon without a trigger";
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
        }

        unconditional = false;
        vd.multi_field_validated("asset", |bv, data| match bv {
            BV::Value(asset) => {
                unconditional = true;
                data.verify_exists(Item::Asset, asset);
            }
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                if !block.has_key("trigger") {
                    unconditional = true;
                }
                vd.field_trigger("trigger", Tooltipped::No, &mut sc);
                vd.field_validated_value("reference", |_, mut vd| {
                    vd.item(Item::Asset);
                });
            }
        });
        if !unconditional {
            let msg = "needs at least one asset without a trigger";
            warn(ErrorKey::Validation).msg(msg).loc(key).push();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactFeature {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactFeature, ArtifactFeature::add)
}

impl ArtifactFeature {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactFeature, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactFeature {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // TODO: it's not clear what the scope is for these triggers
        let mut sc = ScopeContext::new_unrooted(Scopes::Artifact | Scopes::Character, key);
        sc.define_name("newly_created_artifact", Scopes::Artifact, key);
        sc.define_name("owner", Scopes::Character, key);
        sc.define_name("wealth", Scopes::Value, key);

        let loca = format!("feature_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_item("group", Item::ArtifactFeatureGroup);
        vd.field_script_value("weight", &mut sc);

        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactFeatureGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactFeatureGroup, ArtifactFeatureGroup::add)
}

impl ArtifactFeatureGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactFeatureGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactFeatureGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut _vd = Validator::new(block, data);
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactBlueprint {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ArtifactBlueprint, ArtifactBlueprint::add)
}

impl ArtifactBlueprint {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ArtifactBlueprint, key, block, Box::new(Self {}));
    }
}

impl DbKind for ArtifactBlueprint {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("in_type");
        vd.req_field("in_visuals");
        vd.field_item("in_type", Item::ArtifactType);
        vd.field_item("in_visuals", Item::ArtifactVisual);

        vd.req_field("out_type");
        vd.req_field("out_visuals");
        vd.field_item("out_type", Item::ArtifactType);
        vd.field_item("out_visuals", Item::ArtifactVisual);

        vd.field_validated_list("disallowed_modifiers", |token, data| {
            verify_modif_exists(token, data, ModifKinds::Character, Severity::Warning);
        });
        vd.field_validated_block("replacement_modifiers", |block, data| {
            let mut vd = Validator::new(block, data);
            for field in ARTIFACT_RARITIES {
                vd.field_validated_list(field, |token, data| {
                    data.verify_exists(Item::Modifier, token);
                    // Verify that all the modifs in this modifier are artifact-compatible.
                    data.database.validate_property_use(
                        Item::Modifier,
                        token,
                        data,
                        token,
                        "artifact_modifier",
                    );
                });
            }
        });

        vd.field_item("template", Item::ArtifactTemplate);
    }
}
