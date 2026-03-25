use crate::block::Block;
use crate::context::ScopeContext;
use crate::datacontext::DataContext;
use crate::datatype::{
    Code, CodeArg, CodeChain, Datatype, scope_from_datatype, validate_datatypes,
};
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::effect::validate_effect;
use crate::everything::Everything;
use crate::game::{Game, GameFlags};
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err, warn};
use crate::scopes::Scopes;
use crate::script_value::validate_non_dynamic_script_value;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::trigger::validate_trigger;
use crate::validate::validate_modifiers_with_base;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct ScriptedGui {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::jomini(), Item::ScriptedGui, ScriptedGui::add)
}

impl ScriptedGui {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::ScriptedGui, key, block, Box::new(Self {}));
    }
}

impl DbKind for ScriptedGui {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::None, key);
        if let Some(token) = vd.field_value("scope") {
            if let Some(scope) = Scopes::from_snake_case(token.as_str()) {
                sc = ScopeContext::new(scope, token);
            } else {
                warn(ErrorKey::Scopes).msg("unknown scope type").loc(token).push();
            }
        }

        // TODO: JominiNotification
        vd.field_value("notification_key");
        vd.field_validated_sc("confirm_title", &mut sc.clone(), validate_desc);
        vd.field_validated_sc("confirm_text", &mut sc.clone(), validate_desc);
        vd.field_trigger("ai_is_valid", Tooltipped::No, &mut sc.clone());
        vd.field_validated_block_sc("ai_chance", &mut sc.clone(), validate_modifiers_with_base);
        vd.field_validated("ai_frequency", validate_non_dynamic_script_value);

        vd.field_validated_list("saved_scopes", |token, _| {
            sc.define_name(token.as_str(), Scopes::all_but_none(), token);
        });
        // validate_guicall() will evaluate these with strict scopes.
        sc.set_strict_scopes(false);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc.clone());
        vd.field_trigger("is_valid", Tooltipped::No, &mut sc.clone());
        vd.field_effect("effect", Tooltipped::No, &mut sc.clone());
    }
}

const KNOWN_SGUICALLS: &[&str] = &[
    "BuildTooltip",
    "Execute",
    "ExecuteTooltip",
    "IsValid",
    "IsValidTooltip",
    "IsShown",
    "IsShownTooltip",
];

impl ScriptedGui {
    #[allow(clippy::unused_self)] // self is unused but don't want that in the API
    pub fn validate_guicall(
        &self,
        key: &Token,
        block: &Block,
        data: &Everything,
        context_sc: &mut ScopeContext,
        dc: &DataContext,
        code: &Code,
    ) {
        if !KNOWN_SGUICALLS.contains(&code.name.as_str()) || code.arguments.len() != 1 {
            return;
        }
        if let CodeArg::Chain(chain) = &code.arguments[0] {
            if chain.codes.len() < 2 {
                warn(ErrorKey::Gui)
                    .msg("expected GuiScope.SetRoot in argument")
                    .loc(&code.name)
                    .push();
                return;
            }

            let ghw = Game::is_ck3()
                && chain.codes[0].name.is("GreatHolyWarWindow")
                && chain.codes[1].name.is("GetScope");

            if !ghw {
                if !chain.codes[0].name.is("GuiScope") {
                    warn(ErrorKey::Gui).msg("expected GuiScope").loc(&chain.codes[0].name).push();
                    return;
                }
                if !chain.codes[1].name.is("SetRoot") {
                    warn(ErrorKey::Gui).msg("expected SetRoot").loc(&chain.codes[1].name).push();
                    return;
                }
                if chain.codes[1].arguments.len() != 1 {
                    // The caller already warns about this
                    return;
                }
            }
            // Get the root scope
            let scope = if ghw {
                Scopes::Character
            } else if let CodeArg::Chain(chain) = &chain.codes[1].arguments[0] {
                deduce_scope(chain, data, context_sc, dc)
            } else {
                // TODO: caller will warn about this once argument type is filled in
                return;
            };
            // Compare it to the declared root scope of the scripted gui
            if let Some(token) = block.get_field_value("scope")
                && let Some(declared_scope) = Scopes::from_snake_case(token.as_str())
                && !scope.intersects(declared_scope)
            {
                warn(ErrorKey::Scopes)
                    .msg("SetRoot scope does not match scripted gui scope")
                    .loc(&chain.codes[1].name)
                    .loc_msg(token, "scripted gui scope here")
                    .push();
            }
            let mut sc = ScopeContext::new(scope, &code.name);
            if ghw {
                #[cfg(feature = "ck3")]
                sc.define_name("great_holy_war", Scopes::GreatHolyWar, &chain.codes[0].name);
            }

            // Get the additional scopes
            for code in chain.codes.iter().skip(2) {
                if code.name.is("AddScope") {
                    if code.arguments.len() != 2 {
                        // The caller already warns about this
                        return;
                    }
                    let scope = if let CodeArg::Chain(chain) = &code.arguments[1] {
                        deduce_scope(chain, data, context_sc, dc)
                    } else {
                        Scopes::all()
                    };
                    match &code.arguments[0] {
                        CodeArg::Literal(name) => sc.define_name(name.as_str(), scope, name),
                        CodeArg::Chain(_) => sc.set_strict_scopes(false),
                    }
                } else if !code.name.is("End") {
                    warn(ErrorKey::Gui).msg("expected AddScope or End").loc(&code.name).push();
                    return;
                }
            }
            match code.name.as_str() {
                "BuildTooltip" => {
                    if let Some(block) = block.get_field_block("is_valid") {
                        validate_trigger(block, data, &mut sc.clone(), Tooltipped::Yes);
                    }
                    if let Some(block) = block.get_field_block("effect") {
                        validate_effect(block, data, &mut sc, Tooltipped::Yes);
                    }
                }
                "Execute" => {
                    if let Some(block) = block.get_field_block("effect") {
                        validate_effect(block, data, &mut sc, Tooltipped::No);
                    } else {
                        err(ErrorKey::Gui)
                            .msg(format!("scripted gui `{key}` has no effect block"))
                            .loc(&code.name)
                            .loc_msg(key, "scripted gui here")
                            .push();
                    }
                }
                "ExecuteTooltip" => {
                    if let Some(block) = block.get_field_block("effect") {
                        validate_effect(block, data, &mut sc, Tooltipped::Yes);
                    } else {
                        warn(ErrorKey::Gui)
                            .msg(format!("scripted gui `{key}` has no effect block"))
                            .loc(&code.name)
                            .loc_msg(key, "scripted gui here")
                            .push();
                    }
                }
                "IsShown" => {
                    if let Some(block) = block.get_field_block("is_shown") {
                        validate_trigger(block, data, &mut sc, Tooltipped::No);
                    }
                }
                "IsShownTooltip" => {
                    if let Some(block) = block.get_field_block("is_shown") {
                        validate_trigger(block, data, &mut sc, Tooltipped::Yes);
                    }
                }
                "IsValid" => {
                    if let Some(block) = block.get_field_block("is_valid") {
                        validate_trigger(block, data, &mut sc, Tooltipped::No);
                    }
                }
                "IsValidTooltip" => {
                    if let Some(block) = block.get_field_block("is_valid") {
                        validate_trigger(block, data, &mut sc, Tooltipped::Yes);
                    }
                }
                // Checked at the top of the function
                _ => unreachable!(),
            }
        }
    }
}

// TODO: handle MakeScopeValue calls
fn deduce_scope(
    chain: &CodeChain,
    data: &Everything,
    context_sc: &mut ScopeContext,
    dc: &DataContext,
) -> Scopes {
    // Deduce the scope type from the argument chain. It's made a bit
    // tricky by the MakeScope at the end, which transforms the actual
    // scope type into just a Datatype::Scope, so leave that off the
    // chain.
    if chain.codes.last().is_some_and(|code| code.name.is("MakeScope")) {
        let chain = chain.without_last();
        let rtype =
            validate_datatypes(&chain, data, context_sc, dc, Datatype::Unknown, None, None, true);
        scope_from_datatype(rtype).unwrap_or(Scopes::all())
    } else {
        Scopes::all()
    }
}
