use std::fmt::Debug;
use std::path::PathBuf;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_effect_internal;
use crate::everything::Everything;
use crate::fileset::{FileEntry, FileHandler};
#[cfg(feature = "hoi4")]
use crate::game::Game;
use crate::helpers::{BANNED_NAMES, TigerHashMap, limited_item_prefix_should_insert};
use crate::item::Item;
use crate::lowercase::Lowercase;
use crate::macros::{MACRO_MAP, MacroCache};
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
use crate::report::{ErrorKey, err, warn};
use crate::scopes::Scopes;
use crate::special_tokens::SpecialTokens;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::ListType;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Debug, Default)]
pub struct Effects {
    scope_overrides: TigerHashMap<&'static str, Scopes>,
    effects: TigerHashMap<&'static str, Effect>,
}

impl Effects {
    fn load_item(&mut self, key: Token, block: Block) {
        if BANNED_NAMES.contains(&key.as_str()) {
            let msg = "scripted effect has the same name as an important builtin";
            err(ErrorKey::NameConflict).strong().msg(msg).loc(key).push();
        } else if let Some(name) =
            limited_item_prefix_should_insert(Item::ScriptedEffect, key, |key| {
                self.effects.get(key).map(|entry| &entry.key)
            })
        {
            let scope_override = self.scope_overrides.get(name.as_str()).copied();
            if block.source.is_some() {
                MACRO_MAP.insert_or_get_loc(name.loc);
            }
            self.effects.insert(name.as_str(), Effect::new(name, block, scope_override));
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.effects.values() {
            registry.scan(&item.block);
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.effects.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        self.effects.values().map(|item| &item.key)
    }

    pub fn get(&self, key: &str) -> Option<&Effect> {
        self.effects.get(key)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.effects.values() {
            item.validate(data);
        }
    }
}

impl FileHandler<Block> for Effects {
    fn config(&mut self, config: &Block) {
        if let Some(block) = config.get_field_block("scope_override") {
            for (key, token) in block.iter_assignments() {
                let mut scopes = Scopes::empty();
                if token.lowercase_is("all") {
                    scopes = Scopes::all();
                } else {
                    for part in token.split('|') {
                        if let Some(scope) = Scopes::from_snake_case(part.as_str()) {
                            scopes |= scope;
                        } else {
                            let msg = format!("unknown scope type `{part}`");
                            warn(ErrorKey::Config).msg(msg).loc(part).push();
                        }
                    }
                }
                self.scope_overrides.insert(key.as_str(), scopes);
            }
        }
    }

    fn subpath(&self) -> PathBuf {
        PathBuf::from("common/scripted_effects")
    }

    fn load_file(&self, entry: &FileEntry, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        #[cfg(feature = "hoi4")]
        if Game::is_hoi4() {
            return PdxFile::read_no_bom(entry, parser);
        }
        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &FileEntry, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            self.load_item(key, block);
        }
    }
}

#[derive(Debug)]
pub struct Effect {
    pub key: Token,
    pub block: Block,
    cache: MacroCache<(ScopeContext, SpecialTokens, bool)>,
    scope_override: Option<Scopes>,
}

impl Effect {
    pub fn new(key: Token, block: Block, scope_override: Option<Scopes>) -> Self {
        Self { key, block, cache: MacroCache::default(), scope_override }
    }

    pub fn validate(&self, data: &Everything) {
        if self.block.source.is_none() {
            let mut sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            sc.set_strict_scopes(false);
            if self.scope_override.is_some() {
                sc.set_no_warn(true);
            }
            self.validate_call(
                &self.key,
                data,
                &mut sc,
                Tooltipped::No,
                &mut SpecialTokens::none(),
            );
        }
    }

    pub fn validate_call(
        &self,
        key: &Token,
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: Tooltipped,
        special_tokens: &mut SpecialTokens,
    ) -> bool {
        let mut has_tooltip = false;
        if !self.cached_compat(key, &[], tooltipped, sc, data, special_tokens, &mut has_tooltip) {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            our_sc.set_strict_scopes(false);
            if self.scope_override.is_some() {
                our_sc.set_no_warn(true);
            }
            self.cache.insert(
                key,
                &[],
                tooltipped,
                false,
                (our_sc.clone(), SpecialTokens::none(), false),
            );
            let mut our_st = SpecialTokens::empty();
            let mut vd = Validator::new(&self.block, data);
            has_tooltip |= validate_effect_internal(
                Lowercase::empty(),
                ListType::None,
                &self.block,
                data,
                &mut our_sc,
                &mut vd,
                tooltipped,
                &mut our_st,
            );
            if let Some(scopes) = self.scope_override {
                our_sc = ScopeContext::new_unrooted(scopes, key);
                our_sc.set_strict_scopes(false);
            }
            sc.expect_compatibility(&our_sc, key, data);
            special_tokens.merge(&our_st);
            self.cache.insert(key, &[], tooltipped, false, (our_sc, our_st, has_tooltip));
        }
        has_tooltip && tooltipped.is_tooltipped()
    }

    pub fn macro_parms(&self) -> Vec<&'static str> {
        self.block.macro_parms()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cached_compat(
        &self,
        key: &Token,
        args: &[(&'static str, Token)],
        tooltipped: Tooltipped,
        sc: &mut ScopeContext,
        data: &Everything,
        special_tokens: &mut SpecialTokens,
        has_tooltip: &mut bool,
    ) -> bool {
        self.cache.perform(key, args, tooltipped, false, |(our_sc, our_st, ht)| {
            sc.expect_compatibility(our_sc, key, data);
            special_tokens.merge(our_st);
            *has_tooltip |= ht;
        })
    }

    pub fn validate_macro_expansion(
        &self,
        key: &Token,
        args: &[(&'static str, Token)],
        data: &Everything,
        sc: &mut ScopeContext,
        tooltipped: Tooltipped,
        special_tokens: &mut SpecialTokens,
    ) -> bool {
        let mut has_tooltip = false;
        // Every invocation is treated as different even if the args are the same,
        // because we want to point to the correct one when reporting errors.
        if !self.cached_compat(key, args, tooltipped, sc, data, special_tokens, &mut has_tooltip)
            && let Some(block) = self.block.expand_macro(args, key.loc, &data.parser.pdxfile)
        {
            let mut our_sc = ScopeContext::new_unrooted(Scopes::all(), &self.key);
            our_sc.set_strict_scopes(false);
            if self.scope_override.is_some() {
                our_sc.set_no_warn(true);
            }
            // Insert the dummy sc before continuing. That way, if we recurse, we'll hit
            // that dummy context instead of macro-expanding again.
            self.cache.insert(
                key,
                args,
                tooltipped,
                false,
                (our_sc.clone(), SpecialTokens::none(), false),
            );
            let mut our_st = SpecialTokens::empty();
            let mut vd = Validator::new(&block, data);
            has_tooltip |= validate_effect_internal(
                Lowercase::empty(),
                ListType::None,
                &block,
                data,
                &mut our_sc,
                &mut vd,
                tooltipped,
                &mut our_st,
            );
            if let Some(scopes) = self.scope_override {
                our_sc = ScopeContext::new_unrooted(scopes, key);
                our_sc.set_strict_scopes(false);
            }

            sc.expect_compatibility(&our_sc, key, data);
            special_tokens.merge(&our_st);
            self.cache.insert(key, args, tooltipped, false, (our_sc, our_st, has_tooltip));
        }
        has_tooltip && tooltipped.is_tooltipped()
    }
}
