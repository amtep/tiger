use crate::block::Block;
use crate::context::ScopeContext;
use crate::data::on_actions::validate_on_action;
use crate::db::{Db, DbKind};
use crate::desc::validate_desc;
use crate::everything::Everything;
use crate::game::GameFlags;
use crate::item::{Item, ItemLoader};
use crate::report::{ErrorKey, err};
use crate::scopes::Scopes;
use crate::token::Token;
use crate::tooltipped::Tooltipped;
use crate::validator::Validator;

#[derive(Clone, Debug)]
pub struct JournalEntry {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::JournalEntry, JournalEntry::add)
}

impl JournalEntry {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::JournalEntry, key, block, Box::new(Self {}));
    }
}

impl DbKind for JournalEntry {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        let mut vd = Validator::new(block, data);

        data.verify_exists(Item::Localization, key);
        let loca = format!("{key}_reason");
        data.verify_exists_implied(Item::Localization, &loca, key);
        // TODO: make this depend on whether the journalentry uses the "goal" mechanic
        let loca = format!("{key}_goal");
        data.mark_used(Item::Localization, &loca);

        let sc_context = if let Some(group) = block.get_field_value("group") {
            if data.item_has_property(Item::JournalEntryGroup, group.as_str(), "none_context") {
                Scopes::None
            } else {
                Scopes::Country
            }
        } else {
            Scopes::Country
        };
        let mut sc = ScopeContext::new(sc_context, key);
        sc.define_name("journal_entry", Scopes::JournalEntry, key);
        sc.define_name("target", Scopes::all(), key);

        let mut country_sc = ScopeContext::new(Scopes::Country, key);
        country_sc.define_name("journal_entry", Scopes::JournalEntry, key);
        country_sc.define_name("target", Scopes::all(), key);

        vd.field_item("group", Item::JournalEntryGroup);

        vd.field_item("icon", Item::File);

        vd.field_trigger("is_shown_when_inactive", Tooltipped::No, &mut sc);
        vd.field_trigger("should_be_involved", Tooltipped::No, &mut country_sc);
        vd.field_trigger("should_show_when_not_involved", Tooltipped::No, &mut country_sc);

        vd.multi_field_item("scripted_button", Item::ScriptedButton);

        vd.field_trigger("possible", Tooltipped::Yes, &mut sc);
        vd.field_effect("immediate", Tooltipped::No, &mut sc);
        vd.field_effect("immediate_all_involved", Tooltipped::No, &mut country_sc);
        vd.field_trigger("complete", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_complete", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_complete_all_involved", Tooltipped::No, &mut country_sc);
        vd.field_trigger("fail", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_fail", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_fail_all_involved", Tooltipped::No, &mut country_sc);
        vd.field_trigger("invalid", Tooltipped::No, &mut sc);
        vd.field_effect("on_invalid", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_invalid_all_involved", Tooltipped::No, &mut country_sc);
        vd.field_effect("on_become_involved_after_activation", Tooltipped::No, &mut country_sc);
        vd.field_effect("on_no_longer_involved", Tooltipped::No, &mut country_sc);

        if !vd.field_validated_sc("status_desc", &mut sc, validate_desc) {
            data.mark_used(Item::Localization, &format!("{key}_status"));
        }

        vd.multi_field_validated_sc("event_outcome_activated_desc", &mut sc, validate_desc);
        vd.multi_field_validated_block_sc(
            "event_outcome_activated_effect_desc",
            &mut sc,
            validate_effect_desc,
        );
        vd.multi_field_validated_sc("event_outcome_invalidated_desc", &mut sc, validate_desc);
        vd.multi_field_validated_block_sc(
            "event_outcome_invalidated_effect_desc",
            &mut sc,
            validate_effect_desc,
        );
        vd.multi_field_validated_sc("event_outcome_completed_desc", &mut sc, validate_desc);
        vd.multi_field_validated_block_sc(
            "event_outcome_completed_effect_desc",
            &mut sc,
            validate_effect_desc,
        );
        vd.multi_field_validated_sc("event_outcome_failed_desc", &mut sc, validate_desc);
        vd.multi_field_validated_block_sc(
            "event_outcome_failed_effect_desc",
            &mut sc,
            validate_effect_desc,
        );
        vd.multi_field_validated_sc("event_outcome_timeout_desc", &mut sc, validate_desc);
        vd.multi_field_validated_block_sc(
            "event_outcome_timeout_effect_desc",
            &mut sc,
            validate_effect_desc,
        );
        vd.field_localization("custom_completion_header", &mut sc);
        vd.field_localization("custom_failure_header", &mut sc);
        vd.field_localization("custom_on_completion_header", &mut sc);
        vd.field_localization("custom_on_failure_header", &mut sc);

        vd.field_script_value("timeout", &mut sc);
        if let Some(token) = block.get_field_value("timeout") {
            if token.is("0") {
                let msg = "as of 1.9.5, a timeout of 0 will close the journal entry immediately";
                err(ErrorKey::Logic).msg(msg).loc(token).push();
            }
        }
        vd.field_effect("on_timeout", Tooltipped::Yes, &mut sc);
        vd.field_effect("on_timeout_all_involved", Tooltipped::No, &mut country_sc);

        vd.field_list_items("modifiers_while_active", Item::Modifier);

        for field in &["on_weekly_pulse", "on_monthly_pulse", "on_yearly_pulse"] {
            vd.multi_field_validated_block_sc(field, &mut sc, validate_on_action);
        }

        vd.field_script_value("current_value", &mut sc);
        vd.field_script_value("goal_add_value", &mut sc);
        vd.field_script_value("weight", &mut sc);

        vd.field_bool("transferable");
        vd.field_bool("can_revolution_inherit");

        vd.field_trigger("is_progressing", Tooltipped::No, &mut sc);
        vd.field_bool("progressbar");
        vd.multi_field_item("scripted_progress_bar", Item::ScriptedProgressBar);

        vd.field_bool("can_deactivate");

        if block.field_value_is("progressbar", "yes") {
            if !vd.field_validated_sc("progress_desc", &mut sc, validate_desc) {
                data.mark_used(Item::Localization, &format!("{key}_progress"));
            }
        } else {
            vd.ban_field("progress_desc", || "progressbar = yes");
        }

        vd.field_item("how_tutorial", Item::TutorialLesson);
        vd.field_item("why_tutorial", Item::TutorialLesson);

        vd.field_bool("should_be_pinned_by_default");

        // undocumented

        vd.field_integer("active_update_frequency");
        vd.field_bool("should_update_on_player_command");
        vd.field_bool("display_progressbar_as_months");
        vd.field_trigger("is_shown_in_lobby", Tooltipped::No, &mut sc);
    }
}

#[derive(Clone, Debug)]
pub struct JournalEntryGroup {}

inventory::submit! {
    ItemLoader::Normal(GameFlags::Vic3, Item::JournalEntryGroup, JournalEntryGroup::add)
}

impl JournalEntryGroup {
    pub fn add(db: &mut Db, key: Token, block: Block) {
        db.add(Item::JournalEntryGroup, key, block, Box::new(Self {}));
    }
}

impl DbKind for JournalEntryGroup {
    fn validate(&self, key: &Token, block: &Block, data: &Everything) {
        data.verify_exists(Item::Localization, key);

        let mut vd = Validator::new(block, data);
        vd.field_choice("context", &["none", "country"]);
    }

    fn has_property(
        &self,
        _key: &Token,
        block: &Block,
        property: &str,
        _data: &Everything,
    ) -> bool {
        if property == "none_context" {
            return block.get_field_value("context").is_some_and(|v| v.is("none"));
        }
        false
    }
}

fn validate_effect_desc(block: &Block, data: &Everything, sc: &mut ScopeContext) {
    let mut vd = Validator::new(block, data);

    vd.field_localization("header", sc);
    vd.field_effect("effect", Tooltipped::Yes, sc);
}
