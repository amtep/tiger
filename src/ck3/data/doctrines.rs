use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::ck3::validate::{validate_parameter_value, validate_traits};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::{TigerHashMap, TigerHashSet, stringify_choices};
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, err, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct DoctrineCategory {}
#[derive(Clone, Debug)]
pub struct DoctrineGroup {}
#[derive(Clone, Debug)]
pub struct Doctrine {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DoctrineCategory, DoctrineCategory::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DoctrineGroup, DoctrineGroup::add)
}
inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Doctrine, Doctrine::add)
}

impl DoctrineCategory {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DoctrineCategory, key, block, Box::new(Self {}));
    }
}
impl DoctrineGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DoctrineGroup, key, block, Box::new(Self {}));
    }
}
impl Doctrine {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Doctrine, key, block, Box::new(Self {}));
    }
}

impl DbKind for DoctrineCategory {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("doctrine_category_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_bool("visible");
        vd.field_integer("sort_order");
        vd.field_list("gui_tags");
    }
}

impl DbKind for DoctrineGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        // Optional: the icon of a doctrine that has none, which the doctrine checks
        data.mark_used_icon("NGameIcons|DOCTRINE_GROUP_TYPE_ICON_PATH", key, ".dds");

        vd.field_item("category", Item::DoctrineCategory);
        vd.field_bool("main_rite");
        vd.field_choice("divergence", &["category", "sequence"]);
        vd.field_choice("doctrine_lock", &["none", "rite", "faith", "religion"]);

        vd.advice_field("number_of_picks", "removed in 1.20");
        vd.field_trigger_rooted("is_available_on_create", Tooltipped::No, Scopes::Faith);

        vd.advice_field(
            "doctrine_types",
            "removed in 1.20; doctrines now name their group with `doctrine_group_type`",
        );
    }
}

impl DbKind for Doctrine {
    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        property == "unreformed" && is_unreformed(block)
    }

    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(block) = block.get_field_block("parameters") {
            for token in block.iter_values() {
                db.add_flag(Item::DoctrineParameter, token.clone());
            }
            // undocumented
            for (key, _) in block.iter_assignments() {
                db.add_flag(Item::DoctrineParameter, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        // `selected_doctrines` holds the selected core tenets and doctrines
        let selected = Scopes::Doctrine | Scopes::Tenet;
        let mut sc = ScopeContext::new(Scopes::Faith, key);
        let mut can_pick_sc = ScopeContext::new(Scopes::Faith, key);
        can_pick_sc.define_name("actor", Scopes::Character, key);
        can_pick_sc.define_list("selected_doctrines", selected, key);

        let icon = vd.field_value("icon");
        // Without an icon file of its own, a doctrine shows its group's icon
        let uses_group_icon = icon.is_none()
            && !icon_exists("NGameIcons|DOCTRINE_TYPE_ICON_PATH", key, data)
            && block.get_field_value("doctrine_group_type").is_some_and(|group| {
                icon_exists("NGameIcons|DOCTRINE_GROUP_TYPE_ICON_PATH", group, data)
            });
        if !uses_group_icon {
            data.verify_icon("NGameIcons|DOCTRINE_TYPE_ICON_PATH", icon.unwrap_or(key), ".dds");
        }

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_sc("descriptive_name", &mut sc, validate_desc) {
            let loca = format!("{key}_descriptive_name");
            data.mark_used(Item::Localization, &loca);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_validated_sc("historical_glossary", &mut sc, validate_desc);

        vd.field_bool("visible");
        vd.field_numeric("divergence");
        vd.req_field("doctrine_group_type");
        vd.field_item("doctrine_group_type", Item::DoctrineGroup);
        vd.field_integer("index");
        vd.field_validated_block("parameters", validate_parameters);
        vd.field_script_value_rooted("piety_cost", Scopes::Rite);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc);
        vd.field_trigger("can_pick", Tooltipped::Yes, &mut can_pick_sc);

        vd.field_validated_block("character_modifier", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_item("name", Item::Localization); // undocumented
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.field_validated_block("clergy_modifier", |block, data| {
            let vd = Validator::new(block, data);
            validate_modifs(block, data, ModifKinds::Character, vd);
        });

        vd.advice_field("doctrine_character_modifier", "removed in 1.20");

        vd.field_validated_block("traits", validate_traits);
        vd.field_validated_block("special_parameters", validate_special_parameters);
    }
}

fn icon_exists(define: &str, name: &Token, data: &Everything) -> bool {
    data.get_defined_string(define)
        .is_some_and(|path| data.item_exists(Item::File, &format!("{path}/{name}.dds")))
}

fn validate_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for name in vd.values() {
        verify_parameter_loca(name, data);
    }
    // The `parameter = yes` form is undocumented, but still works (verified in 1.20.1)
    vd.unknown_value_fields(|key, value| {
        verify_parameter_loca(key, data);
        validate_parameter_value(key, value);
    });
}

/// The docs say a doctrine's effects are localized with `doctrine_parameter_` plus the parameter
/// name, and that tenets follow the doctrine docs
pub fn verify_parameter_loca(name: &Token, data: &Everything) {
    // TODO: verify. The game may skip a parameter without this loca instead of showing the key.
    let loca = format!("doctrine_parameter_{name}");
    data.verify_exists_implied(Item::Localization, &loca, name);
}

/// Check that no doctrine is listed twice, and that no two doctrines are from the same group
pub fn validate_doctrine_groups(doctrines: &[Token], data: &Everything) {
    let mut seen: TigerHashSet<&Token> = TigerHashSet::default();
    let mut groups: TigerHashMap<&str, &Token> = TigerHashMap::default();
    for doctrine in doctrines {
        if let Some(&other) = seen.get(doctrine) {
            let msg = format!("duplicate doctrine {doctrine}");
            warn(ErrorKey::DuplicateField)
                .msg(msg)
                .loc(doctrine)
                .loc_msg(other, "earlier doctrine")
                .push();
            continue;
        }
        seen.insert(doctrine);
        let Some((_, block)) = data.database.get_key_block(Item::Doctrine, doctrine.as_str())
        else {
            continue;
        };
        let Some(group) = block.get_field_value("doctrine_group_type") else {
            continue;
        };
        if let Some(other) = groups.get(group.as_str()) {
            let msg = format!("{doctrine} and {other} are both from {group}");
            let info = format!("{group} only allows 1 pick");
            err(ErrorKey::Conflict)
                .msg(msg)
                .info(info)
                .loc(doctrine)
                .loc_msg(*other, "earlier doctrine")
                .push();
        } else {
            groups.insert(group.as_str(), doctrine);
        }
    }
}

const HOSTILITY_LEVELS: &[&str] = &["righteous", "astray", "hostile", "evil"];

/// Whether a doctrine or tenet makes its faith unreformed
pub fn is_unreformed(block: &Block) -> bool {
    block
        .get_field_block("special_parameters")
        .is_some_and(|special| special.field_value_is("unreformed", "yes"))
}

/// The `special_parameters` block of doctrines and tenets
pub fn validate_special_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_integer("number_of_spouses");
    vd.field_integer("number_of_consorts");
    vd.field_integer("divine_marriage_opinion");
    vd.field_integer("minimum_fervor");
    vd.field_numeric("fervor_per_holy_site");
    vd.field_numeric("bonus_fervor_gain");
    vd.field_integer("bonus_heresy_protection");
    vd.field_numeric("divergence_protection");
    vd.field_numeric("heresy_threshold");
    vd.field_validated_block("hostility_levels", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_choice("same_religion", HOSTILITY_LEVELS);
        vd.field_choice("same_family", HOSTILITY_LEVELS);
        vd.field_choice("unrelated", HOSTILITY_LEVELS);
    });
    vd.field_validated_block("hostility_level_override", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            if !data.item_exists(Item::Doctrine, key.as_str()) {
                data.verify_exists(Item::Tenet, key);
            }
            if !HOSTILITY_LEVELS.contains(&value.as_str()) {
                let msg = format!("expected one of {}", stringify_choices(HOSTILITY_LEVELS));
                err(ErrorKey::Choice).msg(msg).loc(value).push();
            }
        });
    });
    vd.field_choice("same_hof_hostility_override", HOSTILITY_LEVELS);
    vd.field_validated_block("parameter_opinion_override", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.unknown_value_fields(|key, value| {
            data.verify_exists(Item::DoctrineParameter, key);
            // The docs say the opinion tooltip uses this loca
            let loca = format!("doctrine_parameter_{key}_opinion_active");
            data.verify_exists_implied(Item::Localization, &loca, key);
            value.expect_integer();
        });
    });
    vd.field_numeric("hostility_towards_us_mult");
    vd.field_numeric("piety_from_long_peace");
    vd.field_list_items("allowed_holding_types", Item::HoldingType);
    vd.field_bool("unreformed");
    vd.field_bool("warmonger");
    vd.field_numeric("increased_dynasty_prestige_birth_mult");
    vd.field_numeric("increased_dynasty_prestige_marriage_mult");
    vd.field_numeric("increased_dynasty_prestige_long_reign_mult");
}
