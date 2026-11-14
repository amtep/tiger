use std::path::PathBuf;

use crate::block::Block;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
use crate::helpers::{TigerHashMap, TigerHashSet, dup_error};
use crate::item::Item;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{ErrorKey, err};
use crate::token::Token;
use crate::validate::validate_color;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Clone, Debug, Default)]
pub struct CharacterInteractionCategories {
    categories: TigerHashMap<&'static str, Category>,
}

impl CharacterInteractionCategories {
    pub fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.categories.get(key.as_str())
            && other.key.loc.kind == key.loc.kind
        {
            dup_error(&key, &other.key, "interaction category");
        }
        self.categories.insert(key.as_str(), Category::new(key, block));
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.categories.values() {
            registry.scan(&item.block);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.categories.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.categories.values().map(|item| &item.key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.categories.values() {
            item.validate(data);
        }

        // The docs say the game logs an error for categories that no interaction uses
        let used: TigerHashSet<&str> = data
            .database
            .iter_key_block(Item::CharacterInteraction)
            .filter_map(|(_, block)| block.get_field_value("category"))
            .map(Token::as_str)
            .collect();
        for item in self.categories.values() {
            if !used.contains(item.key.as_str())
                && !item.block.field_value_is("default", "yes")
                && !item.block.field_value_is("favorite_interactions", "yes")
            {
                let msg = "no interaction uses this category";
                err(ErrorKey::Validation).msg(msg).loc(&item.key).push();
            }
        }
    }
}

impl FileHandler<Block> for CharacterInteractionCategories {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/character_interaction_categories")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }

    fn finalize(&mut self) {
        let mut taken = vec![None; self.categories.len()];
        for item in self.categories.values() {
            if let Some(index) = item.index {
                if index >= 0 {
                    let index = usize::try_from(index).expect("internal error");
                    if index < taken.len() {
                        if let Some(other) = taken[index] {
                            let msg = format!("index duplicates the index of {other}");
                            err(ErrorKey::DuplicateItem).msg(msg).loc(&item.key).push();
                        } else {
                            taken[index] = Some(&item.key);
                        }
                        continue;
                    }
                }
                let msg = "index needs to be from 0 to the number of categories";
                err(ErrorKey::Range).msg(msg).loc(&item.key).push();
            }
            // if no index, the item will warn about that in validate
        }

        // The docs say the first one read is kept
        let mut categories: Vec<_> = self.categories.values().collect();
        categories.sort_by_key(|item| item.key.loc);
        for field in ["default", "favorite_interactions"] {
            let mut first: Option<&Token> = None;
            for item in &categories {
                if item.block.field_value_is(field, "yes") {
                    if let Some(other) = first {
                        let msg = format!("only one category can have `{field} = yes`");
                        err(ErrorKey::Validation)
                            .msg(msg)
                            .loc(&item.key)
                            .loc_msg(other, "other category")
                            .push();
                    } else {
                        first = Some(&item.key);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Category {
    key: Token,
    block: Block,
    index: Option<i64>,
}

impl Category {
    pub fn new(key: Token, block: Block) -> Self {
        let index = block.get_field_integer("index");
        Category { key, block, index }
    }

    pub fn validate(&self, data: &Everything) {
        data.verify_exists(Item::Localization, &self.key);

        let mut vd = Validator::new(&self.block, data);
        vd.req_field("index");
        vd.field_integer("index");
        vd.field_item("desc", Item::Localization);
        vd.field_icon("icon", "NGameIcons|CHARACTER_INTERACTION_CATEGORY_ICON_PATH", ".dds");
        vd.field_bool("default");
        vd.field_bool("favorite_interactions");
        vd.field_validated_block("color", validate_color);
    }
}
