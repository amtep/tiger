use crate::block::{BV, Block};
use crate::ck3::data::doctrines::validate_doctrine_groups;
use crate::ck3::tables::misc::CUSTOM_RELIGION_LOCAS;
use crate::ck3::validate::validate_traits;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::fileset::FileKind;
use crate::game::GameFlags;
use crate::helpers::TigerHashSet;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct Religion {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Religion, Religion::add)
}

impl Religion {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Religion, key, block, Box::new(Self {}));
    }
}

impl DbKind for Religion {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("custom_faith_icons") {
            for token in block.iter_values() {
                db.add_flag(Item::FaithIcon, token.clone());
            }
        }
        if let Some(block) = block.get_field_block("religion_details")
            && let Some(token) = block.get_field_value("graphical_faith")
        {
            db.add_flag(Item::GraphicalFaith, token.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_adj");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent_plural");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);

        vd.req_field("religion_details");
        vd.field_validated_block("religion_details", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("family");
            vd.field_item("family", Item::ReligionFamily);
            vd.field_value("graphical_faith");
            validate_piety_icon_group(vd.field_value("piety_icon_group"), data);
            validate_tenet_background_icons(data, &mut vd);
            // The docs say all four must be set to override the family's icons
            let icons: Vec<_> =
                TENET_BACKGROUND_ICONS.iter().filter_map(|f| block.get_key(f)).collect();
            if !icons.is_empty() && icons.len() < TENET_BACKGROUND_ICONS.len() {
                let msg = "religion sets only some of the tenet background icons";
                let info = "all four must be set to override the religion family's icons";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(icons[0]).push();
            }
            vd.field_item("theocracy_government_type", Item::GovernmentType);
            vd.field_item("theocracy_lease_contract_type", Item::LeaseContract);
        });
        for field in ["family", "graphical_faith", "piety_icon_group"] {
            vd.replaced_field(field, &format!("`{field}` in `religion_details`"));
        }
        vd.replaced_field(
            "doctrine_background_icon",
            "`tenet_background_icon` in `religion_details`",
        );

        vd.field_item("main_holy_site", Item::HolySite);
        vd.field_integer("eminent_holy_sites_max");
        vd.field_integer("holy_sites_max");
        vd.field_integer("eminent_holy_sites_min");
        vd.field_integer("holy_sites_min");

        vd.field_bool("pagan_roots");
        validate_doctrines(data, &mut vd);
        vd.field_validated_block("traits", validate_traits);

        vd.field_validated_list("custom_faith_icons", |icon, data| {
            data.verify_icon("NGameIcons|FAITH_ICON_PATH", icon, ".dds");
        });

        vd.field_list("reserved_male_names"); // TODO
        vd.field_list("reserved_female_names"); // TODO

        vd.field_validated_block("holy_order_names", validate_holy_order_names);
        vd.field_list_items("holy_order_maa", Item::MenAtArms);
        vd.field_validated_block("localization", validate_localization);
        vd.replaced_field("faiths", "top-level faiths in `common/religion/faith_types`");
    }

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if let Some(block) = block.get_field_block("localization") {
            block.has_key(property)
        } else {
            false
        }
    }
}

fn validate_doctrines(data: &Everything, vd: &mut Validator) {
    let mut doctrines = Vec::new();
    vd.multi_field_validated_value("doctrine", |_, mut vd| {
        vd.item(Item::Doctrine);
        doctrines.push(vd.value().clone());
    });
    validate_doctrine_groups(&doctrines, data);
    vd.replaced_field("doctrine_selection_pair", "`tenet_selection_pair` in faiths and rites");
}

const TENET_BACKGROUND_ICONS: &[&str] = &[
    "tenet_background_icon",
    "tenet_heretical_background_icon",
    "tenet_neutral_background_icon",
    "tenet_unknown_background_icon",
];

/// The tenet background icons of religions and religion families
fn validate_tenet_background_icons(data: &Everything, vd: &mut Validator) {
    for field in TENET_BACKGROUND_ICONS {
        if let Some(icon) = vd.field_value(field) {
            data.verify_icon("NGameIcons|TENET_BACKGROUND_PATH", icon, ".dds");
            data.verify_icon("NGameIcons|TENET_BACKGROUND_PATH", icon, "_small.dds");
        }
    }
}

pub(crate) fn validate_tenet_selection_pair(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.req_field("requires_dlc_flag");
    vd.field_item("requires_dlc_flag", Item::DlcFeature);
    vd.req_field("tenet");
    vd.field_item("tenet", Item::Tenet);
    vd.field_item("fallback_tenet", Item::Tenet);
}

pub(crate) fn validate_localization(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for field in CUSTOM_RELIGION_LOCAS {
        vd.field_validated(field, |bv, data| match bv {
            BV::Value(token) => data.verify_exists(Item::Localization, token),
            BV::Block(block) => {
                let mut vd = Validator::new(block, data);
                for token in vd.values() {
                    data.verify_exists(Item::Localization, token);
                }
            }
        });
    }
}

fn validate_holy_order_names(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for block in vd.blocks() {
        let mut vd = Validator::new(block, data);
        vd.req_field("name");
        vd.field_item("name", Item::Localization);
        vd.field_item("coat_of_arms", Item::Coa);
    }
}

#[derive(Clone, Debug)]
pub struct Faith {
    religion: Option<Token>,
}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Faith, Faith::add)
}

impl Faith {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        let religion = block
            .get_field_block("faith_details")
            .and_then(|block| block.get_field_value("religion"))
            .cloned();
        db.add(Item::Faith, key, block, Box::new(Self { religion }));
    }

    fn check_have_customs(&self, key: &Token, block: &Block, data: &Everything) {
        let locas = block.get_field_block("localization");
        for loca in CUSTOM_RELIGION_LOCAS {
            if let Some(block) = locas
                && block.has_key(loca)
            {
                continue;
            }
            if let Some(religion) = &self.religion
                && data.item_has_property(Item::Religion, religion.as_str(), loca)
            {
                continue;
            }
            let msg = format!("faith or religion missing localization for {loca}");
            warn(ErrorKey::MissingLocalization).msg(msg).loc(key).push();
        }
    }
}

impl DbKind for Faith {
    fn add_subitems(&self, key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("faith_details") {
            if let Some(token) = block.get_field_value("graphical_faith") {
                db.add_flag(Item::GraphicalFaith, token.clone());
            }
            if let Some(token) = block.get_field_value("icon") {
                db.add_flag(Item::FaithIcon, token.clone());
            } else {
                db.add_flag(Item::FaithIcon, key.clone());
            }
            if let Some(token) = block.get_field_value("reformed_icon") {
                db.add_flag(Item::FaithIcon, token.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_adj");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_adherent_plural");
        data.verify_exists_implied(Item::Localization, &loca, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // The docs say any doctrine or core tenet can make a faith unreformed, and the
        // religion's doctrines apply to its faiths
        let unreformed = |itype, keys: &[Token]| {
            keys.iter().any(|key| data.item_has_property(itype, key.as_str(), "unreformed"))
        };
        let religion_doctrines: Vec<Token> = block
            .get_field_block("faith_details")
            .and_then(|block| block.get_field_value("religion"))
            .and_then(|religion| data.database.get_key_block(Item::Religion, religion.as_str()))
            .map(|(_, block)| block.get_field_values("doctrine").into_iter().cloned().collect())
            .unwrap_or_default();
        // The docs say the main rite's tenets and doctrines become the faith's, and in game
        // they can make it unreformed
        let main_rite = block
            .get_field_value("main_rite")
            .and_then(|rite| data.database.get_key_block(Item::RiteType, rite.as_str()))
            .map(|(_, block)| block);
        let rite_list = |field| main_rite.and_then(|block| block.get_field_list(field));
        let pagan =
            unreformed(Item::Doctrine, &block.get_field_list("doctrines").unwrap_or_default())
                || unreformed(Item::Tenet, &block.get_field_list("tenets").unwrap_or_default())
                || unreformed(Item::Doctrine, &religion_doctrines)
                || unreformed(Item::Doctrine, &rite_list("doctrines").unwrap_or_default())
                || unreformed(Item::Tenet, &rite_list("tenets").unwrap_or_default());
        if pagan {
            let loca = format!("{key}_old");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_old_adj");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_old_adherent");
            data.verify_exists_implied(Item::Localization, &loca, key);
            let loca = format!("{key}_old_adherent_plural");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let mut vd = Validator::new(block, data);

        vd.req_field("faith_details");
        vd.field_validated_block("faith_details", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("religion");
            vd.field_item("religion", Item::Religion);

            vd.req_field("color");
            vd.field_validated("color", validate_possibly_named_color);

            let icon = vd.field_value("icon").unwrap_or(key);
            data.verify_icon("NGameIcons|FAITH_ICON_PATH", icon, ".dds");
            if pagan {
                vd.req_field_fatal("reformed_icon");
            } else {
                vd.ban_field("reformed_icon", || "unreformed faiths");
            }
            vd.field_icon("reformed_icon", "NGameIcons|FAITH_ICON_PATH", ".dds");
            vd.field_item("religious_head", Item::Title);
            vd.field_item("head_of_rite", Item::Title);
            vd.field_value("graphical_faith");
            vd.field_item("theocracy_government_type", Item::GovernmentType);
        });
        for field in ["color", "icon", "reformed_icon", "graphical_faith", "religious_head"] {
            vd.replaced_field(field, &format!("`{field}` in `faith_details`"));
        }

        vd.field_item("origin", Item::Faith);

        vd.field_list_items("holy_sites", Item::HolySite);
        vd.field_list_items("eminent_holy_sites", Item::HolySite);
        validate_holy_sites(block);
        vd.replaced_field("holy_site", "`holy_sites` or `eminent_holy_sites`");

        vd.field_list_items("tenets", Item::Tenet);
        vd.field_list_items("doctrines", Item::Doctrine);
        if let Some(doctrines) = block.get_field_list("doctrines") {
            validate_doctrine_groups(&doctrines, data);
        }
        vd.replaced_field("doctrine", "doctrines");
        vd.replaced_field("doctrine_selection_pair", "tenet_selection_pair");
        vd.multi_field_validated_block("tenet_selection_pair", validate_tenet_selection_pair);

        vd.field_list("reserved_male_names");
        vd.field_list("reserved_female_names");
        vd.field_list_items("cultures", Item::Culture);
        vd.field_list_items("clerical_elector_titles", Item::Title);
        vd.field_bool("historical");
        vd.field_item("main_rite", Item::RiteType);
        vd.field_validated_block("localization", validate_localization);
        vd.field_validated_block("holy_order_names", validate_holy_order_names);

        self.check_have_customs(key, block, data);
    }

    fn has_property(
        &self,
        key: &Token,
        _block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "is_modded" {
            return key.loc.kind == FileKind::Mod;
        }
        false
    }
}

/// The docs say a holy site is either eminent or not, and is never listed twice
fn validate_holy_sites(block: &Block) {
    let mut seen: TigerHashSet<Token> = TigerHashSet::default();
    for field in ["eminent_holy_sites", "holy_sites"] {
        for site in block.get_field_list(field).unwrap_or_default() {
            if let Some(other) = seen.get(&site) {
                let msg = format!("holy site {site} is listed twice");
                warn(ErrorKey::DuplicateItem)
                    .msg(msg)
                    .loc(&site)
                    .loc_msg(other, "earlier listing")
                    .push();
            } else {
                seen.insert(site);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReligionFamily {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ReligionFamily, ReligionFamily::add)
}

impl ReligionFamily {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ReligionFamily, key, block, Box::new(Self {}));
    }
}

impl DbKind for ReligionFamily {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(token) = block.get_field_value("graphical_faith") {
            db.add_flag(Item::GraphicalFaith, token.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        let name = vd.field_value("name").unwrap_or(key);
        data.verify_exists(Item::Localization, name);
        let loca = format!("{name}_desc");
        data.verify_exists_implied(Item::Localization, &loca, name);

        vd.field_bool("is_pagan");
        vd.field_value("graphical_faith");
        validate_piety_icon_group(vd.field_value("piety_icon_group"), data);
        validate_tenet_background_icons(data, &mut vd);
        vd.replaced_field("doctrine_background_icon", "tenet_background_icon");
        vd.field_item("hostility_doctrine", Item::Doctrine);
    }
}

#[derive(Clone, Debug)]
pub struct FervorModifier {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::FervorModifier, FervorModifier::add)
}

impl FervorModifier {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::FervorModifier, key, block, Box::new(Self {}));
    }
}

impl DbKind for FervorModifier {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Faith, key);
        vd.field_script_value("value", &mut sc);
        vd.field_trigger("trigger", Tooltipped::No, &mut sc);
    }
}

fn validate_piety_icon_group(group: Option<&Token>, data: &Everything) {
    if let Some(group) = group {
        if let Some(valid) = data.get_defined_array_warn(group, "NGameIcons|PIETY_GROUPS") {
            for valid_group in valid.iter_values() {
                if group == valid_group {
                    return;
                }
            }
        }
        let msg = "piety icon group not listed in PIETY_GROUPS define";
        warn(ErrorKey::Choice).msg(msg).loc(group).push();
    }
}
