use std::path::PathBuf;
use std::sync::Arc;

use crate::block::Block;
use crate::context::ScopeContext;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::files::{FileEntry, FileHandler};
use crate::game::Game;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::on_action::on_action_scopecontext;
use crate::parse::ParserMemory;
use crate::pdxfile::PdxFile;
#[cfg(feature = "ck3")]
use crate::report::warn;
#[cfg(any(feature = "ck3", feature = "hoi4"))]
use crate::report::{err, ErrorKey};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
#[cfg(feature = "jomini")]
use crate::validate::validate_duration;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;
use crate::variables::Variables;

#[derive(Clone, Debug, Default)]
pub struct OnActions {
    on_actions: TigerHashMap<&'static str, OnAction>,
}

impl OnActions {
    fn load_item(&mut self, key: Token, block: Block) {
        if let Some(other) = self.on_actions.get_mut(key.as_str()) {
            other.add(key, block);
        } else {
            self.on_actions.insert(key.as_str(), OnAction::new(key, block));
        }
    }

    pub fn scan_variables(&self, registry: &mut Variables) {
        for item in self.on_actions.values() {
            for (_, block) in &item.actions {
                registry.scan(block);
            }
        }
    }

    pub fn exists(&self, key: &str) -> bool {
        self.on_actions.contains_key(key)
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = &Token> {
        // SAFETY: The item's constructor guarantees at least one element in `actions`.
        self.on_actions.values().map(|item| &item.actions[0].0)
    }

    pub fn validate(&self, data: &Everything) {
        for item in self.on_actions.values() {
            item.validate(data);
        }
    }

    pub fn validate_call(&self, key: &Token, data: &Everything, sc: &mut ScopeContext) {
        if let Some(action) = self.on_actions.get(key.as_str()) {
            action.validate_call(data, sc);
        }
    }
}

impl FileHandler<Block> for OnActions {
    fn subpath(&self) -> PathBuf {
        match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => PathBuf::from("common/on_action"),
            #[cfg(feature = "imperator")]
            Game::Imperator => PathBuf::from("common/on_action"),
            #[cfg(feature = "vic3")]
            Game::Vic3 => PathBuf::from("common/on_actions"),
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => PathBuf::from("common/on_actions"),
        }
    }

    fn load_file(&self, entry: &Arc<FileEntry>, parser: &ParserMemory) -> Option<Block> {
        if !entry.filename().to_string_lossy().ends_with(".txt") {
            return None;
        }

        #[cfg(feature = "hoi4")]
        if Game::is_hoi4() {
            return PdxFile::read_no_bom(entry, parser);
        }
        PdxFile::read(entry, parser)
    }

    fn handle_file(&mut self, _entry: &Arc<FileEntry>, mut block: Block) {
        for (key, block) in block.drain_definitions_warn() {
            if Game::is_hoi4() {
                #[cfg(feature = "hoi4")]
                let mut block = block;
                #[cfg(feature = "hoi4")]
                if key.is("on_actions") {
                    for (key, block) in block.drain_definitions_warn() {
                        self.load_item(key, block);
                    }
                } else {
                    let msg = "unexpected key";
                    let info = "expected only `on_actions` here";
                    err(ErrorKey::UnknownField).msg(msg).info(info).loc(key).push();
                }
            } else {
                self.load_item(key, block);
            }
        }
    }
}

#[derive(Clone, Debug)]
/// Actions override in a special way, which is why this struct contains all actions defined under
/// the same name, rather than each new one replacing the prior one.
pub struct OnAction {
    actions: Vec<(Token, Block)>,
}

impl OnAction {
    pub fn new(key: Token, block: Block) -> Self {
        Self { actions: vec![(key, block)] }
    }

    pub fn add(&mut self, key: Token, block: Block) {
        self.actions.push((key, block));
    }

    pub fn validate(&self, data: &Everything) {
        let mut seen_trigger = false;
        let mut seen_effect = false;
        for (key, block) in self.actions.iter().rev() {
            // Make an sc for each array entry, to make sure it uses the local `key`.
            // This is important to distinguish between vanilla errors and mod errors.
            let mut sc = if let Some(builtin_sc) = on_action_scopecontext(key, data) {
                builtin_sc
            } else {
                let mut generated_sc = ScopeContext::new(Scopes::non_primitive(), key);
                generated_sc.set_strict_scopes(false);
                generated_sc
            };
            validate_on_action_internal(block, data, &mut sc, &mut seen_trigger, &mut seen_effect);
        }
    }

    /// Revalidate an `on_action` under the given scope context.
    /// It is not necessary to validate anything non scope related,
    /// but in this case it is easier to just call the full function because
    /// everything in an action is scope related.
    pub fn validate_call(&self, data: &Everything, sc: &mut ScopeContext) {
        let mut seen_trigger = false;
        let mut seen_effect = false;
        for (_, block) in self.actions.iter().rev() {
            validate_on_action_internal(block, data, sc, &mut seen_trigger, &mut seen_effect);
        }
    }
}

fn validate_on_action_internal(
    block: &Block,
    data: &Everything,
    sc: &mut ScopeContext,
    seen_trigger: &mut bool,
    seen_effect: &mut bool,
) {
    let mut vd = Validator::new(block, data);
    if Game::is_jomini() {
        vd.field_validated_block("trigger", |block, data| {
            if !*seen_trigger {
                *seen_trigger = true;
                validate_trigger(block, data, sc, Tooltipped::No);
            }
        });
        vd.field_validated_block_sc("weight_multiplier", sc, validate_modifiers_with_base);
    }

    if Game::is_hoi4() {
        // Hoi4 allows defining an effect in each entry.
        vd.multi_field_validated_block("effect", |block, data| {
            *seen_effect = true;
            validate_effect(block, data, sc, Tooltipped::No);
        });
    } else {
        vd.field_validated_block("effect", |block, data| {
            if !*seen_effect {
                *seen_effect = true;
                validate_effect(block, data, sc, Tooltipped::No);
            }
        });
    }

    // TODO: multiple random_events blocks in one on_action aren't outright bugged on Vic3,
    // but they might still get merged together into one big event pool. Verify.

    let mut count = 0;
    if Game::is_jomini() {
        #[allow(unused_variables)] // vic3 doesn't use `key`
        vd.multi_field_validated_key_block("events", |key, b, data| {
            let mut vd = Validator::new(b, data);
            #[cfg(feature = "jomini")]
            if Game::is_jomini() {
                vd.multi_field_validated_block_sc("delay", sc, validate_duration);
            }
            for token in vd.values() {
                data.verify_exists(Item::Event, token);
                data.event_check_scope(token, sc);
                if let Some(mut event_sc) = sc.root_for_event(token) {
                    data.event_validate_call(token, &mut event_sc);
                }
            }
            count += 1;
            #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
            if Game::is_ck3() && count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try combining them into one block";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
            }
        });
    }
    count = 0;
    #[allow(unused_variables)] // vic3 doesn't use `key`
    vd.multi_field_validated_key_block("random_events", |key, b, data| {
        let mut vd = Validator::new(b, data);
        #[cfg(feature = "jomini")]
        if Game::is_jomini() {
            vd.field_numeric("chance_to_happen"); // TODO: 0 - 100
            vd.field_script_value("chance_of_no_event", sc);
            vd.multi_field_validated_block_sc("delay", sc, validate_duration); // undocumented
        }
        for (_key, token) in vd.integer_values() {
            if token.is("0") {
                continue;
            }
            data.verify_exists(Item::Event, token);
            data.event_check_scope(token, sc);
            // Hoi4 uses the scope context directly.
            if Game::is_hoi4() {
                data.event_validate_call(token, sc);
            } else if let Some(mut event_sc) = sc.root_for_event(token) {
                data.event_validate_call(token, &mut event_sc);
            }
        }
        count += 1;
        #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
        if Game::is_ck3() && count == 2 {
            let msg = format!("multiple `{key}` blocks in one on_action do not work");
            let info = "try putting each into its own on_action and firing those separately";
            err(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
        }
    });
    if Game::is_jomini() {
        count = 0;
        #[allow(unused_variables)] // vic3 doesn't use `key`
        vd.multi_field_validated_key_block("first_valid", |key, b, data| {
            let mut vd = Validator::new(b, data);
            for token in vd.values() {
                data.verify_exists(Item::Event, token);
                data.event_check_scope(token, sc);
                if let Some(mut event_sc) = sc.root_for_event(token) {
                    data.event_validate_call(token, &mut event_sc);
                }
            }
            count += 1;
            #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
            if Game::is_ck3() && count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try putting each into its own on_action and firing those separately";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
            }
        });
        count = 0;
        #[allow(unused_variables)] // vic3 doesn't use `key`
        vd.multi_field_validated_key_block("on_actions", |key, b, data| {
            let mut vd = Validator::new(b, data);
            #[cfg(feature = "jomini")]
            if Game::is_jomini() {
                vd.multi_field_validated_block_sc("delay", sc, validate_duration);
            }
            for token in vd.values() {
                data.verify_exists(Item::OnAction, token);
                if let Some(mut action_sc) = sc.root_for_action(token) {
                    data.on_actions.validate_call(token, data, &mut action_sc);
                }
            }
            count += 1;
            #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
            if Game::is_ck3() && count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try combining them into one block";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
            }
        });
        count = 0;
        #[allow(unused_variables)] // vic3 doesn't use `key`
        vd.multi_field_validated_key_block("random_on_action", |key, b, data| {
            let mut vd = Validator::new(b, data);
            #[cfg(feature = "jomini")]
            if Game::is_jomini() {
                vd.field_numeric("chance_to_happen"); // TODO: 0 - 100
                vd.field_script_value("chance_of_no_event", sc);
            }
            for (_key, token) in vd.integer_values() {
                if token.is("0") {
                    continue;
                }
                data.verify_exists(Item::OnAction, token);
                if let Some(mut action_sc) = sc.root_for_action(token) {
                    data.on_actions.validate_call(token, data, &mut action_sc);
                }
            }
            count += 1;
            #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
            if Game::is_ck3() && count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try putting each into its own on_action and firing those separately";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
            }
        });
        count = 0;
        #[allow(unused_variables)] // vic3 doesn't use `key`
        vd.multi_field_validated_key_block("first_valid_on_action", |key, b, data| {
            let mut vd = Validator::new(b, data);
            for token in vd.values() {
                data.verify_exists(Item::OnAction, token);
                if let Some(mut action_sc) = sc.root_for_action(token) {
                    data.on_actions.validate_call(token, data, &mut action_sc);
                }
            }
            count += 1;
            #[cfg(feature = "ck3")] // Verified: this is only a problem in CK3
            if Game::is_ck3() && count == 2 {
                // TODO: verify
                let msg = format!("not sure if multiple `{key}` blocks in one on_action work");
                let info = "try putting each into its own on_action and firing those separately";
                warn(ErrorKey::Validation).msg(msg).info(info).loc(key).push();
            }
        });
        // TODO: check for infinite fallback loops?
        vd.field_action("fallback", sc);
    }
}

#[cfg(feature = "vic3")]
pub fn validate_on_action(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut seen_trigger = false;
    let mut seen_effect = false;
    validate_on_action_internal(block, data, sc, &mut seen_trigger, &mut seen_effect);
}
