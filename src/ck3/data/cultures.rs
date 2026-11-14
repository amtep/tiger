use crate::block::Block;
use crate::ck3::modif::ModifKinds;
use crate::ck3::validate::{validate_cost, validate_maa_upgrade, validate_parameter_value};
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::helpers::TigerHashMap;
use crate::item::{Item, ItemLoader};
use crate::modif::validate_modifs;
use crate::report::{ErrorKey, err, untidy, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::validate_possibly_named_color;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct CultureEra {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureEra, CultureEra::add)
}

impl CultureEra {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureEra, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureEra {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        vd.req_field("year");
        vd.field_integer("year");

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.multi_field_item("invalid_for_government", Item::GovernmentType);
        vd.multi_field_item("custom", Item::Localization);

        validate_modifiers(&mut vd);

        // The docs say this has the same format as in innovations
        vd.multi_field_validated_block("maa_upgrade", validate_maa_upgrade);

        vd.multi_field_item("unlock_building", Item::Building);
        vd.multi_field_item("unlock_decision", Item::Decision);
        vd.multi_field_item("unlock_casus_belli", Item::CasusBelli);
        vd.multi_field_item("unlock_maa", Item::MenAtArms);
        vd.multi_field_item("unlock_law", Item::Law);
    }

    // TODO: validate that none have the same year
    // If they have the same year, the game gets confused about which era is later
}

#[derive(Clone, Debug)]
pub struct Culture {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Culture, Culture::add)
}

impl Culture {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Culture, key, block, Box::new(Self {}));
    }
}

impl DbKind for Culture {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        if let Some(list) = block.get_field_list("coa_gfx") {
            for token in list {
                db.add_flag(Item::CoaGfx, token);
            }
        }
        if let Some(list) = block.get_field_list("building_gfx") {
            for token in list {
                db.add_flag(Item::BuildingGfx, token);
            }
        }
        if let Some(list) = block.get_field_list("clothing_gfx") {
            for token in list {
                db.add_flag(Item::ClothingGfx, token);
            }
        }
        if let Some(list) = block.get_field_list("unit_gfx") {
            for token in list {
                db.add_flag(Item::UnitGfx, token);
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        // let modif = format!("{key}_opinion");
        // data.verify_exists_implied(Item::ModifierFormat, &modif, key);

        if !vd.field_item("name", Item::Localization) {
            data.verify_exists(Item::Localization, key);
        }
        if !vd.field_item("collective_noun", Item::Localization) {
            let loca = format!("{key}_collective_noun");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        if !vd.field_item("prefix", Item::Localization) {
            let loca = format!("{key}_prefix");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.field_date("created");
        if block.has_key("created") {
            vd.field_validated_key_block("parents", |key, block, data| {
                let mut vd = Validator::new(block, data);
                let mut count = 0;
                for token in vd.values() {
                    count += 1;
                    data.verify_exists(Item::Culture, token);
                }
                if count > 2 {
                    let msg = "a culture can have at most 2 parent cultures";
                    warn(ErrorKey::Validation).msg(msg).loc(key).push();
                }
            });
        } else {
            vd.ban_field("parents", || "cultures with `created`");
        }

        vd.field_validated("color", validate_possibly_named_color);

        vd.field_item("ethos", Item::CultureEthos);
        vd.field_item("heritage", Item::CultureHeritage);
        vd.field_item("language", Item::Language);
        vd.field_item("martial_custom", Item::MartialCustom);
        vd.field_item("head_determination", Item::HeadDetermination);

        vd.field_list_items("traditions", Item::CultureTradition);
        vd.field_value("name_order_convention");
        vd.multi_field_item("name_list", Item::NameList);

        vd.multi_field_list_items("coa_gfx", Item::Localization);
        vd.field_list_items("building_gfx", Item::Localization);
        vd.multi_field_list_items("clothing_gfx", Item::Localization);
        vd.field_list_items("unit_gfx", Item::Localization);

        for field in &["house_coa_frame", "dynasty_coa_frame"] {
            if let Some(token) = vd.field_value(field) {
                let pathname = format!("gfx/interface/coat_of_arms/frames/{token}.dds");
                data.verify_exists_implied(Item::File, &pathname, token);
                let pathname = format!("gfx/interface/coat_of_arms/frames/{token}_mask.dds");
                data.verify_exists_implied(Item::File, &pathname, token);
            }
        }
        vd.field_list_numeric_exactly("house_coa_mask_offset", 2);
        vd.field_list_numeric_exactly("house_coa_mask_scale", 2);
        vd.field_list_numeric_exactly("dynasty_coa_mask_offset", 2);
        vd.field_list_numeric_exactly("dynasty_coa_mask_scale", 2);

        vd.field_validated_block("ethnicities", |block, data| {
            let mut vd = Validator::new(block, data);
            for (_, value) in vd.integer_values() {
                data.verify_exists(Item::Ethnicity, value);
            }
        });

        vd.multi_field_validated_block("dlc_tradition", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("trait");
            vd.req_field("requires_dlc_flag");
            vd.field_item("trait", Item::CultureTradition);
            vd.field_item("requires_dlc_flag", Item::DlcFeature);
            vd.field_item("fallback", Item::CultureTradition);
        });
        vd.field_validated_block("dlc_fallback_pillar", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.req_field("fallback");
            vd.req_field("requires_dlc_flag");
            vd.field_item("fallback", Item::CulturePillar);
            vd.field_item("requires_dlc_flag", Item::DlcFeature);
        });

        vd.field_item("history_loc_override", Item::Localization);
    }
}

#[derive(Clone, Debug)]
pub struct CulturePillar {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CulturePillar, CulturePillar::add)
}

impl CulturePillar {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CulturePillar, key, block, Box::new(Self {}));
    }
}

impl DbKind for CulturePillar {
    fn add_subitems(&self, key: &Token, block: &Block, db: &mut Db) {
        add_culture_trait_subitems(block, db);
        if let Some(pillar) = block.get_field_value("type") {
            if pillar.is("language") {
                db.add_flag(Item::Language, key.clone());
            } else if pillar.is("ethos") {
                db.add_flag(Item::CultureEthos, key.clone());
            } else if pillar.is("heritage") {
                db.add_flag(Item::CultureHeritage, key.clone());
            } else if pillar.is("martial_custom") {
                db.add_flag(Item::MartialCustom, key.clone());
            } else if pillar.is("head_determination") {
                db.add_flag(Item::HeadDetermination, key.clone());
            }
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        validate_culture_trait(key, block, data, &mut vd, CultureTraitKind::Pillar);

        vd.field_choice(
            "type",
            &["ethos", "heritage", "language", "martial_custom", "head_determination"],
        );
        if block.field_value_is("type", "heritage") {
            let loca = format!("{key}_collective_noun");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
        if block.field_value_is("type", "language") {
            vd.field_validated("color", validate_possibly_named_color);
        } else {
            vd.ban_field("color", || "languages");
        }
        if block.field_value_is("type", "heritage") {
            vd.field_value("audio_parameter");
        } else {
            vd.ban_field("audio_parameter", || "heritages");
        }
        if block.field_value_is("type", "head_determination") {
            vd.field_choice("head_determination_type", &["domain", "herd"]);
        } else {
            vd.ban_field("head_determination_type", || "head_determination");
        }
    }
}

#[derive(Clone, Debug)]
pub struct CultureTradition {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureTradition, CultureTradition::add)
}

impl CultureTradition {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureTradition, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureTradition {
    fn add_subitems(&self, _key: &Token, block: &Block, db: &mut Db) {
        add_culture_trait_subitems(block, db);
        if let Some(value) = block.get_field_value("category") {
            db.add_flag(Item::CultureTraditionCategory, value.clone());
        }
    }

    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        validate_culture_trait(key, block, data, &mut vd, CultureTraitKind::Tradition);

        vd.req_field("category");
        vd.field_value("category");
        vd.field_validated_block("layers", |block, data| {
            let mut layer_path = Vec::new();
            if let Some(block) =
                data.get_defined_array_warn(key, "NGraphics|CULTURE_TRADITION_LAYER_PATHS")
            {
                for path in block.iter_values_warn() {
                    layer_path.push(path.as_str());
                }
            }

            let mut vd = Validator::new(block, data);
            vd.unknown_value_fields(|key, value| {
                if let Some(layer_idx) =
                    key.expect_integer().and_then(|i| match usize::try_from(i) {
                        Ok(u) if u < layer_path.len() => Some(u),
                        _ => {
                            let msg = format!(
                                "layer index out of range between 0 and {}",
                                layer_path.len()
                            );
                            err(ErrorKey::Range).msg(msg).loc(key).push();
                            None
                        }
                    })
                {
                    let loca = format!("{}/{}", layer_path[layer_idx], value);
                    data.verify_exists_implied(Item::Entry, &loca, value);
                }
            });
        });
    }
}

/// Culture pillars and traditions are both culture traits, documented in `_cultural_traits.info`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CultureTraitKind {
    Pillar,
    Tradition,
}

fn add_culture_trait_subitems(block: &Block, db: &mut Db) {
    if let Some(block) = block.get_field_block("parameters") {
        for token in block.iter_values() {
            db.add_flag(Item::CultureParameter, token.clone());
        }
        for (key, value) in block.iter_assignments() {
            if value.is("yes") {
                db.add_flag(Item::CultureParameter, key.clone());
            }
        }
    }
}

/// Validate the fields that culture pillars and traditions have in common
fn validate_culture_trait(
    key: &Token,
    block: &Block,
    data: &Everything,
    vd: &mut Validator,
    kind: CultureTraitKind,
) {
    // The docs say the name always comes from `<key>_name`
    vd.advice_field("name", "removed in 1.20");
    let loca = format!("{key}_name");
    data.verify_exists_implied(Item::Localization, &loca, key);

    // The docs say this is a dynamic desc with the culture as root
    let mut sc = ScopeContext::new(Scopes::Culture, key);
    if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
        let loca = format!("{key}_desc");
        // The docs say pillars fall back to `<type>_generic_label_desc`
        let fallback = block
            .get_field_value("type")
            .filter(|_| kind == CultureTraitKind::Pillar)
            .map(|pillar| format!("{pillar}_generic_label_desc"));
        if data.item_exists(Item::Localization, &loca) {
            data.mark_used(Item::Localization, &loca);
        } else if let Some(fallback) = fallback
            && data.item_exists(Item::Localization, &fallback)
        {
            data.mark_used(Item::Localization, &fallback);
        } else {
            data.verify_exists_implied(Item::Localization, &loca, key);
        }
    }
    // TODO: verify the icon path. The docs say it defaults to the key, but not where it is.
    vd.field_value("icon");

    let sc_builder = |key: &Token| {
        let mut sc = ScopeContext::new(Scopes::Culture, key);
        sc.define_name("character", Scopes::Character, key);
        sc.define_list("traits", Scopes::CulturePillar | Scopes::CultureTradition, key);
        sc
    };

    if kind == CultureTraitKind::Tradition {
        vd.field_validated_key_block("cost", |key, block, data| {
            let mut sc = sc_builder(key);
            sc.define_name("replacing", Scopes::Bool, key);
            validate_cost(block, data, &mut sc);
        });
    } else {
        vd.ban_field("cost", || "traditions");
    }

    validate_modifiers(vd);
    vd.replaced_field("doctrine_character_modifier", "faith_character_modifier");
    vd.multi_field_validated_block("faith_character_modifier", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.field_item("doctrine", Item::Doctrine);
        vd.field_item("name", Item::Localization);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });

    vd.field_trigger_builder("can_pick", Tooltipped::Yes, |key| {
        let mut sc = sc_builder(key);
        // The docs say it never exists for pillars
        if kind == CultureTraitKind::Tradition {
            sc.define_name("replacing", Scopes::CultureTradition, key);
        }
        sc
    });
    vd.field_trigger_builder("is_shown", Tooltipped::No, sc_builder);
    vd.field_trigger_builder("can_pick_for_hybridization", Tooltipped::Yes, sc_builder);

    vd.field_validated_block("parameters", validate_parameters);
    vd.field_validated_block("partners", validate_partners);
    vd.field_script_value_no_breakdown_builder("ai_will_do", sc_builder);
}

const INTEGER_PARAMETERS: &[&str] = &[
    "number_of_spouses",
    "number_of_consorts",
    "number_of_consorts_barony",
    "number_of_consorts_county",
    "number_of_consorts_duchy",
    "number_of_consorts_kingdom",
    "number_of_consorts_empire",
    "number_of_consorts_hegemony",
];

/// The number of spouses and consorts, which overrides the faith's
fn validate_partners(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_integer_range("number_of_spouses", -1..);
    vd.field_integer_range("number_of_consorts", -1..);
    vd.field_validated_block("number_of_consorts_by_tier", |block, data| {
        let mut vd = Validator::new(block, data);
        for tier in ["barony", "county", "duchy", "kingdom", "empire", "hegemony"] {
            vd.field_integer_range(tier, 0..);
        }
    });
}

fn validate_parameters(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    for token in vd.values() {
        // culture parameter loca are lowercased, verified in 1.11
        let loca = format!("culture_parameter_{}", token.as_str().to_ascii_lowercase());
        data.verify_exists_implied(Item::Localization, &loca, token);
    }
    vd.unknown_value_fields(|key, value| {
        if INTEGER_PARAMETERS.contains(&key.as_str()) {
            let replacement = if key.is("number_of_spouses") || key.is("number_of_consorts") {
                format!("`{key}` in `partners`")
            } else {
                "`number_of_consorts_by_tier` in `partners`".to_string()
            };
            let msg = format!("`{key}` has been replaced by {replacement}");
            err(ErrorKey::Validation).msg(msg).loc(key).push();
            return;
        }
        validate_parameter_value(key, value);
        // culture parameter loca are lowercased, verified in 1.11
        let loca = format!("culture_parameter_{}", key.as_str().to_ascii_lowercase());
        data.verify_exists_implied(Item::Localization, &loca, key);
    });
}

fn validate_modifiers(vd: &mut Validator) {
    vd.multi_field_validated_block("character_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Character, vd);
    });
    vd.multi_field_validated_block("culture_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Culture, vd);
    });
    vd.multi_field_validated_block("county_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::County, vd);
    });
    vd.multi_field_validated_block("province_modifier", |block, data| {
        let vd = Validator::new(block, data);
        validate_modifs(block, data, ModifKinds::Province, vd);
    });
}

#[derive(Clone, Debug)]
pub struct CultureAesthetic {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureAesthetic, CultureAesthetic::add)
}

impl CultureAesthetic {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureAesthetic, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureAesthetic {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let loca = format!("{key}_name");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.req_field("name_list");
        vd.multi_field_item("name_list", Item::NameList);
        for (field, itype) in [
            ("building_gfx", Item::BuildingGfx),
            ("clothing_gfx", Item::ClothingGfx),
            ("unit_gfx", Item::UnitGfx),
            ("coa_gfx", Item::CoaGfx),
        ] {
            vd.req_field(field);
            vd.field_validated_list(field, |token, data| {
                data.verify_exists(itype, token);
                data.verify_exists(Item::Localization, token);
            });
        }

        vd.field_trigger_builder("is_shown", Tooltipped::No, |key| {
            let mut sc = ScopeContext::new(Scopes::Culture, key);
            sc.define_name("character", Scopes::Character, key);
            sc
        });
    }
}

#[derive(Clone, Debug)]
pub struct CultureCreationName {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::CultureCreationName, CultureCreationName::add)
}

impl CultureCreationName {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::CultureCreationName, key, block, Box::new(Self {}));
    }
}

impl DbKind for CultureCreationName {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("culture", Scopes::Culture, key);
        if block.field_value_is("hybrid", "yes") {
            sc.define_name("other_culture", Scopes::Culture, key);
        }

        if !vd.field_validated_sc("name", &mut sc, validate_desc) {
            let loca = format!("{key}_name");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_sc("collective_noun", &mut sc, validate_desc) {
            let loca = format!("{key}_collective_noun");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        if !vd.field_validated_sc("prefix", &mut sc, validate_desc) {
            let loca = format!("{key}_prefix");
            data.verify_exists_implied(Item::Localization, &loca, key);
        }

        vd.req_field("trigger");
        if let Some(key) = block.get_key("trigger")
            && block.get_field_block("trigger").is_some_and(|block| block.num_items() == 0)
        {
            err(ErrorKey::Validation).msg("`trigger` must not be empty").loc(key).push();
        }
        vd.field_trigger("trigger", Tooltipped::No, &mut sc);

        vd.field_bool("hybrid");
    }
}

#[derive(Clone, Debug)]
pub struct NameEquivalency {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::NameEquivalency, NameEquivalency::add)
}

impl NameEquivalency {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::NameEquivalency, key, block, Box::new(Self {}));
    }

    /// Check that no name is in two lists of the same gender. The docs say the game logs an error.
    pub fn validate_all(db: &Db, _data: &Everything) {
        // Each name with its list
        let mut seen: TigerHashMap<(bool, &str), (&Token, &Token)> = TigerHashMap::default();
        // Go in load order, so that reports point at the later list
        let mut lists: Vec<_> = db.iter_key_block(Item::NameEquivalency).collect();
        lists.sort_by_key(|(key, _)| key.loc);
        for (key, block) in lists {
            let female = key.as_str().ends_with("_female");
            for name in block.iter_values() {
                match seen.get(&(female, name.as_str())) {
                    Some(&(other_list, other)) if other_list == key => {
                        let msg = format!("name {name} is listed twice");
                        untidy(ErrorKey::DuplicateItem)
                            .msg(msg)
                            .loc(name)
                            .loc_msg(other, "earlier listing")
                            .push();
                    }
                    Some(&(_, other)) => {
                        let msg = format!("name {name} is in two name equivalency lists");
                        err(ErrorKey::Conflict)
                            .msg(msg)
                            .loc(name)
                            .loc_msg(other, "other list")
                            .push();
                    }
                    None => {
                        seen.insert((female, name.as_str()), (key, name));
                    }
                }
            }
        }
    }
}

impl DbKind for NameEquivalency {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        if !key.as_str().ends_with("_male") && !key.as_str().ends_with("_female") {
            let msg = "name equivalency key should end in `_male` or `_female`";
            let info = "the game logs an error and treats the list as male";
            err(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }

        let mut vd = Validator::new(block, data);
        for name in vd.values() {
            data.verify_exists(Item::Localization, name);
        }
    }
}
