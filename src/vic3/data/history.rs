use std::path::PathBuf;
use std::sync::Arc;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::files::{FileEntry, FileHandler};
use crate::helpers::TigerHashMap;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{err, warn, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::variables::Variables;

/// The history files in Vic3 are fairly simple. Files under `common/history/` have `keyword = { effect... }` as top-level blocks,
/// where the effects from the same keywords are all added together. The keywords seem to be arbitrary, except for GLOBAL which
/// is documented to go last.

#[derive(Clone, Debug, Default)]
pub struct History {
    history: TigerHashMap<&'static str, HistoryEffect>,
}

impl History {
    fn load_item(&mut self, key: Token, mut block: Block) {
        if let Some(entry) = self.history.get_mut(key.as_str()) {
            entry.block.append(&mut block);
        } else {
            self.history.insert(key.as_str(), HistoryEffect::new(key, block));
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.history.values() {
            registry.scan(&item.block);
        }
    }

    pub fn validate(&self, data: &Everything) {
        for name in HISTORY_SEQUENCE {
            if let Some(item) = self.history.get(name) {
                item.validate(data);
            }
        }

        // Validate the remaining ones even if we don't know about them.
        // They may be for a newer game version.
        for (name, item) in &self.history {
            if HISTORY_SEQUENCE.contains(name) {
                continue;
            }
            if *name == "CONSCRIPTION" {
                let msg = "CONSCRIPTION history is not processed by the game";
                let info = "as of 1.9.7";
                warn(ErrorKey::Bugs).msg(msg).info(info).loc(&item.key).push();
                continue;
            }
            let msg = format!("unknown history classification `{name}`");
            err(ErrorKey::UnknownField).msg(msg).loc(&item.key).push();
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for History {
    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/history/")
    }

    fn load_file(&self, entry: &Arc<FileEntry>, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &Arc<FileEntry>, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Clone, Debug)]
pub struct HistoryEffect {
    key: Token,
    block: Block,
}

impl HistoryEffect {
    pub fn new(key: Token, block: Block) -> Self {
        Self { key, block }
    }

    pub fn validate(&self, data: &Everything) {
        let mut sc = ScopeContext::new(Scopes::None, &self.key);
        validate_effect(&self.block, data, &mut sc, Tooltipped::No);
    }
}

/// The order in which history files are processed by the game engine.
/// It was determined by adding `debug_log` entries to the history files.
/// LAST UPDATED VIC3 VERSION 1.9.7
const HISTORY_SEQUENCE: &[&str] = &[
    "STATES",
    "COUNTRIES",
    "POPS",
    "DIPLOMACY",
    "POPULATION",
    "POWER_BLOCS",
    "INTERESTS",
    "BUILDINGS",
    "PRODUCTION_METHODS",
    "MILITARY_FORMATIONS",
    "AI",
    "TREATIES",
    "LOBBIES",
    "DIPLOMATIC_PLAYS",
    "TRADE",
    // GLOBAL is documented to go last, but it doesn't.
    "GLOBAL",
    "POLITICAL_MOVEMENTS",
    "CHARACTERS",
    "GOVERNMENT",
    "GOVERNMENT_SETUP",
    "MILITARY_DEPLOYMENTS",
];
