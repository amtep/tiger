use crate::block::Block;
use crate::ck3::validate::validate_portrait_modifier_overrides;
use crate::context::ScopeContext;
use crate::data::dna::validate_genes;
use crate::date::Date;
use crate::db::{Db, DbKind};
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader, LoadAsFile, Recursive};
use crate::pdxfile::PdxEncoding;
use crate::report::{ErrorKey, err, fatal, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct BookmarkGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::BookmarkGroup, BookmarkGroup::add)
}

impl BookmarkGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BookmarkGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for BookmarkGroup {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_date("default_start_date");
    }
}

#[derive(Clone, Debug)]
pub struct Bookmark {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Bookmark, Bookmark::add)
}

impl Bookmark {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Bookmark, key, block, Box::new(Self {}));
    }
}

impl DbKind for Bookmark {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        // The docs say the group's `default_start_date` is used if this is not set
        let group_start_date = block
            .get_field_value("group")
            .and_then(|group| data.database.get_key_block(Item::BookmarkGroup, group.as_str()))
            .and_then(|(_, block)| block.get_field_date("default_start_date"));
        if group_start_date.is_none() {
            vd.req_field("start_date");
        }
        vd.field_date("start_date");
        vd.field_bool("is_playable");
        vd.field_bool("recommended");
        vd.field_bool("test_default");
        vd.field_item("group", Item::BookmarkGroup);
        vd.field_item("requires_dlc_flag", Item::DlcFeature);
        vd.field_script_value("weight", &mut sc);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_desc");
        data.verify_exists_implied(Item::Localization, &loca, key);

        let pathname = format!("gfx/interface/bookmarks/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);
        data.verify_icon("NGameIcons|BOOKMARK_START_BUTTON_PATH", key, ".dds");
        let pathname = format!("gfx/interface/icons/bookmark_buttons/{key}.dds");
        data.verify_exists_implied(Item::File, &pathname, key);

        let start_date = block.get_field_date("start_date").or(group_start_date);
        vd.multi_field_validated_block("character", |block, data| {
            // The docs say the location image is not required with `display = no`
            if let Some(name) = block.get_field_value("name")
                && !block.field_value_is("display", "no")
            {
                let pathname = format!("gfx/interface/bookmarks/{key}_{name}.dds");
                data.verify_exists_implied(Item::File, &pathname, name);
            }
            validate_bookmark_character(block, data, true, start_date);
        });
    }
}

fn validate_bookmark_character(
    block: &Block,
    data: &Everything,
    toplevel: bool,
    start_date: Option<Date>,
) {
    let existing_ruler =
        !block.get_field_value("character_design_type").is_some_and(|t| t.is("noble_family"));
    let mut vd = Validator::new(block, data);
    vd.field_bool("tutorial");
    vd.field_bool("test_default");
    vd.field_bool("display");
    if block.field_value_is("display", "no") {
        vd.field_value("name");
    } else {
        vd.field_item("name", Item::Localization);
        if existing_ruler
            && let Some(name) = block.get_field_value("name")
            && !data.item_exists(Item::BookmarkPortrait, name.as_str())
        {
            let msg =
                format!("bookmark portrait for {name} not found in common/bookmark_portraits");
            let info = "This causes a crash in CK3 1.13";
            fatal(ErrorKey::Crash).msg(msg).info(info).loc(name).push();
        }
    }
    if toplevel {
        if let Some(token) = block.get_field_value("name") {
            let loca = format!("{token}_desc");
            data.verify_exists_implied(Item::Localization, &loca, token);
        }
        vd.req_field("difficulty");
        vd.req_field("government");
        if !block.has_key("target_title") {
            vd.req_field("title");
        }
    } else {
        vd.field_item("relation", Item::Localization);
    }
    vd.field_item("dynasty", Item::Dynasty);
    vd.field_item("dynasty_house", Item::House);
    if let Some(token) =
        block.get_field_value("dynasty_house").or_else(|| block.get_field_value("dynasty"))
        && !data.item_exists(Item::Coa, token.as_str())
    {
        let msg = format!("{} {token} not defined in {}", Item::Coa, Item::Coa.path());
        let info = "bookmark characters must have a defined coa or their shields will be blank";
        warn(ErrorKey::MissingItem).msg(msg).info(info).loc(token).push();
    }
    vd.field_integer("dynasty_splendor_level");
    // The docs say it must be above 0 if the character has a dynasty
    if let Some(level) = block.get_field_value("dynasty_splendor_level")
        && (block.has_key("dynasty") || block.has_key("dynasty_house"))
        && level.get_integer().is_some_and(|level| level <= 0)
    {
        let msg = "`dynasty_splendor_level` must be above 0 for a character with a dynasty";
        err(ErrorKey::Range).msg(msg).loc(level).push();
    }
    vd.field_choice("type", &["male", "female", "boy", "girl"]);
    vd.field_date("birth");
    vd.field_item("title", Item::Title);
    vd.field_item("title_text_override", Item::Localization);
    vd.field_item("government", Item::GovernmentType);
    vd.field_item("fallback_government", Item::GovernmentType);
    vd.field_item("culture", Item::Culture);
    // The docs say this is the localization key of the religion shown
    vd.field_item("religion", Item::Localization);
    vd.field_item("difficulty", Item::Localization);
    vd.field_item("history_id", Item::Character);
    vd.field_item("animation", Item::PortraitAnimation);
    vd.field_validated_block("position", |block, data| {
        let mut vd = Validator::new(block, data);
        vd.req_tokens_integers_exactly(2);
    });
    vd.field_choice(
        "character_design_type",
        &["default", "landless_adventurer", "noble_family", "none"],
    );
    vd.replaced_field("bookmark_type", "character_design_type");
    vd.field_item("target_title", Item::Title);
    if let Some(start_date) = start_date
        && let Some(id) = block.get_field_value("history_id")
    {
        let name = block.get_field_value("name");
        if data.item_exists(Item::Character, id.as_str()) {
            validate_bookmark_against_history(
                block.get_field_value("dynasty"),
                "dynasty",
                start_date,
                data.characters.get_dynasty(id, start_date, data),
                name,
            );
            validate_bookmark_against_history(
                block.get_field_value("dynasty_house"),
                "house",
                start_date,
                data.characters.get_house(id, start_date),
                name,
            );
            validate_bookmark_against_history(
                block.get_field_value("culture"),
                "culture",
                start_date,
                data.characters.get_culture(id, start_date),
                name,
            );
            // `religion` is a loca key, but vanilla uses the key of the character's rite.
            // A history that gives only a faith implies the faith's main rite. Skip faiths whose
            // main rite is created dynamically.
            let faith = data.characters.get_faith(id, start_date);
            let rite = data.characters.get_rite(id, start_date).or_else(|| {
                faith.and_then(|faith| faith_main_rite(faith.as_str(), start_date, data))
            });
            if let Some(religion) = block.get_field_value("religion")
                && data.item_exists(Item::RiteType, religion.as_str())
                && (rite.is_some() || faith.is_none())
            {
                validate_bookmark_against_history(Some(religion), "rite", start_date, rite, name);
            }
        }
    }
    vd.multi_field_validated_block("character", |block, data| {
        validate_bookmark_character(block, data, false, start_date);
    });
}

#[derive(Clone, Debug)]
pub struct BookmarkPortrait {}

inventory::submit! {
    ItemLoader::Full(GameFlags::Ck3, Item::BookmarkPortrait, PdxEncoding::Utf8OptionalBom, ".txt", LoadAsFile::No, Recursive::Yes, BookmarkPortrait::add)
}

impl BookmarkPortrait {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::BookmarkPortrait, key, block, Box::new(Self {}));
    }
}

impl DbKind for BookmarkPortrait {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_choice("type", &["male", "female", "boy", "girl"]);
        vd.field_integer("id");
        vd.field_integer("random_seed");
        vd.field_precise_numeric_range("age", 0.0..=1.0);
        vd.field_list_integers_exactly("entity", 2);
        vd.field_validated_block("genes", validate_genes);
        vd.field_validated_block("override", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_validated_block(
                "portrait_modifier_overrides",
                validate_portrait_modifier_overrides,
            );
            vd.field_validated_block("colors", |block, data| {
                let mut vd = Validator::new(block, data);
                for field in ["skin", "eye", "hair"] {
                    vd.field_list_precise_numeric_exactly(field, 3);
                    for token in block.get_field_list(field).unwrap_or_default() {
                        if token.get_number().is_some_and(|v| !(0.0..=1.0).contains(&v)) {
                            let msg = "color values should be between 0 and 1";
                            err(ErrorKey::Colors).msg(msg).loc(token).push();
                        }
                    }
                }
            });
        });
        vd.field_validated_block("tags", |block, data| {
            let mut vd = Validator::new(block, data);
            for block in vd.blocks() {
                let mut vd = Validator::new(block, data);
                vd.field_integer("hash");
                vd.field_bool("invert");
            }
        });
    }
}

/// The main rite of a faith at `date`, from the faith history or else the faith itself
fn faith_main_rite<'a>(faith: &str, date: Date, data: &'a Everything) -> Option<&'a Token> {
    data.database
        .get_key_block(Item::FaithHistory, faith)
        .and_then(|(_, block)| block.get_field_value_at_date("main_rite", date))
        .or_else(|| {
            data.database
                .get_key_block(Item::Faith, faith)
                .and_then(|(_, block)| block.get_field_value("main_rite"))
        })
}

fn validate_bookmark_against_history(
    field: Option<&Token>,
    desc: &str,
    date: Date,
    history: Option<&Token>,
    name: Option<&Token>,
) {
    if let Some(field) = field {
        if let Some(history) = history {
            if field != history {
                let msg = format!(
                    "{desc} is {field} in bookmark but {history} in character history at {date}"
                );
                warn(ErrorKey::Bookmarks)
                    .strong()
                    .msg(msg)
                    .loc_msg(field, "bookmark")
                    .loc_msg(history, "history")
                    .opt_loc_msg(name, "character")
                    .push();
            }
        } else {
            let msg = format!(
                "{desc} is {field} in bookmark but character has no {desc} in history at {date}"
            );
            warn(ErrorKey::Bookmarks)
                .strong()
                .msg(msg)
                .loc(field)
                .opt_loc_msg(name, "bookmark")
                .push();
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChallengeCharacter {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::ChallengeCharacter, ChallengeCharacter::add)
}

impl ChallengeCharacter {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ChallengeCharacter, key, block, Box::new(Self {}));
    }
}

impl DbKind for ChallengeCharacter {
    fn validate(&self, _key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        vd.field_date("start_date");
        let start_date = block.get_field_date("start_date");
        vd.field_validated_block("character", |block, data| {
            validate_bookmark_character(block, data, true, start_date);
        });

        vd.field_list_items("achievements", Item::Achievement);
    }
}
