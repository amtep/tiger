use crate::block::{BV, Block, Field};
use crate::ck3::validate::validate_cost;
use crate::context::ScopeContext;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err, fatal, warn};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validate::{validate_duration, validate_modifiers_with_base};
use crate::validator::{Validator, ValueValidator};

#[derive(Clone, Debug)]
pub struct Decision {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::Decision, Decision::add)
}

impl Decision {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::Decision, key, block, Box::new(Self {}));
    }
}

impl DbKind for Decision {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);
        let mut sc = ScopeContext::new(Scopes::Character, key);

        // Add builtin scopes added by known widget controllers.
        // The controller is either `widget = { controller = ... }` or the short `widget = ...`.
        let widget = block.get_field_block("widget");
        let controller = match widget {
            Some(widget) => widget.get_field_value("controller"),
            None => block.get_field_value("widget"),
        };
        if let Some(controller) = controller {
            if controller.is("decision_option_list_controller") {
                for block in widget.map(|w| w.get_field_blocks("item")).unwrap_or_default() {
                    if let Some(token) = block.get_field_value("value") {
                        sc.define_name(token.as_str(), Scopes::Bool, token);
                    }
                }
            } else if controller.is("create_holy_order") || controller.is("revoke_holy_order_lease")
            {
                // The docs give `create_holy_order` the same data context as
                // `revoke_holy_order_lease`
                sc.define_name("barony", Scopes::LandedTitle, controller);
            } else if controller.is("select_scope_object") {
                sc.define_name("selected_item", Scopes::all_but_none(), controller);
            } else if SELECT_TITLE_CONTROLLERS.contains(&controller.as_str()) {
                sc.define_name("title", Scopes::LandedTitle, controller);
            }
        }

        vd.req_field_warn("picture");
        vd.multi_field_validated_block("picture", |block, data| {
            let mut vd = Validator::new(block, data);
            vd.field_trigger_rooted("trigger", Tooltipped::No, Scopes::Character);
            vd.field_item("reference", Item::File);
            vd.field_item("soundeffect", Item::Sound);
        });
        vd.field_item("extra_picture", Item::File);
        vd.advice_field("major", "Replaced with decision_group_type");
        vd.field_item("decision_group_type", Item::DecisionGroup);
        vd.field_script_value_rooted("progress", Scopes::Character);
        vd.field_validated_block("advice", validate_advice);
        vd.field_integer("sort_order");
        vd.field_bool("is_invisible");
        vd.field_bool("ai_goal");
        vd.field_integer("ai_check_interval");
        vd.field_validated_key_block("ai_check_interval_by_tier", |key, b, data| {
            let mut vd = Validator::new(b, data);
            for tier in &["barony", "county", "duchy", "kingdom", "empire", "hegemony"] {
                vd.req_field(tier);
                vd.field_integer(tier);
            }
            if block.has_key("ai_check_interval") {
                let msg = "must not have both `ai_check_interval` and `ai_check_interval_by_tier`";
                warn(ErrorKey::Validation).msg(msg).loc(key).push();
            }
        });
        if block.get_field_bool("ai_goal").unwrap_or(false) {
            vd.advice_field("ai_check_interval", "not needed if ai_goal = yes");
        } else if !block.has_key("ai_check_interval") && !block.has_key("ai_check_interval_by_tier")
        {
            let msg = "decision needs `ai_check_interval` or `ai_check_interval_by_tier`";
            let info = "unless it has `ai_goal = yes`";
            err(ErrorKey::FieldMissing).msg(msg).info(info).loc(key).push();
        }
        vd.field_validated_block_sc("cooldown", &mut sc, validate_duration);

        vd.field_item("confirm_click_sound", Item::Sound);

        if !vd.field_validated_sc("selection_tooltip", &mut sc, validate_desc) {
            let loca = format!("{key}_tooltip");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.validate_localization_sc(&loca, &mut sc);
        }

        if !vd.field_validated_sc("title", &mut sc, validate_desc) {
            data.verify_exists(Item::Localization, key);
            data.validate_localization_sc(key.as_str(), &mut sc);
        }

        if !vd.field_validated_sc("desc", &mut sc, validate_desc) {
            let loca = format!("{key}_desc");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.validate_localization_sc(&loca, &mut sc);
        }

        if !vd.field_validated_sc("confirm_text", &mut sc, validate_desc) {
            let loca = format!("{key}_confirm");
            data.verify_exists_implied(Item::Localization, &loca, key);
            data.validate_localization_sc(&loca, &mut sc);
        }

        // The decision docs give `scope:puppeteer` to the triggers and the cost, and the puppet
        // action docs to the effect too, for a ruler taking the decision through root as their
        // puppet
        let mut sc_puppet = sc.clone();
        sc_puppet.define_name("puppeteer", Scopes::Character, key);
        vd.field_trigger("is_shown", Tooltipped::No, &mut sc_puppet);
        vd.field_trigger(
            "is_valid_showing_failures_only",
            Tooltipped::FailuresOnly,
            &mut sc_puppet,
        );
        vd.field_trigger("is_valid", Tooltipped::Yes, &mut sc_puppet);

        // cost can have multiple definitions and they will be combined
        // however, two costs of the same type are not summed
        vd.multi_field_validated_block("cost", |b, data| validate_cost(b, data, &mut sc_puppet));
        check_cost(&block.get_field_blocks("cost"));
        vd.multi_field_validated_block("minimum_cost", |b, data| {
            validate_cost(b, data, &mut sc_puppet);
        });
        check_cost(&block.get_field_blocks("minimum_cost"));

        vd.field_effect("effect", Tooltipped::Yes, &mut sc_puppet);
        vd.field_trigger("ai_potential", Tooltipped::No, &mut sc);
        vd.field_validated_block_sc("ai_will_do", &mut sc, validate_modifiers_with_base);
        vd.field_trigger("should_create_alert", Tooltipped::No, &mut sc);
        // The docs allow "up to one custom widget"; a second one crashes the game (seen in 1.20)
        let mut widgets = block.iter_fields().filter(|Field(key, _, _)| key.is("widget"));
        if let Some(Field(first, _, _)) = widgets.next() {
            for Field(other, _, _) in widgets {
                let msg = "a decision can have only one widget";
                let info = "a second widget crashes the game";
                fatal(ErrorKey::Crash)
                    .strong()
                    .msg(msg)
                    .info(info)
                    .loc(other)
                    .loc_msg(first, "the first one")
                    .push();
            }
        }
        vd.multi_field_validated_key("widget", |key, bv, data| {
            match bv {
                BV::Value(value) => {
                    let mut vvd = ValueValidator::new(value, data);
                    vvd.choice(CONTROLLERS);
                    if CONTROLLERS.contains(&value.as_str()) {
                        verify_default_widget_gui(value.as_str(), value, data);
                    }
                }
                BV::Block(block) => {
                    let mut vd = Validator::new(block, data);
                    vd.field_validated_value("gui", |key, mut vd| {
                        let filename = format!("gui/decision_view_widgets/{vd}.gui");
                        data.verify_exists_implied(Item::File, &filename, key);
                        vd.accept();
                    });
                    if !block.has_key("gui") {
                        match block.get_field_value("controller") {
                            None => verify_default_widget_gui("default", key, data),
                            Some(controller) if CONTROLLERS.contains(&controller.as_str()) => {
                                verify_default_widget_gui(controller.as_str(), controller, data);
                            }
                            Some(_) => (),
                        }
                    }

                    vd.field_choice("controller", CONTROLLERS);
                    vd.field_bool("show_from_start");
                    // The docs say this must be set if `show_from_start = yes`
                    if block.field_value_is("show_from_start", "yes") {
                        vd.req_field("decision_to_second_step_button");
                    }
                    vd.field_item("decision_to_second_step_button", Item::Localization);

                    match block.get_field_value("controller").map(Token::as_str) {
                        Some("decision_option_list_controller") => {
                            vd.multi_field_validated_block("item", |block, data| {
                                let mut vd = Validator::new(block, data);
                                vd.field_value("value");
                                vd.field_trigger_rooted(
                                    "is_shown",
                                    Tooltipped::No,
                                    Scopes::Character,
                                );
                                vd.field_trigger_rooted(
                                    "is_valid",
                                    Tooltipped::FailuresOnly,
                                    Scopes::Character,
                                );
                                vd.field_validated_rooted(
                                    "current_description",
                                    Scopes::Character,
                                    validate_desc,
                                );
                                vd.field_validated_rooted(
                                    "localization",
                                    Scopes::Character,
                                    validate_desc,
                                );
                                vd.field_bool("is_default");
                                vd.field_item("icon", Item::File);
                                vd.field_bool("flat");

                                vd.field_script_value_no_breakdown_rooted(
                                    "ai_chance",
                                    Scopes::Character,
                                );
                            });
                        }
                        Some(controller)
                            if controller == "create_holy_order"
                                || controller == "revoke_holy_order_lease"
                                || SELECT_TITLE_CONTROLLERS.contains(&controller) =>
                        {
                            vd.replaced_field("barony_valid", "title_valid");
                            vd.field_trigger_builder("title_valid", Tooltipped::No, |key| {
                                let mut sc = ScopeContext::new(Scopes::LandedTitle, key);
                                sc.define_name("ruler", Scopes::Character, key);
                                sc
                            });
                        }
                        Some("select_scope_object") => {
                            let none_sc = |key: &Token| {
                                let mut sc = ScopeContext::new(Scopes::None, key);
                                sc.define_name("actor", Scopes::Character, key);
                                sc
                            };
                            vd.field_effect_builder("setup_items", Tooltipped::No, none_sc);
                            vd.field_effect_builder("default_item", Tooltipped::No, none_sc);
                            vd.field_effect_builder("ai_select_item", Tooltipped::No, none_sc);
                            let item_sc = |key: &Token| {
                                let mut sc = ScopeContext::new(Scopes::all_but_none(), key);
                                sc.define_name("actor", Scopes::Character, key);
                                sc
                            };
                            vd.field_script_value_no_breakdown_builder("ai_item_will_do", item_sc);
                            // The docs say invalid items get a tooltip explaining why
                            vd.field_trigger_builder(
                                "is_item_valid",
                                Tooltipped::FailuresOnly,
                                item_sc,
                            );
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}

// LAST UPDATED CK3 VERSION 1.20.1
const CONTROLLERS: &[&str] = &[
    "default",
    "decision_option_list_controller",
    "create_holy_order",
    "revoke_holy_order_lease",
    "select_scope_object",
    "select_barony_title",
    "select_county_title",
    "select_duchy_title",
    "select_kingdom_title",
    "select_empire_title",
    "select_hegemony_title",
    "select_barony_title_in_realm",
    "select_county_title_in_realm",
    "select_duchy_title_in_realm",
    "select_kingdom_title_in_realm",
    "select_empire_title_in_realm",
    "select_hegemony_title_in_realm",
];

// LAST UPDATED CK3 VERSION 1.20.1
const SELECT_TITLE_CONTROLLERS: &[&str] = &[
    "select_barony_title",
    "select_county_title",
    "select_duchy_title",
    "select_kingdom_title",
    "select_empire_title",
    "select_hegemony_title",
    "select_barony_title_in_realm",
    "select_county_title_in_realm",
    "select_duchy_title_in_realm",
    "select_kingdom_title_in_realm",
    "select_empire_title_in_realm",
    "select_hegemony_title_in_realm",
];

/// Check the gui that a widget uses when it doesn't set `gui`
fn verify_default_widget_gui(controller: &str, token: &Token, data: &Everything) {
    // The docs say these controllers share a gui
    let gui = if SELECT_TITLE_CONTROLLERS.contains(&controller) {
        "select_title"
    } else if controller == "revoke_holy_order_lease" {
        "create_holy_order"
    } else {
        controller
    };
    let filename = format!("gui/decision_view_widgets/decision_view_widget_{gui}.gui");
    data.verify_exists_implied(Item::File, &filename, token);
}

fn check_cost(blocks: &[&Block]) {
    let mut seen_gold = false;
    let mut seen_prestige = false;
    let mut seen_piety = false;
    if blocks.len() > 1 {
        for cost in blocks {
            if let Some(gold) = cost.get_field("gold") {
                if seen_gold {
                    let msg = "This value of the gold cost overrides the previously set cost.";
                    warn(ErrorKey::Conflict).msg(msg).loc(gold).push();
                }
                seen_gold = true;
            }
            if let Some(prestige) = cost.get_field("prestige") {
                if seen_prestige {
                    let msg = "This value of the prestige cost overrides the previously set cost.";
                    warn(ErrorKey::Conflict).msg(msg).loc(prestige).push();
                }
                seen_prestige = true;
            }
            if let Some(piety) = cost.get_field("piety") {
                if seen_piety {
                    let msg = "This value of the piety cost overrides the previously set cost.";
                    warn(ErrorKey::Conflict).msg(msg).loc(piety).push();
                }
                seen_piety = true;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DecisionGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Ck3, Item::DecisionGroup, DecisionGroup::add)
}

impl DecisionGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::DecisionGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for DecisionGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        let loca = format!("decision_group_type_{key}");
        data.verify_exists_implied(Item::Localization, &loca, key);

        vd.field_integer("sort_order");
        vd.field_list("gui_tags");
        vd.field_bool("important_decision_group");
    }
}

fn validate_advice(block: &Block, data: &Everything) {
    let mut vd = Validator::new(block, data);
    vd.field_item("region", Item::Region);
    // The docs say this is a collection of hints
    vd.multi_field_validated_key_block("per_title_hint", |key, block, data| {
        let mut sc = ScopeContext::new(Scopes::Character, key);
        sc.define_name("title", Scopes::LandedTitle, key);
        let mut vd = Validator::new(block, data);
        vd.field_trigger("is_valid", Tooltipped::No, &mut sc);
        vd.field_trigger("is_doing", Tooltipped::No, &mut sc);
        // The docs say `doing` is the result of `is_doing`, for the fields after it
        sc.define_name("doing", Scopes::Bool, key);
        vd.field_script_value_no_breakdown("relevance", &mut sc);
        vd.field_validated_sc("summary", &mut sc, validate_desc);
        vd.field_validated_sc("description", &mut sc, validate_desc);
    });
}
