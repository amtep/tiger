use crate::block::Block;
use crate::ck3::data::doctrines::validate_doctrine_groups;
use crate::ck3::data::religions::{validate_localization, validate_tenet_selection_pair};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct RiteType {}
#[derive(Clone, Debug)]
pub struct RiteName {}
#[derive(Clone, Debug)]
pub struct RiteIcon {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::RiteType, RiteType::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::RiteName, RiteName::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::RiteIcon, RiteIcon::add)
}

impl RiteType {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::RiteType, key, block, Box::new(Self {}));
    }
}
impl RiteName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::RiteName, key, block, Box::new(Self {}));
    }
}
impl RiteIcon {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::RiteIcon, key, block, Box::new(Self {}));
    }
}

impl DbKind for RiteType {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Rite, key);

        // The adjective and adherent locs are derived from the name, like for faiths
        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            data.verify_exists(Item::Localization, key);
            for sfx in ["_adj", "_adherent", "_adherent_plural"] {
                let loca = format!("{key}{sfx}");
                data.verify_exists_implied(Item::Localization, &loca, key);
            }
        }
        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_icon("icon", "NGameIcons|DYNAMIC_RITE_ICON_PATH", ".dds");
        vd.field_validated("color", validate_possibly_named_color);
        vd.field_item("founder", Item::Title);
        vd.field_item("faith", Item::Faith);
        vd.field_list_items("cultures", Item::Culture);
        vd.field_bool("convert");
        vd.field_bool("create");
        vd.field_bool("history");
        vd.field_list_items("tenets", Item::Tenet);
        vd.field_list_items("doctrines", Item::Doctrine);
        if let Some(doctrines) = block.get_field_list("doctrines") {
            validate_doctrine_groups(&doctrines, data);
        }
        vd.multi_field_validated_block("tenet_selection_pair", validate_tenet_selection_pair);
        vd.field_validated_block("localization", validate_localization);
    }
}

/// The scope context for the triggers of rite names and rite icons
fn sc_builder(key: &Token) -> ScopeContext {
    let mut sc = ScopeContext::new(Scopes::Rite, key);
    sc.define_name("founder", Scopes::Character, key);
    sc.define_name("faith", Scopes::Faith, key);
    sc
}

impl DbKind for RiteName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // The other locs are derived from the name
        let name = vd.field_value("name").unwrap_or(key);
        data.verify_exists(Item::Localization, name);
        for sfx in ["_adj", "_adherent", "_adherent_plural", "_desc"] {
            let loca = format!("{name}{sfx}");
            data.verify_exists_implied(Item::Localization, &loca, name);
        }

        vd.field_trigger_builder("trigger", Tooltipped::No, sc_builder);
    }
}

impl DbKind for RiteIcon {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.field_validated_list("custom_faith_icons", |token, data| {
            data.verify_icon("NGameIcons|DYNAMIC_RITE_ICON_PATH", token, ".dds");
        });
        vd.field_trigger_builder("trigger", Tooltipped::No, sc_builder);
    }
}
