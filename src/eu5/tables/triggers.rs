#![allow(unused_imports)] // TODO EU5: remove this when ready
use std::sync::LazyLock;

use crate::eu5::tables::misc::*;
use crate::everything::Everything;
use crate::helpers::TigerHashMap;
use crate::item::Item;
use crate::scopes::*;
use crate::token::Token;
use crate::trigger::Trigger;
use Trigger::*;

pub fn scope_trigger(name: &Token, _data: &Everything) -> Option<(Scopes, Trigger)> {
    let name_lc = name.as_str().to_ascii_lowercase();
    TRIGGER_MAP.get(&*name_lc).copied()
}

static TRIGGER_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, Trigger)>> = LazyLock::new(|| {
    let mut hash = TigerHashMap::default();
    for (from, s, trigger) in TRIGGER.iter().copied() {
        hash.insert(s, (from, trigger));
    }
    hash
});

/// See `triggers.log` from the game data dumps
/// A key ends with '(' if it is the version that takes a parenthesized argument in script.
const TRIGGER: &[(Scopes, &str, Trigger)] = &[
    // TODO: EU5 fill in the UncheckedTodo
    (Scopes::Country, "active_religious_focus", Item(Item::ReligiousFocus)),
    (
        Scopes::Country,
        "add_estate_satisfaction_utility",
        Block(&[("type", Item(Item::Estate)), ("amount", CompareValue)]),
    ),
    (
        Scopes::Location.union(Scopes::Country).union(Scopes::Character),
        "add_static_modifier_utility",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::all_but_none(), "add_to_temporary_list", Special),
    (Scopes::Location.union(Scopes::Area), "adjacent_to_owned_by", Scope(Scopes::Country)),
    (
        Scopes::Location.union(Scopes::Area),
        "adjacent_to_owned_or_owned_by_subject",
        Scope(Scopes::Country),
    ),
    (Scopes::Character, "adm", CompareValue),
    (Scopes::Country, "advance_no_longer_activated", Item(Item::Advance)),
    (Scopes::Character, "age_in_days", CompareValue),
    (Scopes::Character, "age_in_years", CompareValue),
    (Scopes::Country, "age_preference", Choice(FOCUS_TYPES)),
    (Scopes::ParliamentAgenda, "agenda_for_estate_type", Scope(Scopes::EstateType)),
    (Scopes::ParliamentAgenda, "agenda_for_special_status", Scope(Scopes::EstateType)),
    (
        Scopes::None,
        "ai_issue_voting_bias",
        Block(&[
            ("actor", Scope(Scopes::Country)),
            ("international_organization", Scope(Scopes::InternationalOrganization)),
            ("vote", Scope(Scopes::all_but_none())),
            ("value", CompareValue),
        ]),
    ),
    (
        Scopes::ParliamentIssue,
        "ai_parliament_issue_resolution_vote_bias",
        Block(&[
            ("actor", Scope(Scopes::Country)),
            ("international_organization", Scope(Scopes::InternationalOrganization)),
            ("value", CompareValue),
        ]),
    ),
    (
        Scopes::Policy,
        "ai_policy_reason_to_join",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Policy, "ai_policy_resolution_keep_bias", UncheckedTodo),
    (Scopes::Policy, "ai_policy_resolution_propose_bias", UncheckedTodo),
    (Scopes::Policy, "ai_policy_resolution_vote_bias", UncheckedTodo),
    (Scopes::Country, "ai_unlock_unit_score", UncheckedTodo),
    (Scopes::Religion, "ai_wants_convert", UncheckedTodo),
    (Scopes::None, "ai_will_do", CompareValue),
    (Scopes::None, "all_false", Control),
    (Scopes::Country, "allows_female_rulers", Boolean),
    (Scopes::Country, "allows_male_rulers", Boolean),
    (Scopes::None, "always", Boolean),
    (Scopes::EstateType, "always_loyal", Boolean),
    (Scopes::None, "and", Control),
    (
        Scopes::Country,
        "annexation_cost",
        Block(&[("target", Scope(Scopes::Country)), ("value", CompareValue)]),
    ),
    (
        Scopes::Country,
        "annexation_progress",
        Block(&[("target", Scope(Scopes::Country)), ("value", CompareValue)]),
    ),
    (
        Scopes::Country,
        "antagonism",
        Block(&[("target", Scope(Scopes::Country)), ("value", CompareValue)]),
    ),
    (Scopes::None, "any_false", UncheckedTodo),
    (Scopes::Area, "area_average_control", CompareValue),
    (Scopes::Area, "area_average_integration", CompareValue),
    (Scopes::Area, "area_exploration_progress", CompareValue),
    (Scopes::Country, "army_maintenance", CompareValue),
    (Scopes::Country, "army_size", CompareValue),
    (Scopes::Country, "army_size_percentage", CompareValue),
    (Scopes::Country.union(Scopes::InternationalOrganization), "army_tradition", CompareValue),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "army_tradition_percentage",
        CompareValue,
    ),
    (Scopes::Character, "art_progress", CompareValue),
    (Scopes::WorkOfArt, "art_quality", CompareValue),
    (Scopes::Character, "artist_skill", CompareValue),
    (Scopes::Character, "artist_type", CompareValue),
    (Scopes::None, "assert_if", Block(&[("limit", Control), ("?text", UncheckedValue)])),
    (Scopes::None, "assert_read", UncheckedValue),
    (Scopes::Country, "at_war", Boolean),
    (Scopes::Market, "available_merchant_capacity", CompareValue),
    (Scopes::Country, "average_control_in_home_region", CompareValue),
    (Scopes::Country, "average_country_literacy", CompareValue),
    (Scopes::Country, "average_estate_satisfaction", CompareValue),
    (Scopes::Location, "average_location_literacy", CompareValue),
    (Scopes::Location, "average_satisfaction", CompareValue),
    (
        Scopes::InternationalOrganization,
        "average_special_status_power",
        Block(&[("type", Item(Item::InternationalOrganization)), ("value", CompareValue)]),
    ),
    (Scopes::Siege, "besieger_strength", CompareValue),
    (
        Scopes::all(),
        "bias_value",
        Block(&[("modifier", Item(Item::Bias)), ("value", CompareValue)]),
    ),
    (Scopes::Character, "birth_age", Scope(Scopes::Age)),
    (Scopes::PeaceTreaty, "blocks_full_annexation", UncheckedTodo),
    (
        Scopes::Location.union(Scopes::Country).union(Scopes::Province),
        "border_distance_to",
        Block(&[("country", Scope(Scopes::Country)), ("value", CompareValue)]),
    ),
    (Scopes::Building, "building_can_be_destroyed_by", Scope(Scopes::Country)),
    (Scopes::Building, "building_can_be_upgraded_by", Scope(Scopes::Country)),
    (
        Scopes::Building.union(Scopes::BuildingType),
        "building_category",
        Item(Item::BuildingCategory),
    ),
    (Scopes::Location, "building_efficiency", CompareValue),
    (Scopes::Building, "building_employed_amount", CompareValue),
    (Scopes::Building, "building_employment_size_amount", CompareValue),
    (Scopes::Building, "building_goods_input", CompareValue),
    (Scopes::Building, "building_index", CompareValue),
    (Scopes::Building, "building_level", CompareValue),
    (Scopes::Building, "building_levels_under_construction", CompareValue),
    (Scopes::BuildingType, "building_manpower_produced", CompareValue),
    (Scopes::Building, "building_max_level", CompareValue),
    (Scopes::Building, "building_pop_type", Item(Item::PopType)),
    (Scopes::Building, "building_potential_profit", CompareValue),
    (Scopes::Building, "building_produced_goods", Scope(Scopes::Goods)),
    (Scopes::Building, "building_profit", CompareValue),
    (Scopes::BuildingType, "building_sailors_produced", CompareValue),
    (Scopes::Country, "building_type_is_obsolete", Scope(Scopes::BuildingType)),
    (
        Scopes::Location,
        "building_type_max_level",
        Block(&[
            ("building_type", Scope(Scopes::BuildingType)),
            ("?owner", Scope(Scopes::Country)),
            ("value", CompareValue),
        ]),
    ),
    (Scopes::CabinetAction, "cabinet_action_type", Choice(FOCUS_TYPES)),
    (Scopes::None, "calc_true_if", Control),
    (
        Scopes::None,
        "can_add_relation",
        Block(&[
            ("first", Scope(Scopes::BuildingType)),
            ("second", Scope(Scopes::Country)),
            ("type", Scope(Scopes::RelationType)),
        ]),
    ),
    (Scopes::InternationalOrganization, "can_annex_members", Scope(Scopes::Country)),
    (Scopes::ParliamentAgenda, "can_be_bribe", Boolean),
    (Scopes::SubjectType, "can_be_force_broken_in_peace_treaty", Boolean),
    (Scopes::Location, "can_become_rank", Scope(Scopes::LocationRank)),
    (Scopes::Location.union(Scopes::Country), "can_build_building", Scope(Scopes::BuildingType)),
    (Scopes::Country, "can_build_unit_type", Scope(Scopes::UnitType)),
    (Scopes::Country, "can_build_units_of_category", Scope(Scopes::SubUnitCategory)),
    (
        Scopes::Country,
        "can_create_casus_belli_of_type_on",
        Block(&[("type", Item(Item::CasusBelli)), ("target", Scope(Scopes::Country))]),
    ),
    (Scopes::Country, "can_declare_no_cb_war_on", Scope(Scopes::Country)),
    (Scopes::Country, "can_declare_war_on", Scope(Scopes::Country)),
    (Scopes::Country, "can_do_generic_action", UncheckedTodo),
    (Scopes::Unit, "can_execute_prisoners", UncheckedTodo),
    (Scopes::Country, "can_find_trade_route", UncheckedTodo),
    (Scopes::Country, "can_form", Scope(Scopes::FormableCountry)),
    (Scopes::Unit, "can_hire_prisoners_as_mercenaries", UncheckedTodo),
    (Scopes::InternationalOrganization, "can_initiate_policy_votes", Scope(Scopes::Country)),
    (Scopes::War, "can_join_as_attacker", Scope(Scopes::Country)),
    (Scopes::War, "can_join_as_defender", Scope(Scopes::Country)),
    (Scopes::Country, "can_join_defensive_war_with", Scope(Scopes::Country)),
    (
        Scopes::Country,
        "can_join_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "can_join_offensive_war_with", Scope(Scopes::Country)),
    (
        Scopes::Country,
        "can_lead_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (
        Scopes::Country,
        "can_leave_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "can_make_subject_of", UncheckedTodo),
    (Scopes::Country, "can_pay_price", Scope(Scopes::Price)),
    (Scopes::Country, "can_raise_army_levies", UncheckedTodo),
    (Scopes::Country, "can_raise_levies", UncheckedTodo),
    (Scopes::Country, "can_raise_navy_levies", UncheckedTodo),
    (Scopes::Unit, "can_ransom_prisoners", UncheckedTodo),
    (Scopes::Country, "can_research_advance", UncheckedTodo),
    (Scopes::Country, "can_rival", Scope(Scopes::Country)),
    (Scopes::Country, "can_see_religious_aspect", Scope(Scopes::ReligiousAspect)),
    (Scopes::Country, "can_see_situation", Scope(Scopes::Situation)),
    (Scopes::Unit, "can_sell_prisoners_into_slavery", UncheckedTodo),
    (Scopes::Character, "can_serve_in_cabinet_of", Scope(Scopes::Country)),
    (Scopes::Country, "can_share_maps_with", Scope(Scopes::Country)),
    (Scopes::None, "can_start_tutorial_lesson", UncheckedTodo),
    (Scopes::Unit, "can_upgrade_subunit", UncheckedTodo),
    (Scopes::Unit, "can_upgrade_unit", UncheckedTodo),
    (Scopes::Country, "can_use_agenda_bribe", UncheckedTodo),
    (Scopes::Country, "can_vote_in_parliament", Scope(Scopes::InternationalOrganization)),
    (Scopes::Country, "cancel_exploration_utility", UncheckedTodo),
    (Scopes::Country, "cb_creation_progress_against", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "character_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Character, "character_name", UncheckedTodo),
    (Scopes::Character, "character_nickname", UncheckedTodo),
    (Scopes::Location, "climate", UncheckedTodo),
    (Scopes::Country, "climate_count", UncheckedTodo),
    (Scopes::Country, "climate_percent", UncheckedTodo),
    (Scopes::Country, "colonial_charter_progress", UncheckedTodo),
    (Scopes::Country, "colonial_charter_utility", UncheckedTodo),
    (Scopes::ColonialCharter, "colonial_charter_value", UncheckedTodo),
    (Scopes::Country, "colonial_maintenance", UncheckedTodo),
    (Scopes::Country, "colonial_range", UncheckedTodo),
    (Scopes::CombatSide, "combat_side_strength", UncheckedTodo),
    (Scopes::InternationalOrganization, "combined_special_status_power", UncheckedTodo),
    (Scopes::InternationalOrganization, "combined_unique_special_status_power", UncheckedTodo),
    (Scopes::Value, "compare_value", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "complacency", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "complacency_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "conquer_desire", UncheckedTodo),
    (Scopes::Country, "conquistador_utility", UncheckedTodo),
    (Scopes::Country, "controls", Scope(Scopes::Location)),
    (Scopes::Country, "country_art_quality", UncheckedTodo),
    (
        Scopes::Country,
        "country_can_join_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "country_combined_special_status_power", UncheckedTodo),
    (Scopes::Country, "country_combined_special_status_power_fraction", UncheckedTodo),
    (Scopes::Country, "country_economical_base", UncheckedTodo),
    (Scopes::Country, "country_estate_loan_size", UncheckedTodo),
    (Scopes::None, "country_exists", Scope(Scopes::Country)),
    (Scopes::InternationalOrganization, "country_has_been_member_for_years", UncheckedTodo),
    (Scopes::Country, "country_has_disease", Scope(Scopes::Disease)),
    (Scopes::Country, "country_has_disease_outbreak", Scope(Scopes::DiseaseOutbreak)),
    (Scopes::Country, "country_has_estate", Scope(Scopes::Estate)),
    (Scopes::InternationalOrganization, "country_has_special_status", UncheckedTodo),
    (Scopes::Country, "country_highest_rated_special_status_power", UncheckedTodo),
    (Scopes::Country, "country_interaction_acceptance", UncheckedTodo),
    (Scopes::Country, "country_loan_capacity", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "country_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Country, "country_rank_level", UncheckedTodo),
    (Scopes::Country, "country_rank_level_on_date", UncheckedTodo),
    (Scopes::Country, "country_strength", UncheckedTodo),
    (Scopes::Country, "country_tax_base", UncheckedTodo),
    (Scopes::Country, "country_total_army_levy_size", UncheckedTodo),
    (Scopes::Country, "country_total_navy_levy_size", UncheckedTodo),
    (Scopes::Country, "country_type", UncheckedTodo),
    (Scopes::Language.union(Scopes::Dialect), "court_language_utility", UncheckedTodo),
    (Scopes::Country, "court_maintenance", UncheckedTodo),
    (Scopes::Country, "create_market_utility", UncheckedTodo),
    (Scopes::Culture, "cultural_influence", UncheckedTodo),
    (Scopes::Country, "cultural_maintenance", UncheckedTodo),
    (Scopes::Culture, "cultural_tradition", UncheckedTodo),
    (Scopes::Country, "cultural_unity", UncheckedTodo),
    (Scopes::Culture, "cultural_view", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "culture_group_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "culture_group_percentage_in_country", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "culture_group_population",
        UncheckedTodo,
    ),
    (Scopes::Country, "culture_group_population_in_country", UncheckedTodo),
    (Scopes::Culture, "culture_opinion_impact", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "culture_percentage",
        UncheckedTodo,
    ),
    (Scopes::Area, "culture_percentage_in_area", UncheckedTodo),
    (Scopes::Country, "culture_percentage_in_country", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "culture_population",
        UncheckedTodo,
    ),
    (Scopes::Country, "culture_population_in_country", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "currency_percentage_towards_limit",
        UncheckedTodo,
    ),
    (Scopes::Country, "currency_utility", UncheckedTodo),
    (Scopes::None, "current_age", Item(Item::Age)),
    (Scopes::None, "current_date", UncheckedTodo),
    (Scopes::Country, "current_mission_task", Scope(Scopes::MissionTask)),
    (Scopes::None, "current_month", UncheckedTodo),
    (Scopes::Country, "current_ruler_term_years", UncheckedTodo),
    (Scopes::None, "current_tooltip_depth", UncheckedTodo),
    (Scopes::None, "current_year", UncheckedTodo),
    (Scopes::None, "custom_description", UncheckedTodo),
    (Scopes::None, "custom_tooltip", UncheckedTodo),
    (Scopes::Character, "days_as_rebel", UncheckedTodo),
    (Scopes::Character, "days_of_service_as_admiral", UncheckedTodo),
    (Scopes::Character, "days_of_service_as_general", UncheckedTodo),
    (Scopes::Character, "days_of_service_in_cabinet", UncheckedTodo),
    (Scopes::Disaster, "days_since_disaster_end", UncheckedTodo),
    (Scopes::Disaster, "days_since_disaster_start", UncheckedTodo),
    (Scopes::Situation, "days_since_situation_end", UncheckedTodo),
    (Scopes::Situation, "days_since_situation_start", UncheckedTodo),
    (Scopes::None, "debug_log", UncheckedTodo),
    (Scopes::None, "debug_log_details", UncheckedTodo),
    (Scopes::None, "debug_only", UncheckedTodo),
    (Scopes::Country, "defensive_alliance_strength", UncheckedTodo),
    (Scopes::SubUnit, "definition_is_for_levy", UncheckedTodo),
    (Scopes::Market, "demands_goods", Scope(Scopes::Goods)),
    (Scopes::Market, "demands_goods_by_pops", Scope(Scopes::Goods)),
    (Scopes::Country, "dependency_length_days", UncheckedTodo),
    (Scopes::Country, "destroy_market_utility", UncheckedTodo),
    (Scopes::Location, "development", CompareValue),
    (Scopes::Country.union(Scopes::InternationalOrganization), "devotion", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "devotion_percentage",
        UncheckedTodo,
    ),
    (Scopes::Character, "dip", CompareValue),
    (Scopes::Country, "diplomatic_capacity_of_new_relation", UncheckedTodo),
    (Scopes::Country, "diplomatic_capacity_without_maintenance", UncheckedTodo),
    (Scopes::Country, "diplomatic_maintenance", UncheckedTodo),
    (Scopes::Country, "diplomatic_range", UncheckedTodo),
    (Scopes::Disaster, "disaster_has_ended", UncheckedTodo),
    (Scopes::Disaster, "disaster_is_active", UncheckedTodo),
    (Scopes::Country, "discount_needed_for_law_change", UncheckedTodo),
    (Scopes::Location, "disease_affects_pops_here", Scope(Scopes::Disaster)),
    (Scopes::Country, "disease_country_deaths", UncheckedTodo),
    (Scopes::Location.union(Scopes::SubUnit), "disease_has_outbreak_here", Scope(Scopes::Disease)),
    (Scopes::Location.union(Scopes::SubUnit), "disease_has_stagnated", Scope(Scopes::Disease)),
    (Scopes::None, "disease_is_active", Scope(Scopes::Disease)),
    (Scopes::Country, "disease_outbreak_country_deaths", UncheckedTodo),
    (Scopes::None, "disease_outbreak_is_active", Scope(Scopes::DiseaseOutbreak)),
    (Scopes::Location.union(Scopes::SubUnit), "disease_outbreak_presence", UncheckedTodo),
    (Scopes::DiseaseOutbreak, "disease_outbreak_total_deaths", UncheckedTodo),
    (Scopes::Location.union(Scopes::SubUnit), "disease_presence", UncheckedTodo),
    (Scopes::Location.union(Scopes::SubUnit), "disease_resistance", UncheckedTodo),
    (Scopes::Disease, "disease_total_deaths", UncheckedTodo),
    (Scopes::Location, "distance_to", UncheckedTodo),
    (Scopes::Location, "distance_to_area", UncheckedTodo),
    (Scopes::Location, "distance_to_squared", UncheckedTodo),
    (Scopes::Country, "does_estate_want_other_policy", UncheckedTodo),
    (Scopes::Province, "does_owner_want_to_give_away", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "doom", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "doom_percentage", UncheckedTodo),
    (Scopes::Country.union(Scopes::Dynasty), "dynastic_power", UncheckedTodo),
    (Scopes::None, "dynasty_exists", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "dynasty_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Dynasty, "dynasty_name", UncheckedTodo),
    (Scopes::Character, "education", UncheckedTodo),
    (Scopes::Cabinet, "effective_skill", UncheckedTodo),
    (Scopes::EstateType, "eligible_for_cabinet", UncheckedTodo),
    (Scopes::Location, "employment_percentage", UncheckedTodo),
    (Scopes::BuildingType, "employment_size", UncheckedTodo),
    (Scopes::Country, "employment_system_desire", UncheckedTodo),
    (Scopes::Estate, "estate_gold", UncheckedTodo),
    (Scopes::Country, "estate_loan_interest", UncheckedTodo),
    (Scopes::Country, "estate_max_tax", UncheckedTodo),
    (Scopes::Country, "estate_opinion", UncheckedTodo),
    (Scopes::Country, "estate_satisfaction", UncheckedTodo),
    (Scopes::Estate, "estate_tax", UncheckedTodo),
    (Scopes::Estate, "estate_tax_rate", UncheckedTodo),
    (Scopes::Estate, "estate_taxable_income", UncheckedTodo),
    (Scopes::Country, "estate_type_allowed_in_cabinet", UncheckedTodo),
    (Scopes::Country, "estate_type_allowed_in_command", UncheckedTodo),
    (Scopes::Country, "estate_type_allowed_in_parliament", UncheckedTodo),
    (Scopes::None, "exists", UncheckedTodo),
    (Scopes::Country, "expected_army_size", UncheckedTodo),
    (Scopes::Country, "expected_navy_size", UncheckedTodo),
    (Scopes::Unit, "experience_percentage", UncheckedTodo),
    (Scopes::Area, "exploration_expected_cost", UncheckedTodo),
    (Scopes::Country, "exploration_maintenance", UncheckedTodo),
    (Scopes::Exploration, "exploration_monthly_cost", UncheckedTodo),
    (Scopes::Exploration, "exploration_monthly_progress", UncheckedTodo),
    (Scopes::Area, "exploration_needed_time", UncheckedTodo),
    (Scopes::Exploration, "exploration_progress", UncheckedTodo),
    (Scopes::Exploration, "exploration_time", UncheckedTodo),
    (Scopes::Country, "exploration_utility", UncheckedTodo),
    (Scopes::Country, "favors", UncheckedTodo),
    (Scopes::Country, "favors_needed_to_annul_relations_with", UncheckedTodo),
    (Scopes::Character, "fertility", UncheckedTodo),
    (Scopes::Location, "food_consumption", UncheckedTodo),
    (Scopes::Country, "food_maintenance", UncheckedTodo),
    (Scopes::Unit, "food_percentage", UncheckedTodo),
    (Scopes::Market, "food_price", UncheckedTodo),
    (Scopes::Location, "food_production", UncheckedTodo),
    (Scopes::Goods, "food_value", UncheckedTodo),
    (Scopes::SubjectType, "forbids_sovereign_diplomacy", UncheckedTodo),
    (Scopes::Country, "fort_maintenance", UncheckedTodo),
    (Scopes::Location, "garrison_percentage", UncheckedTodo),
    (Scopes::Location, "garrison_strength", UncheckedTodo),
    (Scopes::Country, "get_antagonism", UncheckedTodo),
    (Scopes::Country, "get_opinion", UncheckedTodo),
    (Scopes::Country, "get_trust", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Pop)
            .union(Scopes::Culture)
            .union(Scopes::Religion)
            .union(Scopes::GraphicalCulture),
        "gfx_culture_applicable",
        UncheckedTodo,
    ),
    (Scopes::SubjectType, "gives_fleet_basing_rights", Boolean),
    (Scopes::Country, "gives_fleet_basing_rights_to", Scope(Scopes::Country)),
    (Scopes::SubjectType, "gives_food_access", UncheckedTodo),
    (Scopes::Country, "gives_food_access_to", Scope(Scopes::Country)),
    (Scopes::Country, "gives_isolation_exemption_to", Scope(Scopes::Country)),
    (Scopes::Country, "gives_military_access_to", Scope(Scopes::Country)),
    (Scopes::Country, "giving_scripted_relation", UncheckedTodo),
    (Scopes::Country, "giving_scripted_relation_of_type", Scope(Scopes::RelationType)),
    (Scopes::None, "global_variable_list_size", UncheckedTodo),
    (Scopes::None, "global_variable_map_size", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "gold", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "gold_percentage", UncheckedTodo),
    (Scopes::Trade, "goods", Scope(Scopes::Goods)),
    (Scopes::Goods, "goods_category", UncheckedTodo),
    (Scopes::Market, "goods_demand_in_market", UncheckedTodo),
    (Scopes::Goods, "goods_method", UncheckedTodo),
    (Scopes::Location, "goods_output", UncheckedTodo),
    (Scopes::Market, "goods_supply_in_market", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "government_power", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "government_power_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "great_power_ranking", UncheckedTodo),
    (Scopes::Country, "great_power_score", UncheckedTodo),
    (Scopes::Country, "had_disaster_for_years", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "harmony", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "harmony_percentage", UncheckedTodo),
    (Scopes::Country, "has_accepted_culture", Scope(Scopes::Culture)),
    (Scopes::Area, "has_accessible_coastline", UncheckedTodo),
    (
        Scopes::InternationalOrganization.union(Scopes::Situation),
        "has_active_resolution",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_advance", UncheckedTodo),
    (Scopes::Country, "has_advance_available", UncheckedTodo),
    (Scopes::Country, "has_advance_for_employment_system", Scope(Scopes::EmploymentSystem)),
    (Scopes::Country, "has_advance_for_succession_law", Scope(Scopes::HeirSelection)),
    (Scopes::Country, "has_antagonism", UncheckedTodo),
    (Scopes::Country, "has_any_active_disaster", UncheckedTodo),
    (Scopes::Location, "has_any_convertable_pops", UncheckedTodo),
    (Scopes::Culture, "has_any_culture_group", UncheckedTodo),
    (Scopes::Location.union(Scopes::SubUnit), "has_any_disease_present", UncheckedTodo),
    (Scopes::Country, "has_any_mission_active", UncheckedTodo),
    (Scopes::Country, "has_any_possible_disaster", UncheckedTodo),
    (Scopes::Character, "has_art_in_progress", UncheckedTodo),
    (Scopes::Area, "has_assigned_explorer", Scope(Scopes::Country)),
    (Scopes::Religion, "has_autocephalous_patriarchates", UncheckedTodo),
    (Scopes::Character, "has_available_marriage_slot", UncheckedTodo),
    (Scopes::Country, "has_avatar", Scope(Scopes::Avatar)),
    (Scopes::FormableCountry, "has_been_formed", UncheckedTodo),
    (
        Scopes::Country,
        "has_been_influenced_by_parliament_agenda",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "has_blocked_treaties", Scope(Scopes::Country)),
    (Scopes::Siege, "has_breach", UncheckedTodo),
    (Scopes::Location, "has_building", Scope(Scopes::BuildingType)),
    (Scopes::Location, "has_building_with_at_least_one_level", UncheckedTodo),
    (Scopes::Location, "has_building_with_graphical_tag", UncheckedTodo),
    (Scopes::Location, "has_building_with_graphical_tag_and_at_least_one_level", UncheckedTodo),
    (Scopes::Character.union(Scopes::Cabinet), "has_cabinet_action", UncheckedTodo),
    (
        Scopes::InternationalOrganization.union(Scopes::Situation),
        "has_cached_or_cast_vote_for",
        UncheckedTodo,
    ),
    (Scopes::Religion, "has_canonization", UncheckedTodo),
    (Scopes::Religion, "has_cardinals", UncheckedTodo),
    (Scopes::War, "has_casus_belli", UncheckedTodo),
    (Scopes::Country, "has_casus_belli_of_type", Scope(Scopes::CasusBelli)),
    (Scopes::Country, "has_casus_belli_of_type_on", UncheckedTodo),
    (Scopes::Country, "has_casus_belli_on", Scope(Scopes::Country)),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_character_modifier",
        UncheckedTodo,
    ),
    (Scopes::Character, "has_child_education", UncheckedTodo),
    (Scopes::Character, "has_child_education_selected", UncheckedTodo),
    (Scopes::Country, "has_claim_on_province", Scope(Scopes::Province)),
    (Scopes::ProvinceDefinition, "has_colonial_charter", Scope(Scopes::Country)),
    (Scopes::Country, "has_colonial_charter_in", Scope(Scopes::ProvinceDefinition)),
    (Scopes::Country, "has_colonial_charters", UncheckedTodo),
    (Scopes::Country, "has_colonial_claim", Scope(Scopes::ProvinceDefinition)),
    (Scopes::Location, "has_combat", UncheckedTodo),
    (Scopes::Unit, "has_commander", UncheckedTodo),
    (Scopes::Country, "has_completed_religious_focus", UncheckedTodo),
    (Scopes::Country, "has_consort", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "has_cooldown", UncheckedTodo),
    (Scopes::Country, "has_core", Scope(Scopes::Location)),
    (Scopes::Country, "has_countries_with_coalition_grade_antagonism", UncheckedTodo),
    (Scopes::Country, "has_countries_with_near_coalition_grade_antagonism", UncheckedTodo),
    (Scopes::Country, "has_countries_with_timed_antagonism", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_country_modifier",
        UncheckedTodo,
    ),
    (Scopes::Culture, "has_culture_group", Scope(Scopes::CultureGroup)),
    (Scopes::Culture, "has_culture_with_tag", UncheckedTodo),
    (Scopes::Mercenary, "has_customer", UncheckedTodo),
    (Scopes::Country, "has_diplomacy_with", UncheckedTodo),
    (Scopes::Country, "has_discovered", Scope(Scopes::Location)),
    (Scopes::Country, "has_discovered_area", Scope(Scopes::Area)),
    (Scopes::None, "has_dlc", UncheckedTodo),
    (Scopes::Religion, "has_doom", UncheckedTodo),
    (Scopes::Character, "has_dynasty", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_dynasty_modifier",
        UncheckedTodo,
    ),
    (Scopes::Location, "has_earthquakes", UncheckedTodo),
    (Scopes::InternationalOrganization, "has_elections", UncheckedTodo),
    (Scopes::Country, "has_embraced_institution", Scope(Scopes::Institution)),
    (Scopes::Country, "has_employment_system", Scope(Scopes::EmploymentSystem)),
    (Scopes::InternationalOrganization, "has_enabled_currency", UncheckedTodo),
    (Scopes::Character, "has_estate", Scope(Scopes::EstateType)),
    (Scopes::Country, "has_estate_privilege", Scope(Scopes::EstatePrivilege)),
    (Scopes::Country.union(Scopes::Character), "has_exploration", UncheckedTodo),
    (Scopes::Character, "has_exploration_construction", UncheckedTodo),
    (Scopes::Location, "has_exports", UncheckedTodo),
    (Scopes::Area, "has_extended_winter", UncheckedTodo),
    (Scopes::None, "has_fired_unique_event", UncheckedTodo),
    (Scopes::Religion, "has_fixed_liturgical_language", UncheckedTodo),
    (Scopes::Location, "has_fort", UncheckedTodo),
    (Scopes::None, "has_game_rule", UncheckedTodo),
    (Scopes::Country, "has_gifted_gold_to", Scope(Scopes::Country)),
    (Scopes::None, "has_global_variable", UncheckedTodo),
    (Scopes::None, "has_global_variable_list", UncheckedTodo),
    (Scopes::None, "has_global_variable_map", UncheckedTodo),
    (Scopes::Culture, "has_graphical_culture", UncheckedTodo),
    (Scopes::Religion, "has_graphical_religion", UncheckedTodo),
    (Scopes::Country, "has_heir", UncheckedTodo),
    (
        Scopes::Country,
        "has_highest_rated_special_status_in_international_organization_of_type",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_historical_rival", Scope(Scopes::Country)),
    (Scopes::Country, "has_historical_rivals", Boolean),
    (Scopes::Religion, "has_holy_sites", UncheckedTodo),
    (Scopes::Religion, "has_honor", UncheckedTodo),
    (Scopes::Location, "has_imports", UncheckedTodo),
    (Scopes::Location, "has_institution", Scope(Scopes::Institution)),
    (Scopes::Country, "has_insulted", Scope(Scopes::Country)),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_international_organization_modifier",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_invited_religious_figure", UncheckedTodo),
    (Scopes::Religion, "has_karma", UncheckedTodo),
    (Scopes::InternationalOrganization, "has_land_ownership_rule", UncheckedTodo),
    (Scopes::Location, "has_latest_road_to", Scope(Scopes::Location)),
    (Scopes::Country, "has_law", UncheckedTodo),
    (Scopes::Exploration, "has_leader", UncheckedTodo),
    (Scopes::Unit, "has_levies", UncheckedTodo),
    (Scopes::Country, "has_limited_diplomacy", UncheckedTodo),
    (Scopes::Loan, "has_loc_key", UncheckedTodo),
    (Scopes::None, "has_local_dlc", UncheckedTodo),
    (Scopes::None, "has_local_variable", UncheckedTodo),
    (Scopes::None, "has_local_variable_list", UncheckedTodo),
    (Scopes::None, "has_local_variable_map", UncheckedTodo),
    (Scopes::InternationalOrganization, "has_location", Scope(Scopes::Location)),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_location_modifier",
        UncheckedTodo,
    ),
    (Scopes::Location, "has_market_construction", UncheckedTodo),
    (Scopes::Country, "has_markets", UncheckedTodo),
    (Scopes::InternationalOrganization, "has_member", Scope(Scopes::Country)),
    (Scopes::Unit, "has_mercenaries", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_mercenary_modifier",
        UncheckedTodo,
    ),
    (Scopes::Market, "has_merchant", Scope(Scopes::Country)),
    (Scopes::Market, "has_merchant_power", UncheckedTodo),
    (Scopes::ProvinceDefinition, "has_migration", UncheckedTodo),
    (Scopes::Country, "has_mission_task", Scope(Scopes::MissionTask)),
    (Scopes::None, "has_multiple_players", UncheckedTodo),
    (Scopes::Country, "has_mutual_scripted_relation", UncheckedTodo),
    (Scopes::Country, "has_mutual_scripted_relation_of_type", Scope(Scopes::RelationType)),
    (Scopes::Market, "has_new_world_goods_in_market", UncheckedTodo),
    (Scopes::None, "has_newsletter_subscription", UncheckedTodo),
    (Scopes::Character, "has_nickname", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "has_ongoing_parliament_debate",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_opinion", UncheckedTodo),
    (Scopes::Country, "has_or_had_tag", UncheckedTodo),
    (Scopes::Goods, "has_origin_in_new_world", UncheckedTodo),
    (Scopes::Goods, "has_origin_in_old_world", UncheckedTodo),
    (Scopes::SubjectType, "has_overlords_ruler", UncheckedTodo),
    (Scopes::Location, "has_owned_buildings", Scope(Scopes::Country)),
    (Scopes::Location, "has_owner", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "has_parliament", UncheckedTodo),
    (Scopes::Country, "has_participated_in_parliament", Scope(Scopes::InternationalOrganization)),
    (Scopes::ProvinceDefinition, "has_passable_land", UncheckedTodo),
    (Scopes::Religion, "has_patriarchs", UncheckedTodo),
    (Scopes::WorkOfArt, "has_periphora", UncheckedTodo),
    (Scopes::Country, "has_policy", UncheckedTodo),
    (Scopes::Country, "has_ports", UncheckedTodo),
    (Scopes::Country, "has_positive_opinion", Scope(Scopes::Country)),
    (Scopes::Province, "has_possible_institution_spawn", UncheckedTodo),
    (Scopes::Country, "has_possible_nomad_targets", Scope(Scopes::ProvinceDefinition)),
    (Scopes::Country, "has_potential_royal_marriage", Scope(Scopes::Country)),
    (Scopes::Country, "has_presence_in", UncheckedTodo),
    (Scopes::Country, "has_primary_or_accepted_culture", Scope(Scopes::Culture)),
    (Scopes::Country, "has_primary_or_accepted_or_tolerated_culture", Scope(Scopes::Culture)),
    (Scopes::Unit, "has_prisoners", Boolean),
    (
        Scopes::Location.union(Scopes::Province).union(Scopes::Area),
        "has_privateers_from",
        Scope(Scopes::Country),
    ),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_province_modifier",
        UncheckedTodo,
    ),
    (Scopes::Religion, "has_purity", UncheckedTodo),
    (Scopes::Country, "has_raised_army_levies", UncheckedTodo),
    (Scopes::Country, "has_raised_levies", UncheckedTodo),
    (Scopes::Country, "has_raised_navy_levies", UncheckedTodo),
    (Scopes::Pop, "has_rebel", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_rebel_modifier",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_reform", Scope(Scopes::GovernmentReform)),
    (Scopes::Country, "has_regent", UncheckedTodo),
    (Scopes::Country, "has_regular_elections", UncheckedTodo),
    (Scopes::Unit, "has_regulars", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_religion_modifier",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_religious_aspect", Scope(Scopes::ReligiousAspect)),
    (Scopes::Religion, "has_religious_factions", UncheckedTodo),
    (Scopes::Religion, "has_religious_focuses", UncheckedTodo),
    (Scopes::Religion, "has_religious_head", UncheckedTodo),
    (Scopes::Religion, "has_religious_influence", UncheckedTodo),
    (Scopes::Religion, "has_religious_schools", UncheckedTodo),
    (Scopes::Location, "has_river", Boolean),
    (Scopes::Location, "has_road_constructions", Boolean),
    (Scopes::Location, "has_road_of_type_to", UncheckedTodo),
    (Scopes::Location, "has_road_to", Scope(Scopes::Location)),
    (Scopes::Location, "has_road_to_capital", Boolean),
    (Scopes::Country, "has_royal_marriage_with", Scope(Scopes::Country)),
    (Scopes::Country, "has_ruler", Boolean),
    (Scopes::Country, "has_scripted_relation", UncheckedTodo),
    (Scopes::Country, "has_scripted_relation_of_type", Scope(Scopes::RelationType)),
    (Scopes::Religion, "has_sects", Boolean),
    (Scopes::Culture, "has_shared_culture_group", Scope(Scopes::Culture)),
    (Scopes::Location, "has_siege", UncheckedTodo),
    (Scopes::Country, "has_societal_value", Scope(Scopes::SocietalValueType)),
    (Scopes::Country, "has_sound_tolls", UncheckedTodo),
    (Scopes::Institution, "has_spawned", UncheckedTodo),
    (
        Scopes::InternationalOrganization,
        "has_special_status_available",
        Scope(Scopes::SpecialStatus),
    ),
    (Scopes::Country, "has_special_status_in_international_organization", UncheckedTodo),
    (Scopes::Rebels, "has_support_from", Scope(Scopes::PopType)),
    (
        Scopes::Religion
            .union(Scopes::Goods)
            .union(Scopes::Building)
            .union(Scopes::Law)
            .union(Scopes::Policy)
            .union(Scopes::BuildingType)
            .union(Scopes::HeirSelection),
        "has_tag",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_target_casus_belli_on_us", Scope(Scopes::Country)),
    (Scopes::Market, "has_temporary_demand", Scope(Scopes::Demand)),
    (Scopes::Market, "has_temporary_demands", Boolean),
    (Scopes::Country, "has_tolerated_culture", Scope(Scopes::Culture)),
    (Scopes::Country, "has_trade_treaty_with", Scope(Scopes::Country)),
    (Scopes::Character, "has_trait", UncheckedTodo),
    (Scopes::Character, "has_trait_category", UncheckedTodo),
    (Scopes::Country, "has_truce_with", Scope(Scopes::Country)),
    (Scopes::Country, "has_trust", UncheckedTodo),
    (Scopes::Character, "has_unit", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "has_unit_modifier",
        UncheckedTodo,
    ),
    (Scopes::Country, "has_unlocked_any_unit_of_category", UncheckedTodo),
    (Scopes::None, "has_variable", UncheckedTodo),
    (Scopes::None, "has_variable_list", UncheckedTodo),
    (Scopes::None, "has_variable_map", UncheckedTodo),
    (Scopes::Location, "has_volcano", UncheckedTodo),
    (Scopes::InternationalOrganization.union(Scopes::Situation), "has_voted", UncheckedTodo),
    (Scopes::InternationalOrganization.union(Scopes::Situation), "has_voted_for", UncheckedTodo),
    (
        Scopes::Country,
        "has_voted_for_issue_in_parliament",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Religion, "has_yanantin", UncheckedTodo),
    (Scopes::Country, "heathen_population_fraction", UncheckedTodo),
    (Scopes::HeirSelection, "heir_candidates_count", UncheckedTodo),
    (Scopes::Character, "heir_position", UncheckedTodo),
    (Scopes::Character, "heir_score", UncheckedTodo),
    (Scopes::Country, "heir_score_country", UncheckedTodo),
    (Scopes::Character, "heir_score_home", UncheckedTodo),
    (Scopes::Location, "hemisphere", UncheckedTodo),
    (Scopes::Country, "heretic_population_fraction", UncheckedTodo),
    (Scopes::None, "hidden_trigger", UncheckedTodo),
    (Scopes::Country, "higher_temporary_taxes_needed", UncheckedTodo),
    (Scopes::Unit, "hire_price", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "honor", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "honor_percentage", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "horde_unity", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "horde_unity_percentage",
        UncheckedTodo,
    ),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::EstatePrivilege)
            .union(Scopes::CabinetAction)
            .union(Scopes::GovernmentReform)
            .union(Scopes::God)
            .union(Scopes::Avatar),
        "implementation_progress_percentage",
        UncheckedTodo,
    ),
    (Scopes::Siege, "in_assault", UncheckedTodo),
    (Scopes::Character, "in_cabinet", UncheckedTodo),
    (Scopes::Country, "in_civil_war", UncheckedTodo),
    (Scopes::Unit, "in_combat", UncheckedTodo),
    (Scopes::Country, "in_marriage_union_with", Scope(Scopes::Country)),
    (Scopes::Unit, "in_retreat", UncheckedTodo),
    (Scopes::Unit, "in_siege", UncheckedTodo),
    (Scopes::Market, "in_trade_range_of", Scope(Scopes::Country)),
    (Scopes::Country, "in_union_with", Scope(Scopes::Country)),
    (Scopes::Country, "in_war_of_casus_belli", Scope(Scopes::CasusBelli)),
    (Scopes::Location, "in_zone_of_control", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "inflation", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "inflation_percentage",
        UncheckedTodo,
    ),
    (Scopes::Location, "integration_level", UncheckedTodo),
    (Scopes::Location, "integration_progress", UncheckedTodo),
    (
        Scopes::None,
        "international_organization_can_add_land",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::InternationalOrganization, "international_organization_can_own_land", UncheckedTodo),
    (
        Scopes::None,
        "international_organization_can_remove_land",
        Scope(Scopes::InternationalOrganization),
    ),
    (
        Scopes::InternationalOrganization,
        "international_organization_has_internal_peace",
        UncheckedTodo,
    ),
    (Scopes::InternationalOrganization, "international_organization_has_law", Scope(Scopes::Law)),
    (Scopes::InternationalOrganization, "international_organization_has_laws", Boolean),
    (Scopes::InternationalOrganization, "international_organization_has_leader", Boolean),
    (
        Scopes::InternationalOrganization,
        "international_organization_has_policy",
        Scope(Scopes::Policy),
    ),
    (Scopes::InternationalOrganization, "international_organization_leader_count", CompareValue),
    (Scopes::InternationalOrganization, "international_organization_leader_reign", CompareValue),
    (
        Scopes::InternationalOrganization,
        "international_organization_leader_reign_in_days",
        CompareValue,
    ),
    (Scopes::InternationalOrganization, "international_organization_lifetime", CompareValue),
    (
        Scopes::InternationalOrganization,
        "international_organization_lifetime_in_days",
        CompareValue,
    ),
    (
        Scopes::InternationalOrganization,
        "international_organization_locations_owned_percentage",
        CompareValue,
    ),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "international_organization_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::InternationalOrganization, "international_organization_num_locations", CompareValue),
    (Scopes::InternationalOrganization, "international_organization_population", CompareValue),
    (Scopes::Location, "intrinsic_disease_resistance", CompareValue),
    (Scopes::InternationalOrganization, "io_within_diplomatic_range", Scope(Scopes::Country)),
    (Scopes::None, "ironman", Boolean),
    (Scopes::War, "is_a_defender", Scope(Scopes::Country)),
    (Scopes::Country, "is_a_threat_for_us", Scope(Scopes::Country)),
    (Scopes::Culture, "is_accepted_in", Scope(Scopes::Country)),
    (Scopes::Country.union(Scopes::InternationalOrganization), "is_active_parliament", Boolean),
    (Scopes::Location, "is_adjacent_to_lake", Boolean),
    (Scopes::Character, "is_admiral", Boolean),
    (Scopes::Character, "is_admiral_of", Scope(Scopes::Country)),
    (Scopes::Character, "is_adolescent", UncheckedTodo),
    (Scopes::Character, "is_adult", UncheckedTodo),
    (Scopes::Country, "is_ai", UncheckedTodo),
    (Scopes::None, "is_alert_shown", UncheckedTodo),
    (Scopes::None, "is_alert_triggered", UncheckedTodo),
    (Scopes::Character, "is_alive", UncheckedTodo),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::BuildingType)
            .union(Scopes::ReligiousAspect)
            .union(Scopes::EstatePrivilege)
            .union(Scopes::CabinetAction)
            .union(Scopes::GovernmentReform)
            .union(Scopes::ProductionMethod)
            .union(Scopes::ParliamentIssue)
            .union(Scopes::ParliamentType)
            .union(Scopes::God)
            .union(Scopes::Avatar)
            .union(Scopes::UnitType)
            .union(Scopes::LevySetup)
            .union(Scopes::ParliamentAgenda)
            .union(Scopes::ArtistType)
            .union(Scopes::Mission)
            .union(Scopes::MissionTask)
            .union(Scopes::RegencyType)
            .union(Scopes::UnitAbility)
            .union(Scopes::RoadType)
            .union(Scopes::HeirSelection)
            .union(Scopes::FormableCountry),
        "is_allowed_for",
        Scope(Scopes::Country),
    ),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::ParliamentIssue)
            .union(Scopes::ParliamentType)
            .union(Scopes::ParliamentAgenda),
        "is_allowed_for_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::CultureGroup, "is_already_merged", Boolean),
    (Scopes::War, "is_an_attacker", Scope(Scopes::Country)),
    (Scopes::Country, "is_annexing", Scope(Scopes::Country)),
    (Scopes::Country, "is_annexing_any_country", Boolean),
    (Scopes::Area, "is_area_coastal_sea", Boolean),
    (Scopes::Area, "is_area_fully_discovered", Scope(Scopes::Country)),
    (Scopes::Area, "is_area_passable", Boolean),
    (Scopes::Area, "is_area_sea", Boolean),
    (Scopes::Unit, "is_army", Boolean),
    (Scopes::WorkOfArt, "is_art_destroyed", UncheckedTodo),
    (Scopes::Character, "is_artist", UncheckedTodo),
    (Scopes::Character, "is_artist_of", Scope(Scopes::Country)),
    (Scopes::Building, "is_at_max_level", UncheckedTodo),
    (Scopes::Country, "is_at_war_with", Scope(Scopes::Country)),
    (Scopes::Country, "is_auto_raise_taxrate_for_all_estates", UncheckedTodo),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::BuildingType)
            .union(Scopes::ReligiousAspect)
            .union(Scopes::EstatePrivilege)
            .union(Scopes::CabinetAction)
            .union(Scopes::GovernmentReform)
            .union(Scopes::ProductionMethod)
            .union(Scopes::ParliamentIssue)
            .union(Scopes::ParliamentType)
            .union(Scopes::God)
            .union(Scopes::Avatar)
            .union(Scopes::UnitType)
            .union(Scopes::LevySetup)
            .union(Scopes::ParliamentAgenda)
            .union(Scopes::ArtistType)
            .union(Scopes::Mission)
            .union(Scopes::MissionTask)
            .union(Scopes::RegencyType)
            .union(Scopes::UnitAbility)
            .union(Scopes::RoadType)
            .union(Scopes::HeirSelection)
            .union(Scopes::FormableCountry),
        "is_available_for",
        Scope(Scopes::Country),
    ),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::ParliamentIssue)
            .union(Scopes::ParliamentType)
            .union(Scopes::ParliamentAgenda),
        "is_available_for_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "is_being_annexed", UncheckedTodo),
    (Scopes::Country, "is_being_annexed_by", Scope(Scopes::Country)),
    (Scopes::Area, "is_being_explored", Scope(Scopes::Country)),
    (Scopes::Combat, "is_bombard_phase", UncheckedTodo),
    (Scopes::Location, "is_border", UncheckedTodo),
    (Scopes::Building, "is_building_owned_by", Scope(Scopes::Country)),
    (Scopes::Location, "is_burgher_positive_deficit", UncheckedTodo),
    (Scopes::None, "is_camera_in_zoom_level", UncheckedTodo),
    (Scopes::Location, "is_capital", UncheckedTodo),
    (Scopes::Unit, "is_carrying_troops", UncheckedTodo),
    (Scopes::Character, "is_child", UncheckedTodo),
    (Scopes::Character, "is_child_of", Scope(Scopes::Character)),
    (Scopes::Location, "is_city", UncheckedTodo),
    (Scopes::War, "is_civil_war_for", Scope(Scopes::Country)),
    (Scopes::Character, "is_close_relative", Scope(Scopes::Character)),
    (Scopes::Location, "is_coastal", UncheckedTodo),
    (Scopes::Country, "is_colonial_overlord", UncheckedTodo),
    (Scopes::Country, "is_colonial_subject", UncheckedTodo),
    (Scopes::Country, "is_colonial_top_overlord", UncheckedTodo),
    (Scopes::Location, "is_connected_to", Scope(Scopes::Location)),
    (Scopes::Character, "is_consort", UncheckedTodo),
    (Scopes::Character, "is_consort_of", Scope(Scopes::Country)),
    (Scopes::Location, "is_core_of", Scope(Scopes::Country)),
    (Scopes::Character, "is_courtier", UncheckedTodo),
    (Scopes::Country, "is_creating_cb_against", Scope(Scopes::Country)),
    (Scopes::Country, "is_creating_cb_of_type", UncheckedTodo),
    (Scopes::Combat, "is_crossing", UncheckedTodo),
    (Scopes::Location, "is_currently_being_integrated", UncheckedTodo),
    (Scopes::CasusBelli, "is_cut_down_in_size_cb", UncheckedTodo),
    (Scopes::Goods, "is_demanded_in_market", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_buildings", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_burgher_trades", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_constructions", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_pops", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_roads", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_trades", Scope(Scopes::Market)),
    (Scopes::Goods, "is_demanded_in_market_by_units", Scope(Scopes::Market)),
    (Scopes::Location.union(Scopes::Country), "is_discovered_by", Scope(Scopes::Country)),
    (Scopes::Country, "is_disloyal_subject", UncheckedTodo),
    (Scopes::Country, "is_dominant_country_of", Scope(Scopes::Culture)),
    (Scopes::Country, "is_during_bankruptcy", UncheckedTodo),
    (Scopes::Character, "is_dynastic_descendant_of", Scope(Scopes::Dynasty)),
    (Scopes::Character, "is_dynasty_head", UncheckedTodo),
    (Scopes::Location, "is_east_of", Scope(Scopes::Location)),
    (
        Scopes::Country,
        "is_elector_in_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Character, "is_eligible_for_royal_marriage", UncheckedTodo),
    (Scopes::Character, "is_eligible_heir", Scope(Scopes::Country)),
    (Scopes::Character, "is_eligible_heir_baseline", Scope(Scopes::Country)),
    (Scopes::Character, "is_eligible_military_leader", Scope(Scopes::Country)),
    (Scopes::Country, "is_embargoed_by", Scope(Scopes::Country)),
    (Scopes::Country, "is_embargoing", Scope(Scopes::Country)),
    (Scopes::Institution, "is_embraced_for", Scope(Scopes::Country)),
    (Scopes::Country, "is_enemy_of", Scope(Scopes::Country)),
    (
        Scopes::Country,
        "is_enemy_of_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Unit, "is_exiled", UncheckedTodo),
    (Scopes::Character, "is_explorer", UncheckedTodo),
    (Scopes::Character, "is_explorer_of", Scope(Scopes::Country)),
    (Scopes::Trade, "is_export", UncheckedTodo),
    (Scopes::Market, "is_export_banned", Scope(Scopes::Goods)),
    (Scopes::Character, "is_female", UncheckedTodo),
    (Scopes::Country, "is_fighting_war_together_with", Scope(Scopes::Country)),
    (Scopes::Goods, "is_food", UncheckedTodo),
    (Scopes::BuildingType, "is_foreign", UncheckedTodo),
    (Scopes::Country, "is_friendly_with", Scope(Scopes::Country)),
    (Scopes::Building, "is_full_capacity", UncheckedTodo),
    (Scopes::Location, "is_full_expanded_rgo", UncheckedTodo),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::EstatePrivilege)
            .union(Scopes::CabinetAction)
            .union(Scopes::GovernmentReform)
            .union(Scopes::God)
            .union(Scopes::Avatar),
        "is_fully_implemented_in",
        Scope(Scopes::Country),
    ),
    (Scopes::None, "is_gamestate_tutorial_active", UncheckedTodo),
    (Scopes::Character, "is_general", UncheckedTodo),
    (Scopes::Character, "is_general_of", Scope(Scopes::Country)),
    (Scopes::Country, "is_great_power", UncheckedTodo),
    (Scopes::Country, "is_hegemon", UncheckedTodo),
    (Scopes::Country, "is_hegemon_type", Scope(Scopes::Hegemony)),
    (Scopes::Character, "is_heir", UncheckedTodo),
    (Scopes::Character, "is_heir_of", Scope(Scopes::Country)),
    (Scopes::Country, "is_historical_rival_of", Scope(Scopes::Country)),
    (Scopes::HolySite, "is_holy_site_for", Scope(Scopes::Religion)),
    (Scopes::Country, "is_hostile_with", Scope(Scopes::Country)),
    (Scopes::Country, "is_human", UncheckedTodo),
    (Scopes::Character, "is_immortal", UncheckedTodo),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::EstatePrivilege)
            .union(Scopes::CabinetAction)
            .union(Scopes::GovernmentReform)
            .union(Scopes::God)
            .union(Scopes::Avatar),
        "is_implementable_in",
        Scope(Scopes::Country),
    ),
    (Scopes::Market, "is_import_banned", Scope(Scopes::Goods)),
    (Scopes::Country, "is_in_any_same_international_organization", Scope(Scopes::Country)),
    (Scopes::None, "is_in_list", UncheckedTodo),
    (Scopes::Country, "is_in_losing_war", UncheckedTodo),
    (Scopes::Country, "is_in_same_international_organization", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "is_in_scripted_geography",
        Scope(Scopes::ScriptedGeography),
    ),
    (Scopes::Goods, "is_in_surplus_in_market", UncheckedTodo),
    (Scopes::War, "is_in_war", UncheckedTodo),
    (Scopes::Character, "is_infant", UncheckedTodo),
    (Scopes::Country, "is_integrating", Scope(Scopes::ProvinceDefinition)),
    (Scopes::InternationalOrganization, "is_international_organization_annullable", UncheckedTodo),
    (Scopes::InternationalOrganization, "is_international_organization_unique", UncheckedTodo),
    (Scopes::None, "is_key_in_global_variable_map", UncheckedTodo),
    (Scopes::None, "is_key_in_local_variable_map", UncheckedTodo),
    (Scopes::None, "is_key_in_variable_map", UncheckedTodo),
    (Scopes::Country, "is_known_by_country", Scope(Scopes::Country)),
    (Scopes::Location, "is_labourer_positive_deficit", UncheckedTodo),
    (Scopes::Building, "is_lacking_goods", UncheckedTodo),
    (Scopes::Location, "is_land", UncheckedTodo),
    (Scopes::RoadType, "is_latest_road_type_for", Scope(Scopes::Country)),
    (
        Scopes::Country,
        "is_leader_of_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Character, "is_leading_largest_army_of", Scope(Scopes::Country)),
    (Scopes::Character, "is_leading_largest_navy_of", Scope(Scopes::Country)),
    (Scopes::SubUnit, "is_levy", UncheckedTodo),
    (Scopes::Pop, "is_linked_to_foreign_building", UncheckedTodo),
    (Scopes::Location, "is_location_holy_site_for", Scope(Scopes::Religion)),
    (Scopes::Trade, "is_locked", UncheckedTodo),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::GovernmentReform)
            .union(Scopes::ParliamentType)
            .union(Scopes::HeirSelection),
        "is_locked_for",
        Scope(Scopes::Country),
    ),
    (
        Scopes::Law.union(Scopes::Policy).union(Scopes::ParliamentType),
        "is_locked_for_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Location, "is_looted", UncheckedTodo),
    (Scopes::Character, "is_loyal", UncheckedTodo),
    (Scopes::GovernmentReform, "is_major_reform", UncheckedTodo),
    (Scopes::None, "is_map_mode_active", UncheckedTodo),
    (Scopes::Location, "is_market_center", UncheckedTodo),
    (Scopes::Character, "is_married", UncheckedTodo),
    (Scopes::Character, "is_matrilineal_descendant_of", Scope(Scopes::Dynasty)),
    (Scopes::Building, "is_max_level", UncheckedTodo),
    (
        Scopes::Country,
        "is_member_of_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "is_member_of_international_organization_of_type", UncheckedTodo),
    (Scopes::SubUnit, "is_mercenary", UncheckedTodo),
    (Scopes::Mercenary, "is_mercenary_hired_by", Scope(Scopes::Country)),
    (Scopes::Character, "is_mercenary_leader", UncheckedTodo),
    (Scopes::Character, "is_mercenary_of", Scope(Scopes::Country)),
    (Scopes::Mercenary, "is_mercenary_owned_by", Scope(Scopes::Country)),
    (Scopes::Culture, "is_merged_culture_group", UncheckedTodo),
    (Scopes::Culture, "is_merged_culture_group_of", Scope(Scopes::CultureGroup)),
    (Scopes::Location, "is_mining_rgo", UncheckedTodo),
    (Scopes::Unit, "is_movement_locked", UncheckedTodo),
    (Scopes::Unit, "is_moving", UncheckedTodo),
    (Scopes::None, "is_multiplayer_session", UncheckedTodo),
    (Scopes::Combat, "is_naval_combat", UncheckedTodo),
    (Scopes::Unit, "is_navy", UncheckedTodo),
    (Scopes::Location.union(Scopes::Country), "is_neighbor_of", Scope(Scopes::Country)),
    (
        Scopes::Location.union(Scopes::Country),
        "is_neighbor_of_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Location, "is_neighbor_of_location", Scope(Scopes::Location)),
    (Scopes::Location, "is_neighbor_of_location_or_across_one_seazone", Scope(Scopes::Location)),
    (
        Scopes::ProvinceDefinition,
        "is_neighbor_of_province_definition",
        Scope(Scopes::ProvinceDefinition),
    ),
    (Scopes::CasusBelli, "is_no_cb", UncheckedTodo),
    (Scopes::War, "is_no_cb_war", UncheckedTodo),
    (Scopes::Building, "is_not_profitable", UncheckedTodo),
    (Scopes::War, "is_on_opposite_sides", UncheckedTodo),
    (Scopes::War, "is_on_same_side", UncheckedTodo),
    (Scopes::Building, "is_opened", UncheckedTodo),
    (Scopes::Country, "is_overlord", UncheckedTodo),
    (Scopes::Location.union(Scopes::Province), "is_overseas_for_owner", UncheckedTodo),
    (Scopes::Location, "is_ownable", UncheckedTodo),
    (Scopes::Location, "is_owned_by_any_international_organization", UncheckedTodo),
    (Scopes::Rebels, "is_owned_by_country", Scope(Scopes::Country)),
    (
        Scopes::Location,
        "is_owned_by_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Location, "is_owned_or_owned_by_subjects_of", Scope(Scopes::Country)),
    (Scopes::Location, "is_owned_or_owned_by_subjects_or_below_of", Scope(Scopes::Country)),
    (Scopes::Character, "is_parent_of", Scope(Scopes::Character)),
    (Scopes::Location, "is_passable", UncheckedTodo),
    (Scopes::Character, "is_patrilineal_descendant_of", Scope(Scopes::Dynasty)),
    (Scopes::Country, "is_player_playstyle", UncheckedTodo),
    (Scopes::Location, "is_port", UncheckedTodo),
    (Scopes::Character, "is_pregnant", UncheckedTodo),
    (Scopes::Culture, "is_primary_in", Scope(Scopes::Country)),
    (Scopes::Culture, "is_primary_or_accepted_in", Scope(Scopes::Country)),
    (Scopes::Goods, "is_produced_by_production_method", Scope(Scopes::ProductionMethod)),
    (Scopes::Location, "is_produced_in_location_market", Scope(Scopes::Goods)),
    (Scopes::Market, "is_produced_in_market", Scope(Scopes::Goods)),
    (Scopes::Building, "is_profitable", UncheckedTodo),
    (Scopes::Market, "is_projected_to_run_out_of_food_stockpile", UncheckedTodo),
    (Scopes::Location, "is_province_capital", UncheckedTodo),
    (Scopes::Country, "is_real_country", UncheckedTodo),
    (Scopes::Country, "is_rebel_country", UncheckedTodo),
    (Scopes::Country, "is_regency_extended", UncheckedTodo),
    (Scopes::Character, "is_regent", UncheckedTodo),
    (Scopes::Character, "is_regent_of", Scope(Scopes::Country)),
    (Scopes::SubUnit, "is_regiment", UncheckedTodo),
    (Scopes::InternationalOrganization, "is_relevant", Scope(Scopes::Country)),
    (Scopes::Religion, "is_religion_enabled", UncheckedTodo),
    (Scopes::Country, "is_religious_aspect_enabled", Scope(Scopes::ReligiousAspect)),
    (Scopes::Character, "is_religious_figure", UncheckedTodo),
    (Scopes::Location, "is_required_for_formable", Scope(Scopes::FormableCountry)),
    (Scopes::Country, "is_revolution_target", UncheckedTodo),
    (Scopes::Country, "is_revolutionary", UncheckedTodo),
    (Scopes::Country, "is_rival_of", Scope(Scopes::Country)),
    (Scopes::Combat, "is_river_crossing", UncheckedTodo),
    (Scopes::Character, "is_ruler", UncheckedTodo),
    (Scopes::Character, "is_ruler_of", Scope(Scopes::Country)),
    (Scopes::Character, "is_saint", UncheckedTodo),
    (Scopes::Character, "is_saint_of", Scope(Scopes::Religion)),
    (Scopes::Character, "is_same_gender", Scope(Scopes::Character)),
    (Scopes::Combat, "is_sea_landing", UncheckedTodo),
    (Scopes::ParliamentIssue, "is_selectable_issue_for", UncheckedTodo),
    (Scopes::None, "is_set", UncheckedTodo),
    (Scopes::SubUnit, "is_ship", UncheckedTodo),
    (Scopes::Character, "is_sibling_of", Scope(Scopes::Character)),
    (Scopes::None, "is_situation_active", Scope(Scopes::Situation)),
    (Scopes::BuildingType, "is_special_building", UncheckedTodo),
    (Scopes::Character, "is_spouse_of", Scope(Scopes::Character)),
    (Scopes::Province, "is_starving", UncheckedTodo),
    (Scopes::Combat, "is_strait_crossing", UncheckedTodo),
    (Scopes::Country, "is_subject", UncheckedTodo),
    (Scopes::Country, "is_subject_of", Scope(Scopes::Country)),
    (Scopes::Country, "is_subject_or_below_of", Scope(Scopes::Country)),
    (Scopes::Country, "is_subject_type", UncheckedTodo),
    (Scopes::SubjectType, "is_subject_type_annullable", UncheckedTodo),
    (Scopes::Building, "is_subsidized", UncheckedTodo),
    (Scopes::Rebels, "is_supported_by_character", Scope(Scopes::Character)),
    (Scopes::Rebels, "is_supported_by_country", Scope(Scopes::Country)),
    (Scopes::None, "is_target_in_global_variable_list", UncheckedTodo),
    (Scopes::None, "is_target_in_local_variable_list", UncheckedTodo),
    (Scopes::None, "is_target_in_variable_list", UncheckedTodo),
    (Scopes::Country, "is_target_of_international_organization_of_type", UncheckedTodo),
    (Scopes::Country, "is_threat_to", Scope(Scopes::Country)),
    (Scopes::Culture, "is_tolerated_in", Scope(Scopes::Country)),
    (Scopes::None, "is_tooltip_with_name_open", UncheckedTodo),
    (Scopes::CasusBelli, "is_trade_cb", UncheckedTodo),
    (Scopes::Market, "is_traded_in_market", Scope(Scopes::Goods)),
    (Scopes::None, "is_tutorial_active", UncheckedTodo),
    (Scopes::None, "is_tutorial_lesson_active", UncheckedTodo),
    (Scopes::None, "is_tutorial_lesson_chain_completed", UncheckedTodo),
    (Scopes::None, "is_tutorial_lesson_completed", UncheckedTodo),
    (Scopes::None, "is_tutorial_lesson_step_completed", UncheckedTodo),
    (Scopes::Location, "is_unified_culture", UncheckedTodo),
    (Scopes::GovernmentReform, "is_unique_reform", UncheckedTodo),
    (Scopes::Unit, "is_unit_locked", UncheckedTodo),
    (Scopes::BuildingType, "is_upgradeable", UncheckedTodo),
    (Scopes::BuildingType, "is_upgraded_level", UncheckedTodo),
    (Scopes::Pop, "is_upper_class", UncheckedTodo),
    (Scopes::Goods, "is_used_by_production_method", Scope(Scopes::ProductionMethod)),
    (Scopes::Country, "is_valid_colonial_charter", Scope(Scopes::ProvinceDefinition)),
    (Scopes::Character, "is_valid_for_exploration", UncheckedTodo),
    (Scopes::None, "is_value_in_global_variable_map", UncheckedTodo),
    (Scopes::None, "is_value_in_local_variable_map", UncheckedTodo),
    (Scopes::None, "is_value_in_variable_map", UncheckedTodo),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::BuildingType)
            .union(Scopes::ReligiousAspect)
            .union(Scopes::EstatePrivilege)
            .union(Scopes::CabinetAction)
            .union(Scopes::GovernmentReform)
            .union(Scopes::ProductionMethod)
            .union(Scopes::ParliamentIssue)
            .union(Scopes::ParliamentType)
            .union(Scopes::God)
            .union(Scopes::Avatar)
            .union(Scopes::UnitType)
            .union(Scopes::LevySetup)
            .union(Scopes::ParliamentAgenda)
            .union(Scopes::ArtistType)
            .union(Scopes::Mission)
            .union(Scopes::MissionTask)
            .union(Scopes::RegencyType)
            .union(Scopes::UnitAbility)
            .union(Scopes::RoadType)
            .union(Scopes::HeirSelection)
            .union(Scopes::FormableCountry),
        "is_visible_for",
        Scope(Scopes::Country),
    ),
    (
        Scopes::Law
            .union(Scopes::Policy)
            .union(Scopes::ParliamentIssue)
            .union(Scopes::ParliamentType)
            .union(Scopes::ParliamentAgenda),
        "is_visible_for_international_organization",
        Scope(Scopes::InternationalOrganization),
    ),
    (Scopes::Country, "is_war_leader_of", Scope(Scopes::War)),
    (Scopes::None, "is_widgetid_open", UncheckedTodo),
    (Scopes::Country, "join_organization_ai_desire", UncheckedTodo),
    (Scopes::War, "join_war_reason", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "karma", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "karma_percentage", UncheckedTodo),
    (Scopes::Country, "knows_about_institution", Scope(Scopes::Institution)),
    (Scopes::Country, "knows_country", Scope(Scopes::Country)),
    (Scopes::Country, "language_percentage_in_country", UncheckedTodo),
    (Scopes::Language.union(Scopes::Dialect), "language_power", UncheckedTodo),
    (
        Scopes::InternationalOrganization,
        "law_enabled_to_international_organization",
        Scope(Scopes::Law),
    ),
    (
        Scopes::InternationalOrganization,
        "law_is_locked_in_international_organization",
        Scope(Scopes::Law),
    ),
    (
        Scopes::InternationalOrganization,
        "law_visible_to_international_organization",
        Scope(Scopes::Law),
    ),
    (Scopes::InternationalOrganization, "leader_change_method", UncheckedTodo),
    (Scopes::InternationalOrganization, "leader_change_trigger_type", UncheckedTodo),
    (Scopes::InternationalOrganization, "leader_special_status_power", UncheckedTodo),
    (Scopes::InternationalOrganization, "leader_special_status_power_fraction", UncheckedTodo),
    (Scopes::InternationalOrganization, "leader_type", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "legitimacy", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "legitimacy_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "liberty_desire", UncheckedTodo),
    (Scopes::None, "list_size", UncheckedTodo),
    (Scopes::Language.union(Scopes::Dialect), "liturgical_language_utility", UncheckedTodo),
    (Scopes::Loan, "loan_amount", UncheckedTodo),
    (Scopes::Loan, "loan_interest", UncheckedTodo),
    (Scopes::Location, "local_control", UncheckedTodo),
    (Scopes::Location, "local_cultural_unity", UncheckedTodo),
    (Scopes::Location, "local_estate_power", UncheckedTodo),
    (Scopes::Location, "local_political_power_fraction", UncheckedTodo),
    (Scopes::Location, "local_relative_estate_power", UncheckedTodo),
    (Scopes::Location, "local_religious_unity", UncheckedTodo),
    (Scopes::None, "local_variable_list_size", UncheckedTodo),
    (Scopes::None, "local_variable_map_size", UncheckedTodo),
    (Scopes::Location, "location_art_quality", UncheckedTodo),
    (Scopes::Location, "location_building_level", UncheckedTodo),
    (
        Scopes::InternationalOrganization,
        "location_can_be_added_to_international_organization",
        Scope(Scopes::Location),
    ),
    (
        Scopes::InternationalOrganization,
        "location_can_be_removed_from_international_organization",
        Scope(Scopes::Location),
    ),
    (
        Scopes::Province
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "location_counter",
        UncheckedTodo,
    ),
    (Scopes::Location, "location_key", UncheckedTodo),
    (Scopes::Location, "location_maritime_merchant_power", UncheckedTodo),
    (Scopes::Location, "location_maritime_presence_power", UncheckedTodo),
    (Scopes::Location, "location_max_population", UncheckedTodo),
    (Scopes::Location, "location_max_winter_level", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "location_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Location, "location_net_building_profit", UncheckedTodo),
    (Scopes::Location, "location_num_holy_sites", UncheckedTodo),
    (Scopes::Location, "location_num_works_of_art", UncheckedTodo),
    (Scopes::Location, "location_peace_cost", UncheckedTodo),
    (Scopes::Location, "location_population_percentage", UncheckedTodo),
    (Scopes::Location, "location_privateer_power", UncheckedTodo),
    (Scopes::Country, "location_progress_for_formable", UncheckedTodo),
    (Scopes::Location, "location_size", UncheckedTodo),
    (Scopes::Location, "location_tax_base", UncheckedTodo),
    (Scopes::Location, "location_unemployed_population_for_building_type", UncheckedTodo),
    (Scopes::Location, "location_within_range", Scope(Scopes::Country)),
    (Scopes::Location, "location_works_of_art_star_rating", UncheckedTodo),
    (Scopes::Country, "long_term_trigger_currency_utility", UncheckedTodo),
    (Scopes::Province, "lowest_prosperity", UncheckedTodo),
    (Scopes::Country, "lowest_war_score", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "manpower", UncheckedTodo),
    (Scopes::Country, "manpower_percentage", UncheckedTodo),
    (Scopes::Location, "market_access", UncheckedTodo),
    (Scopes::Market, "market_food", UncheckedTodo),
    (Scopes::Market, "market_food_deficit", UncheckedTodo),
    (Scopes::Market, "market_food_percentage", UncheckedTodo),
    (Scopes::Market, "market_food_traded", UncheckedTodo),
    (Scopes::Market, "market_max_food", UncheckedTodo),
    (Scopes::Market, "market_monthly_food_balance", UncheckedTodo),
    (Scopes::Market, "market_population", UncheckedTodo),
    (Scopes::Market, "market_possible_goods_trade_surplus", UncheckedTodo),
    (Scopes::Location, "max_control", UncheckedTodo),
    (Scopes::InternationalOrganization, "max_countries_with_special_status", UncheckedTodo),
    (Scopes::Location, "max_garrison_strength", UncheckedTodo),
    (Scopes::Country, "max_manpower", UncheckedTodo),
    (Scopes::HeirSelection, "max_possible_candidates", UncheckedTodo),
    (Scopes::Religion, "max_religious_aspects", UncheckedTodo),
    (Scopes::Location, "max_rgo_workers", CompareValue),
    (Scopes::Country, "max_sailors", CompareValue),
    (Scopes::Religion, "max_sects", UncheckedTodo),
    (Scopes::Mercenary, "mercenary_has_owner", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "mercenary_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Market, "merchant_capacity", UncheckedTodo),
    (Scopes::Market, "merchant_power_in_market", UncheckedTodo),
    (Scopes::Location, "migration_attraction", UncheckedTodo),
    (Scopes::Character, "mil", CompareValue),
    (Scopes::Country, "military_strength", UncheckedTodo),
    (Scopes::Country, "military_tech_level", UncheckedTodo),
    (Scopes::Country, "mission_completed", Scope(Scopes::Mission)),
    (Scopes::Country, "mission_task_bypassed", Scope(Scopes::MissionTask)),
    (Scopes::Country, "mission_task_completed", Scope(Scopes::MissionTask)),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::ReligiousSchool)
            .union(Scopes::InternationalOrganization)
            .union(Scopes::Policy)
            .union(Scopes::ReligiousAspect)
            .union(Scopes::GovernmentReform)
            .union(Scopes::God)
            .union(Scopes::Avatar),
        "modifier_utility",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::ReligiousSchool)
            .union(Scopes::InternationalOrganization)
            .union(Scopes::Policy)
            .union(Scopes::ReligiousAspect)
            .union(Scopes::GovernmentReform)
            .union(Scopes::God)
            .union(Scopes::Avatar),
        "modifier_utility_include_locations",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Country, "monthly_balance", CompareValue),
    (Scopes::Location, "monthly_conversion", CompareValue),
    (Scopes::Mercenary, "monthly_cost", CompareValue),
    (Scopes::Country, "monthly_income_total", CompareValue),
    (Scopes::Country, "monthly_income_trade_and_tax", CompareValue),
    (Scopes::Country, "monthly_manpower", CompareValue),
    (Scopes::Country, "monthly_sailors", CompareValue),
    (Scopes::Country, "monthly_trade_income", CompareValue),
    (Scopes::InternationalOrganization, "months_between_leader_changes", UncheckedTodo),
    (Scopes::Loan, "months_left", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "months_since_last_parliament_called",
        UncheckedTodo,
    ),
    (Scopes::Country, "months_since_peace", UncheckedTodo),
    (Scopes::Country, "months_since_war", UncheckedTodo),
    (Scopes::Unit, "morale_percentage", UncheckedTodo),
    (Scopes::None, "nand", Control),
    (Scopes::Country, "naval_range", UncheckedTodo),
    (Scopes::Country, "navy_maintenance", UncheckedTodo),
    (Scopes::Country, "navy_size", UncheckedTodo),
    (Scopes::Country, "navy_size_percentage", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "navy_tradition", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "navy_tradition_percentage",
        UncheckedTodo,
    ),
    (Scopes::Religion, "need_reforms", UncheckedTodo),
    (Scopes::Country, "needs_opinion_with", UncheckedTodo),
    (Scopes::None, "nor", Control),
    (Scopes::None, "not", Control),
    (Scopes::Country, "num_adult_capable_characters", CompareValue),
    (Scopes::DiseaseOutbreak, "num_affected_locations", CompareValue),
    (Scopes::Location, "num_army_constructions", CompareValue),
    (Scopes::Country, "num_artists", CompareValue),
    (Scopes::Country, "num_avatars", CompareValue),
    (Scopes::Location, "num_buildings", CompareValue),
    (Scopes::Country, "num_cabinet_capable_characters", UncheckedTodo),
    (Scopes::Country, "num_cardinals", CompareValue),
    (Scopes::Country, "num_characters", CompareValue),
    (Scopes::Location, "num_civil_constructions", CompareValue),
    (Scopes::Country, "num_colonial_charters", CompareValue),
    (Scopes::Religion, "num_countries_in_religion", CompareValue),
    (Scopes::InternationalOrganization, "num_countries_with_special_status", UncheckedTodo),
    (Scopes::Country, "num_embraced_institutions", CompareValue),
    (Scopes::Country, "num_explorations", CompareValue),
    (Scopes::Country, "num_explorations_including_in_construction", UncheckedTodo),
    (Scopes::Location, "num_foreign_buildings", CompareValue),
    (Scopes::Country, "num_forts", CompareValue),
    (Scopes::Country, "num_known_institutions", CompareValue),
    (Scopes::Country, "num_loans", CompareValue),
    (Scopes::Country, "num_locations", CompareValue),
    (Scopes::Country, "num_locations_owned_or_owned_by_subjects", CompareValue),
    (Scopes::Country, "num_locations_owned_or_owned_by_subjects_or_below", CompareValue),
    (Scopes::Location, "num_navy_constructions", CompareValue),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "num_of_active_parliament_agendas",
        CompareValue,
    ),
    (Scopes::Country, "num_of_advances_researched", CompareValue),
    (Scopes::Character, "num_of_children", CompareValue),
    (Scopes::Country, "num_of_diplomats", CompareValue),
    (Scopes::InternationalOrganization, "num_of_electors", CompareValue),
    (Scopes::Country, "num_of_locations_owned_by_io", CompareValue),
    (Scopes::Country, "num_of_markets_with_merchants", CompareValue),
    (Scopes::Country, "num_of_non_rural", CompareValue),
    (Scopes::Country, "num_of_non_rural_ports", CompareValue),
    (Scopes::Country, "num_of_ports", CompareValue),
    (Scopes::Rebels, "num_of_rebel_characters", CompareValue),
    (Scopes::Rebels, "num_of_rebel_supporters", CompareValue),
    (Scopes::Country, "num_of_religious_aspects", CompareValue),
    (Scopes::Character, "num_of_spouses", CompareValue),
    (Scopes::Country, "num_of_trades", CompareValue),
    (Scopes::Character, "num_of_traits", CompareValue),
    (Scopes::Character, "num_of_traits_of_category", CompareValue),
    (Scopes::Country, "num_open_reform_slots", CompareValue),
    (Scopes::Location, "num_owned_foreign_buildings_in_location", UncheckedTodo),
    (Scopes::Country.union(Scopes::Estate), "num_possible_privileges", CompareValue),
    (Scopes::Country, "num_possible_rivals", CompareValue),
    (Scopes::Country.union(Scopes::Estate), "num_privileges", CompareValue),
    (Scopes::Area, "num_province_definitions_in_area", CompareValue),
    (Scopes::Country, "num_provinces", CompareValue),
    (Scopes::Country, "num_rebels", CompareValue),
    (Scopes::Country, "num_reforms", CompareValue),
    (Scopes::Country, "num_regiments", CompareValue),
    (Scopes::Country, "num_relations_above_limit", CompareValue),
    (Scopes::Country, "num_rivals", CompareValue),
    (Scopes::Location, "num_roads", CompareValue),
    (Scopes::Country, "num_subjects", CompareValue),
    (Scopes::Unit, "num_subunits", CompareValue),
    (Scopes::Dynasty, "num_union_countries", CompareValue),
    (Scopes::Dynasty, "num_unions", CompareValue),
    (Scopes::Country, "num_works_of_art", CompareValue),
    (Scopes::Country, "offensive_alliance_strength", UncheckedTodo),
    (Scopes::Country, "offer_relation_acceptance", UncheckedTodo),
    (Scopes::SubjectType, "only_allowed_overlord_court_language", UncheckedTodo),
    (Scopes::SubjectType, "only_allowed_overlord_primary_culture", UncheckedTodo),
    (Scopes::SubjectType, "only_allowed_overlord_primary_or_kindred_culture", UncheckedTodo),
    (Scopes::Country, "opinion", UncheckedTodo),
    (Scopes::Country, "opinion_difference_between", UncheckedTodo),
    (Scopes::None, "or", Control),
    (Scopes::InternationalOrganization, "organization_strength_relative_to_country", UncheckedTodo),
    (Scopes::SubjectType, "overlord_can_build_markets", UncheckedTodo),
    (Scopes::SubjectType, "overlord_can_destroy_markets", UncheckedTodo),
    (Scopes::SubjectType, "overlord_can_enforce_peace_on_subject", UncheckedTodo),
    (Scopes::Country, "own_entire_area", Scope(Scopes::Area)),
    (Scopes::Country, "own_entire_province", Scope(Scopes::ProvinceDefinition)),
    (
        Scopes::ProvinceDefinition
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "owned_by_or_its_subjects",
        Scope(Scopes::Country),
    ),
    (Scopes::Country, "owns", Scope(Scopes::Location)),
    (Scopes::Country, "owns_any_foreign_buildings_in", Scope(Scopes::Country)),
    (Scopes::Country, "owns_most_foreign_buildings_in_location", Scope(Scopes::Location)),
    (Scopes::Country, "owns_or_has_subject_in", UncheckedTodo),
    (Scopes::Country, "owns_or_non_sovereign_subject_owns", Scope(Scopes::Location)),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "parliament_issue_chance",
        UncheckedTodo,
    ),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "parliament_issue_support",
        UncheckedTodo,
    ),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "parliament_issue_will_pass",
        UncheckedTodo,
    ),
    (
        Scopes::InternationalOrganization,
        "parliament_type_enabled_in_international_organization",
        Scope(Scopes::ParliamentType),
    ),
    (Scopes::Country, "parliament_type_is_enabled_in", Scope(Scopes::ParliamentType)),
    (Scopes::Country, "parliament_type_is_locked_in", Scope(Scopes::ParliamentType)),
    (
        Scopes::InternationalOrganization,
        "parliament_type_is_locked_in_international_organization",
        Scope(Scopes::ParliamentType),
    ),
    (Scopes::Country, "parliament_type_utility", UncheckedTodo),
    (Scopes::Country, "parliament_type_visible_in", Scope(Scopes::ParliamentType)),
    (
        Scopes::InternationalOrganization,
        "parliament_type_visible_in_international_organization",
        Scope(Scopes::ParliamentType),
    ),
    (Scopes::Country, "payment_contribution", UncheckedTodo),
    (Scopes::Country, "payment_maintenance", UncheckedTodo),
    (Scopes::Country, "peace_treaty_antagonism", UncheckedTodo),
    (Scopes::Country, "peace_treaty_war_score_cost", UncheckedTodo),
    (Scopes::Location, "peasant_enfranchisment", UncheckedTodo),
    (Scopes::Country, "player_proficiency", UncheckedTodo),
    (Scopes::Country, "player_proficiency_greater", UncheckedTodo),
    (Scopes::Country, "player_proficiency_greater_eq", UncheckedTodo),
    (Scopes::Country, "player_proficiency_less", UncheckedTodo),
    (Scopes::Country, "player_proficiency_less_eq", UncheckedTodo),
    (
        Scopes::InternationalOrganization,
        "policy_enabled_to_international_organization",
        Scope(Scopes::Policy),
    ),
    (Scopes::Policy, "policy_has_ai_join_reason", UncheckedTodo),
    (Scopes::Policy, "policy_has_ai_keep_value", UncheckedTodo),
    (Scopes::Policy, "policy_has_ai_propose_value", UncheckedTodo),
    (Scopes::Policy, "policy_has_ai_vote_value", UncheckedTodo),
    (
        Scopes::InternationalOrganization,
        "policy_is_locked_in_international_organization",
        Scope(Scopes::Policy),
    ),
    (Scopes::Policy, "policy_level", UncheckedTodo),
    (
        Scopes::InternationalOrganization,
        "policy_visible_to_international_organization",
        Scope(Scopes::Policy),
    ),
    (Scopes::Pop, "pop_character_chance", UncheckedTodo),
    (Scopes::Pop, "pop_knows_about_goods", Scope(Scopes::Goods)),
    (Scopes::Pop, "pop_literacy", UncheckedTodo),
    (Scopes::Pop, "pop_satisfaction", UncheckedTodo),
    (Scopes::Pop, "pop_size", UncheckedTodo),
    (Scopes::Country, "pop_type_percentage_in_country", UncheckedTodo),
    (Scopes::Country, "pop_type_population_in_country", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "population",
        CompareValue,
    ),
    (Scopes::Area, "population_in_area", UncheckedTodo),
    (Scopes::Location, "population_with_traits", UncheckedTodo), // limit block
    (Scopes::Country, "possible_military_leaders", UncheckedTodo),
    (Scopes::Estate, "power", UncheckedTodo),
    (Scopes::Country, "power_projection", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "prestige", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "prestige_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "prev_antagonism_towards_this", UncheckedTodo),
    (Scopes::Country, "prev_opinion_of_this", UncheckedTodo),
    (Scopes::Country, "prev_trust_of_this", UncheckedTodo),
    (Scopes::Goods, "price_in_market", UncheckedTodo),
    (Scopes::Unit, "prisoner_strength", UncheckedTodo),
    (Scopes::Privateer, "privateer_power", UncheckedTodo),
    (Scopes::Area, "privateer_utility", UncheckedTodo),
    (Scopes::ProductionMethod, "production_method_profit", UncheckedTodo),
    (Scopes::Country, "proper_culture_nobles", UncheckedTodo),
    (Scopes::Location, "prosperity", UncheckedTodo),
    (Scopes::Province, "province_army_levy_size", UncheckedTodo),
    (Scopes::Province, "province_average_control", UncheckedTodo),
    (Scopes::Province, "province_average_development", UncheckedTodo),
    (Scopes::Province, "province_average_integration", UncheckedTodo),
    (Scopes::CabinetAction, "province_cabinet_action", UncheckedTodo),
    (Scopes::Province, "province_cultural_unity", UncheckedTodo),
    (Scopes::Province, "province_food", UncheckedTodo),
    (Scopes::Province, "province_food_percentage", UncheckedTodo),
    (Scopes::Province, "province_max_food", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "province_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Province, "province_monthly_food_production", UncheckedTodo),
    (Scopes::Province, "province_navy_levy_size", UncheckedTodo),
    (Scopes::Province, "province_pop_type_population", UncheckedTodo),
    (Scopes::Province, "province_population", UncheckedTodo),
    (Scopes::Province, "province_possible_institutions", UncheckedTodo),
    (Scopes::Province, "province_prosperity", UncheckedTodo),
    (Scopes::Province, "province_rebel_progress", UncheckedTodo),
    (Scopes::Province, "province_religious_unity", UncheckedTodo),
    (Scopes::Province, "province_satisfaction", UncheckedTodo),
    (Scopes::Province, "province_tax_base", UncheckedTodo),
    (Scopes::Location, "proximity", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "purity", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "purity_percentage", UncheckedTodo),
    (Scopes::None, "random_integer", UncheckedTodo),
    (Scopes::Location, "rank_index", UncheckedTodo),
    (
        Scopes::Market
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "raw_material_amount",
        UncheckedTodo,
    ),
    (Scopes::Goods, "raw_material_occurrence", UncheckedTodo),
    (Scopes::Location, "raw_material_output", UncheckedTodo),
    (Scopes::Country.union(Scopes::Rebels), "rebel_category", UncheckedTodo),
    (Scopes::Rebels, "rebel_estate_type", Scope(Scopes::EstateType)),
    (Scopes::Rebels, "rebel_last_months_progress", UncheckedTodo),
    (Scopes::Rebels, "rebel_locations", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "rebel_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Rebels, "rebel_name_key", UncheckedTodo),
    (Scopes::Rebels, "rebel_progress", UncheckedTodo),
    (Scopes::Rebels, "rebel_size", UncheckedTodo),
    (Scopes::Country, "receives_fleet_basing_rights_from", Scope(Scopes::Country)),
    (Scopes::Country, "receives_food_access_from", Scope(Scopes::Country)),
    (Scopes::Country, "receives_isolation_exemption_from", Scope(Scopes::Country)),
    (Scopes::Country, "receives_military_access_from", Scope(Scopes::Country)),
    (Scopes::Country, "receiving_scripted_relation", UncheckedTodo),
    (Scopes::Country, "receiving_scripted_relation_of_type", Scope(Scopes::RelationType)),
    (Scopes::Religion, "reform_desire", UncheckedTodo),
    (Scopes::Country, "regular_army_size", CompareValue),
    (Scopes::Country, "regular_navy_size", UncheckedTodo),
    (Scopes::Country, "relative_defensive_alliance_strength", UncheckedTodo),
    (Scopes::Country, "relative_military_strength", UncheckedTodo),
    (Scopes::Location, "relative_raw_material_price", UncheckedTodo),
    (Scopes::Country, "relative_strength", UncheckedTodo),
    (Scopes::None, "release_only", UncheckedTodo),
    (Scopes::Country, "relevant_countries", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "religion_group_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "religion_group_percentage_in_country", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "religion_group_population",
        UncheckedTodo,
    ),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "religion_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "religion_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "religion_percentage_in_country", UncheckedTodo),
    (
        Scopes::Location
            .union(Scopes::Province)
            .union(Scopes::ProvinceDefinition)
            .union(Scopes::Area)
            .union(Scopes::Region)
            .union(Scopes::SubContinent)
            .union(Scopes::Continent),
        "religion_population",
        UncheckedTodo,
    ),
    (Scopes::Country, "religion_population_in_country", UncheckedTodo),
    (Scopes::Character, "religious_figure_type", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "religious_influence",
        UncheckedTodo,
    ),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "religious_influence_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "religious_unity", UncheckedTodo),
    (Scopes::Religion, "religious_view", UncheckedTodo),
    (Scopes::Religion, "religious_view_impact", UncheckedTodo),
    (Scopes::Country, "relocate_market_utility", UncheckedTodo),
    (Scopes::Loan, "remaining_debt", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "remaining_parliament_days",
        UncheckedTodo,
    ),
    (
        Scopes::Location.union(Scopes::Country).union(Scopes::Character),
        "remove_static_modifier_utility",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "republican_tradition",
        UncheckedTodo,
    ),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "republican_tradition_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "request_relation_acceptance", UncheckedTodo),
    (Scopes::ProductionMethod, "requires_goods", Scope(Scopes::Goods)),
    (Scopes::Law, "requires_vote", UncheckedTodo),
    (Scopes::Country, "research_progress", UncheckedTodo),
    (
        Scopes::InternationalOrganization.union(Scopes::Situation),
        "resolution_is_active",
        Scope(Scopes::Resolution),
    ),
    (Scopes::Country, "resolution_opinion", UncheckedTodo),
    (Scopes::Country, "reverse_country_interaction_acceptance", UncheckedTodo),
    (Scopes::Culture, "reverse_cultural_view", UncheckedTodo),
    (Scopes::Country, "reverse_offer_relation_acceptance", UncheckedTodo),
    (Scopes::Religion, "reverse_religious_view", UncheckedTodo),
    (Scopes::Religion, "reverse_religious_view_impact", UncheckedTodo),
    (Scopes::Country, "reverse_request_relation_acceptance", UncheckedTodo),
    (Scopes::ReligiousSchool, "reverse_school_opinion", UncheckedTodo),
    (Scopes::Location, "rgo_workers", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "righteousness", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "righteousness_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country.union(Scopes::InternationalOrganization), "rite_power", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "rite_power_percentage",
        UncheckedTodo,
    ),
    (Scopes::Character, "ruled_country_on_or_after", UncheckedTodo),
    (Scopes::Country, "ruler_reign", UncheckedTodo),
    (Scopes::Country, "ruler_reign_in_days", UncheckedTodo),
    (Scopes::Country, "ruler_term_start_date", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "sailors", UncheckedTodo),
    (Scopes::Country, "sailors_percentage", UncheckedTodo),
    (Scopes::Estate, "satisfaction", UncheckedTodo),
    (Scopes::None, "save_temporary_scope_as", UncheckedTodo),
    (Scopes::None, "save_temporary_scope_value_as", UncheckedTodo),
    (Scopes::ReligiousSchool, "school_opinion", UncheckedTodo),
    (Scopes::None, "scope_type", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "self_control", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "self_control_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "short_term_trigger_currency_utility", UncheckedTodo),
    (Scopes::Situation, "situation_has_ended", UncheckedTodo),
    (Scopes::Situation, "situation_is_active", UncheckedTodo),
    (Scopes::Country, "slider_minting_value", UncheckedTodo),
    (Scopes::CabinetAction, "societal_value_cabinet_action", UncheckedTodo),
    (Scopes::Country, "societal_value_progress", UncheckedTodo),
    (Scopes::InternationalOrganization, "special_status_can_be_bestowed", UncheckedTodo),
    (Scopes::InternationalOrganization, "special_status_power", UncheckedTodo),
    (Scopes::InternationalOrganization, "special_status_power_fraction", UncheckedTodo),
    (Scopes::Country, "spy_network", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "stability", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "stability_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "state_religion_clergy", UncheckedTodo),
    (Scopes::Unit, "strength_percentage", UncheckedTodo),
    (Scopes::SubUnit, "sub_unit_type", Scope(Scopes::UnitType)),
    (Scopes::SubjectType, "subject_can_be_annexed", UncheckedTodo),
    (Scopes::SubjectType, "subject_can_be_created_by", UncheckedTodo),
    (Scopes::SubjectType, "subject_level", UncheckedTodo),
    (Scopes::Country, "subject_loyalty", UncheckedTodo),
    (Scopes::SubjectType, "subject_type_annullment_favours_required", UncheckedTodo),
    (Scopes::Country, "subjects_relative_power", UncheckedTodo),
    (Scopes::SubUnit, "subunit_morale", UncheckedTodo),
    (Scopes::SubUnit, "subunit_morale_percentage", UncheckedTodo),
    (Scopes::SubUnit, "subunit_number", UncheckedTodo),
    (Scopes::SubUnit, "subunit_strength", UncheckedTodo),
    (Scopes::SubUnit, "subunit_strength_percentage", UncheckedTodo),
    (Scopes::Country, "supports_rebel", Scope(Scopes::Rebels)),
    (Scopes::None, "switch", UncheckedTodo),
    (Scopes::Country, "tag", UncheckedTodo),
    (Scopes::None, "tag_exists", UncheckedTodo),
    (Scopes::Estate, "target_satisfaction", UncheckedTodo),
    (Scopes::Country, "this_antagonism_towards_prev", UncheckedTodo),
    (Scopes::Country, "this_opinion_of_prev", UncheckedTodo),
    (Scopes::Country, "this_trust_of_prev", UncheckedTodo),
    (Scopes::Country, "threat_level_to", UncheckedTodo),
    (Scopes::None, "time_of_year", UncheckedTodo),
    (Scopes::Religion, "tithe", UncheckedTodo),
    (Scopes::Location, "topography", UncheckedTodo),
    (Scopes::Country, "topography_count", UncheckedTodo),
    (Scopes::Country, "topography_percent", UncheckedTodo),
    (Scopes::Character, "total_abilities", UncheckedTodo),
    (Scopes::Country, "total_accepted_culture_population", UncheckedTodo),
    (Scopes::Location, "total_building_levels", UncheckedTodo),
    (Scopes::Religion, "total_cardinals", UncheckedTodo),
    (Scopes::Country, "total_control_scaled_population", UncheckedTodo),
    (Scopes::Country, "total_debt", UncheckedTodo),
    (Scopes::Country, "total_development", UncheckedTodo),
    (Scopes::Country.union(Scopes::Dynasty), "total_dynastic_power", UncheckedTodo),
    (Scopes::Country, "total_effective_goods_production_buildings", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_enemies", UncheckedTodo),
    (Scopes::Country, "total_foreign_buildings_levels", UncheckedTodo),
    (Scopes::Market, "total_goods_traded", UncheckedTodo),
    (Scopes::Market, "total_goods_value_traded", UncheckedTodo),
    (Scopes::Country, "total_heathen_population", UncheckedTodo),
    (Scopes::Country, "total_heretic_population", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_locations_owned", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_members", UncheckedTodo),
    (Scopes::Country, "total_merchant_capacity", UncheckedTodo),
    (Scopes::Market, "total_merchant_power", UncheckedTodo),
    (Scopes::Country, "total_not_tolerated_culture_population", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_payment_contribution", UncheckedTodo),
    (Scopes::Country, "total_population", UncheckedTodo),
    (Scopes::Country, "total_population_in_international_organization", UncheckedTodo),
    (Scopes::Country, "total_population_in_international_organization_percentage", UncheckedTodo),
    (Scopes::Country, "total_primary_culture_population", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_special_status_power", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_special_status_power_fraction", UncheckedTodo),
    (Scopes::Country, "total_tolerated_culture_population", UncheckedTodo),
    (Scopes::Country, "total_true_faith_population", UncheckedTodo),
    (Scopes::InternationalOrganization, "total_unique_special_status_power", UncheckedTodo),
    (Scopes::Trade, "trade_buy", UncheckedTodo),
    (Scopes::Trade, "trade_capacity_usage_percent", UncheckedTodo),
    (Scopes::Trade, "trade_profit", UncheckedTodo),
    (Scopes::Trade, "trade_sell", UncheckedTodo),
    (Scopes::Trade, "trade_volume", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "tribal_cohesion", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "tribal_cohesion_percentage",
        UncheckedTodo,
    ),
    (Scopes::None, "trigger_else", UncheckedTodo),
    (Scopes::None, "trigger_else_if", UncheckedTodo),
    (Scopes::None, "trigger_if", UncheckedTodo),
    (Scopes::Country, "trust", UncheckedTodo),
    (Scopes::Province, "unfilled_jobs_in_province", UncheckedTodo),
    (Scopes::Province, "unfilled_jobs_in_province_percentage", UncheckedTodo),
    (Scopes::Country, "union_length_days", UncheckedTodo),
    (
        Scopes::None,
        "unique_international_organization_type_exists",
        Scope(Scopes::InternationalOrganizationType),
    ),
    (
        Scopes::Location
            .union(Scopes::Country)
            .union(Scopes::Unit)
            .union(Scopes::Character)
            .union(Scopes::Dynasty)
            .union(Scopes::Religion)
            .union(Scopes::Province)
            .union(Scopes::Rebels)
            .union(Scopes::Mercenary)
            .union(Scopes::InternationalOrganization),
        "unit_modifier_strength",
        Block(&[("modifier", Item(Item::Modifier)), ("value", CompareValue)]),
    ),
    (Scopes::Unit, "unit_strength", UncheckedTodo),
    (Scopes::Country, "upkeep_maintenance", UncheckedTodo),
    (Scopes::Country, "used_cultures_capacity", UncheckedTodo),
    (Scopes::Country, "used_diplomatic_capacity", UncheckedTodo),
    (Scopes::Country, "used_fort_limit", UncheckedTodo),
    (Scopes::Country, "used_fort_limit_percentage", UncheckedTodo),
    (Scopes::Market, "used_merchant_capacity", UncheckedTodo),
    (Scopes::HeirSelection, "uses_elections", UncheckedTodo),
    (Scopes::Country, "uses_government_power", UncheckedTodo),
    (Scopes::Character, "valid_estate_for_heir_selection", Scope(Scopes::HeirSelection)),
    (Scopes::None, "variable_list_size", UncheckedTodo),
    (Scopes::None, "variable_map_size", UncheckedTodo),
    (Scopes::Location, "vegetation", UncheckedTodo),
    (Scopes::Country, "vegetation_count", UncheckedTodo),
    (Scopes::Country, "vegetation_percent", UncheckedTodo),
    (Scopes::Country, "vote_impact_in_resolution", UncheckedTodo),
    (Scopes::InternationalOrganization.union(Scopes::Situation), "vote_is_locked", UncheckedTodo),
    (Scopes::Country, "vote_percentage_impact_in_resolution", UncheckedTodo),
    (Scopes::Resolution, "vote_type", UncheckedTodo),
    (
        Scopes::InternationalOrganization.union(Scopes::Situation),
        "votes_for_resolution",
        UncheckedTodo,
    ),
    (Scopes::Country, "wants_casus_belli_with", Scope(Scopes::Country)),
    (Scopes::Country, "wants_military_access_in", Scope(Scopes::Country)),
    (Scopes::Country, "wants_opinion_with", Scope(Scopes::Country)),
    (Scopes::Country, "wants_to_attack", Scope(Scopes::Country)),
    (Scopes::Country, "wants_to_give_away_any_province", Boolean),
    (Scopes::Country, "wants_to_subjugate", Scope(Scopes::Country)),
    (Scopes::Country, "war_enthusiasm", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "war_exhaustion", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "war_exhaustion_percentage",
        UncheckedTodo,
    ),
    (Scopes::War, "war_goal_type", UncheckedTodo),
    (Scopes::War, "war_length", UncheckedTodo),
    (Scopes::War, "war_length_in_years", UncheckedTodo),
    (Scopes::Country, "war_score_in_war", UncheckedTodo),
    (Scopes::Country, "war_score_in_war_whole_side", UncheckedTodo),
    (Scopes::War, "war_score_of_country", UncheckedTodo),
    (Scopes::War, "war_score_of_country_side", UncheckedTodo),
    (Scopes::Country, "war_score_versus", UncheckedTodo),
    (Scopes::War, "war_stalling_length", UncheckedTodo),
    (Scopes::War, "war_stalling_length_in_years", UncheckedTodo),
    (Scopes::None, "weighted_calc_true_if", UncheckedTodo),
    (Scopes::Location, "winter_level", UncheckedTodo),
    (Scopes::Location, "winter_power", UncheckedTodo),
    (
        Scopes::Location.union(Scopes::Province).union(Scopes::Area),
        "within_colonial_range_of",
        Scope(Scopes::Country),
    ),
    (Scopes::Country, "within_diplomatic_range", Scope(Scopes::Country)),
    (
        Scopes::Location.union(Scopes::Province).union(Scopes::Area),
        "within_naval_range_of",
        Scope(Scopes::Country),
    ),
    (Scopes::None, "world_art_quality", UncheckedTodo),
    (Scopes::None, "world_culture_group_percentage", UncheckedTodo),
    (Scopes::None, "world_culture_group_population", UncheckedTodo),
    (Scopes::None, "world_culture_percentage", UncheckedTodo),
    (Scopes::None, "world_culture_population", UncheckedTodo),
    (Scopes::None, "world_religion_group_percentage", UncheckedTodo),
    (Scopes::None, "world_religion_group_population", UncheckedTodo),
    (Scopes::None, "world_religion_percentage", UncheckedTodo),
    (Scopes::None, "world_religion_population", UncheckedTodo),
    (Scopes::Country.union(Scopes::InternationalOrganization), "yanantin", UncheckedTodo),
    (
        Scopes::Country.union(Scopes::InternationalOrganization),
        "yanantin_percentage",
        UncheckedTodo,
    ),
    (Scopes::Country, "yearly_gold", UncheckedTodo),
    (Scopes::Country, "yearly_manpower", UncheckedTodo),
    (Scopes::Country, "yearly_sailors", UncheckedTodo),
    (Scopes::Character, "yearly_salary", UncheckedTodo),
    (Scopes::Religion, "years_active", UncheckedTodo),
    (Scopes::Character, "years_as_rebel", UncheckedTodo),
    (Scopes::Country, "years_in_international_organization", UncheckedTodo),
    (Scopes::Character, "years_of_service_as_admiral", UncheckedTodo),
    (Scopes::Character, "years_of_service_as_general", UncheckedTodo),
    (Scopes::Character, "years_of_service_in_cabinet", UncheckedTodo),
    (Scopes::Disaster, "years_since_disaster_end", UncheckedTodo),
    (Scopes::Disaster, "years_since_disaster_start", UncheckedTodo),
    (Scopes::Situation, "years_since_situation_end", UncheckedTodo),
    (Scopes::Situation, "years_since_situation_start", UncheckedTodo),
];

#[inline]
pub fn scope_trigger_complex(name: &str) -> Option<(Scopes, ArgumentValue, Scopes)> {
    TRIGGER_COMPLEX_MAP.get(name).copied()
}

static TRIGGER_COMPLEX_MAP: LazyLock<TigerHashMap<&'static str, (Scopes, ArgumentValue, Scopes)>> =
    LazyLock::new(|| {
        let mut hash = TigerHashMap::default();
        for (from, s, trigger, outscopes) in TRIGGER_COMPLEX.iter().copied() {
            hash.insert(s, (from, trigger, outscopes));
        }
        hash
    });

/// LAST UPDATED EU5 VERSION 1.1.9
/// See `triggers.log` from the game data dumps
/// `(inscopes, trigger name, argtype, outscopes)`
/// Currently only works with single argument triggers
const TRIGGER_COMPLEX: &[(Scopes, &str, ArgumentValue, Scopes)] = {
    use crate::item::Item;
    use ArgumentValue::*;
    &[
        (
            Scopes::Country,
            "add_estate_satisfaction_utility",
            Multiple(&[Item(Item::Estate), Scope(Scopes::Value)]),
            Scopes::Value,
        ),
        (
            Scopes::Location.union(Scopes::Country).union(Scopes::Character),
            "add_static_modifier_utility",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::all(),
            "ai_issue_voting_bias",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::Country), Scope(Scopes::Policy)]),
            Scopes::Value,
        ),
        (
            Scopes::ParliamentIssue,
            "ai_parliament_issue_resolution_vote_bias",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (
            Scopes::Policy,
            "ai_policy_reason_to_join",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::InternationalOrganization)]),
            Scopes::Value,
        ),
        (
            Scopes::Policy,
            "ai_policy_resolution_keep_bias",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::InternationalOrganization)]),
            Scopes::Value,
        ),
        (
            Scopes::Policy,
            "ai_policy_resolution_propose_bias",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::InternationalOrganization)]),
            Scopes::Value,
        ),
        (
            Scopes::Policy,
            "ai_policy_resolution_vote_bias",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::InternationalOrganization)]),
            Scopes::Value,
        ),
        (Scopes::Country, "ai_unlock_unit_score", Item(Item::UnitType), Scopes::Value),
        (Scopes::Country, "annexation_cost", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "annexation_progress", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "antagonism", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Area, "area_average_control", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Area, "area_average_integration", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Area, "area_exploration_progress", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Market, "available_merchant_capacity", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::InternationalOrganization,
            "average_special_status_power",
            Item(Item::InternationalOrganizationSpecialStatus),
            Scopes::Value,
        ),
        (Scopes::all(), "bias_value", Item(Item::Bias), Scopes::Value),
        (
            Scopes::Location.union(Scopes::Country).union(Scopes::Province),
            "border_distance_to",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (Scopes::Location, "building_efficiency", Scope(Scopes::BuildingType), Scopes::Value),
        (Scopes::Building, "building_goods_input", Scope(Scopes::Goods), Scopes::Value),
        (Scopes::Location, "building_type_max_level", Scope(Scopes::BuildingType), Scopes::Value),
        (Scopes::Country, "cancel_exploration_utility", Scope(Scopes::Area), Scopes::Value),
        (Scopes::Country, "cb_creation_progress_against", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "character_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (Scopes::Country, "climate_count", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "climate_percent", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Country,
            "colonial_charter_progress",
            Scope(Scopes::ProvinceDefinition),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "colonial_charter_utility",
            Scope(Scopes::ProvinceDefinition),
            Scopes::Value,
        ),
        (Scopes::Country, "conquer_desire", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "conquistador_utility", Scope(Scopes::Area), Scopes::Value),
        (
            Scopes::Country,
            "country_combined_special_status_power",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "country_combined_special_status_power_fraction",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "country_has_been_member_for_years",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "country_highest_rated_special_status_power",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "country_interaction_acceptance",
            Multiple(&[Scope(Scopes::CountryInteraction), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "country_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (Scopes::Country, "country_rank_level_on_date", Scope(Scopes::Date), Scopes::Value),
        (
            Scopes::Language.union(Scopes::Dialect),
            "court_language_utility",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (Scopes::Country, "create_market_utility", Scope(Scopes::Location), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "culture_group_percentage",
            Scope(Scopes::CultureGroup),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "culture_group_percentage_in_country",
            Scope(Scopes::CultureGroup),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "culture_group_population",
            Scope(Scopes::CultureGroup),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "culture_group_population_in_country",
            Scope(Scopes::CultureGroup),
            Scopes::Value,
        ),
        (Scopes::Culture, "culture_opinion_impact", Scope(Scopes::Culture), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "culture_percentage",
            Scope(Scopes::Culture),
            Scopes::Value,
        ),
        (
            Scopes::Area,
            "culture_percentage_in_area",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::Culture)]),
            Scopes::Value,
        ),
        (Scopes::Country, "culture_percentage_in_country", Scope(Scopes::Culture), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "culture_population",
            Scope(Scopes::Culture),
            Scopes::Value,
        ),
        (Scopes::Country, "culture_population_in_country", Scope(Scopes::Culture), Scopes::Value),
        (
            Scopes::Country.union(Scopes::InternationalOrganization),
            "currency_percentage_towards_limit",
            Item(Item::Currency),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "currency_utility",
            Multiple(&[Item(Item::Currency), Scope(Scopes::Value)]),
            Scopes::Value,
        ),
        (Scopes::Country, "dependency_length_days", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "destroy_market_utility", Scope(Scopes::Market), Scopes::Value),
        (Scopes::Country, "disease_country_deaths", Scope(Scopes::Disease), Scopes::Value),
        (
            Scopes::Country,
            "disease_outbreak_country_deaths",
            Scope(Scopes::DiseaseOutbreak),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "disease_outbreak_presence",
            Scope(Scopes::DiseaseOutbreak),
            Scopes::Value,
        ),
        (
            Scopes::Location.union(Scopes::SubUnit),
            "disease_presence",
            Scope(Scopes::Disease),
            Scopes::Value,
        ),
        (
            Scopes::Location.union(Scopes::SubUnit),
            "disease_resistance",
            Scope(Scopes::Disease),
            Scopes::Value,
        ),
        (Scopes::Location, "distance_to", Scope(Scopes::Location), Scopes::Value),
        (Scopes::Location, "distance_to_area", Scope(Scopes::Area), Scopes::Value),
        (Scopes::Country, "distance_to_squared", Scope(Scopes::Location), Scopes::Value),
        (
            Scopes::Country.union(Scopes::Dynasty),
            "dynastic_power",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "dynasty_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "employment_system_desire",
            Scope(Scopes::EmploymentSystem),
            Scopes::Value,
        ),
        (Scopes::Country, "estate_max_tax", Scope(Scopes::EstateType), Scopes::Value),
        (
            Scopes::Country,
            "estate_opinion",
            Multiple(&[Scope(Scopes::EstateType), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (Scopes::Country, "estate_satisfaction", Scope(Scopes::EstateType), Scopes::Value),
        (
            Scopes::Country,
            "exploration_utility",
            Multiple(&[Scope(Scopes::Area), Scope(Scopes::Character), Scope(Scopes::Location)]),
            Scopes::Value,
        ),
        (Scopes::Country, "favors", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Country,
            "favors_needed_to_annul_relations_with",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (Scopes::Country, "get_antagonism", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "get_opinion", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "get_trust", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Market, "goods_demand_in_market", Scope(Scopes::Goods), Scopes::Value),
        (Scopes::Location, "goods_output", Scope(Scopes::Goods), Scopes::Value),
        (Scopes::Market, "goods_supply_in_market", Scope(Scopes::Goods), Scopes::Value),
        (Scopes::Country, "had_disaster_for_years", Scope(Scopes::DisasterType), Scopes::Value),
        (
            Scopes::Law
                .union(Scopes::Policy)
                .union(Scopes::EstatePrivilege)
                .union(Scopes::CabinetAction)
                .union(Scopes::GovernmentReform)
                .union(Scopes::God)
                .union(Scopes::Avatar),
            "implementation_progress_percentage",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "international_organization_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (Scopes::Goods, "is_in_surplus_in_market", Scope(Scopes::Market), Scopes::Value),
        (
            Scopes::Country,
            "join_organization_ai_desire",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (Scopes::Country, "language_percentage_in_country", Scope(Scopes::Language), Scopes::Value),
        (Scopes::Country, "liturgical_language_utility", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Location, "local_relative_estate_power", Scope(Scopes::EstateType), Scopes::Value),
        (Scopes::Location, "location_building_level", Scope(Scopes::BuildingType), Scopes::Value),
        (
            Scopes::Location,
            "location_maritime_merchant_power",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (
            Scopes::Location,
            "location_maritime_presence_power",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "location_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Location,
            "location_peace_cost",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (Scopes::Location, "location_privateer_power", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "location_progress_for_formable", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Location,
            "location_unemployed_population_for_building_type",
            Scope(Scopes::BuildingType),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "mercenary_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::ReligiousSchool)
                .union(Scopes::InternationalOrganization)
                .union(Scopes::Policy)
                .union(Scopes::ReligiousAspect)
                .union(Scopes::GovernmentReform)
                .union(Scopes::God)
                .union(Scopes::Avatar),
            "modifier_utility",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::ReligiousSchool)
                .union(Scopes::InternationalOrganization)
                .union(Scopes::Policy)
                .union(Scopes::ReligiousAspect)
                .union(Scopes::GovernmentReform)
                .union(Scopes::God)
                .union(Scopes::Avatar),
            "modifier_utility_include_locations",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (Scopes::Country, "needs_opinion_with", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Country,
            "num_of_traits_of_category",
            Item(Item::CharacterTraitCategory),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "offer_relation_acceptance",
            Multiple(&[Scope(Scopes::RelationType), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (Scopes::Country, "opinion", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Country,
            "opinion_difference_between",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "organization_strength_relative_to_country",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::Bool)]),
            Scopes::Value,
        ),
        (
            Scopes::Country.union(Scopes::InternationalOrganization),
            "parliament_issue_chance",
            Scope(Scopes::ParliamentIssue),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "parliament_type_utility",
            Multiple(&[Scope(Scopes::ParliamentType), Scope(Scopes::Bool)]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "payment_contribution",
            Multiple(&[
                Scope(Scopes::InternationalOrganization),
                Item(Item::InternationalOrganizationPayment),
            ]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "payment_maintenance",
            Multiple(&[
                Scope(Scopes::InternationalOrganization),
                Item(Item::InternationalOrganizationPayment),
            ]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "peace_treaty_antagonism",
            Multiple(&[
                Scope(Scopes::PeaceTreaty),
                Scope(Scopes::Country),
                Scope(Scopes::all_but_none()),
            ]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "peace_treaty_war_score_cost",
            Multiple(&[
                Scope(Scopes::PeaceTreaty),
                Scope(Scopes::Country),
                Scope(Scopes::all_but_none()),
            ]),
            Scopes::Value,
        ),
        (Scopes::Country, "pop_type_percentage_in_country", Scope(Scopes::PopType), Scopes::Value),
        (Scopes::Country, "pop_type_population_in_country", Scope(Scopes::PopType), Scopes::Value),
        (Scopes::Location, "population_with_traits", Item(Item::ScriptedTrigger), Scopes::Pop),
        (Scopes::Goods, "price_in_market", Scope(Scopes::Market), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "province_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Market
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent),
            "raw_material_amount",
            Scope(Scopes::Goods),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "rebel_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "relative_defensive_alliance_strength",
            Scope(Scopes::Country),
            Scopes::Value,
        ),
        (Scopes::Country, "relative_military_strength", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Location, "relative_raw_material_price", Scope(Scopes::Location), Scopes::Value),
        (Scopes::Country, "relative_strength", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "religion_group_percentage",
            Scope(Scopes::ReligionGroup),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "religion_group_percentage_in_country",
            Scope(Scopes::ReligionGroup),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "religion_group_population",
            Scope(Scopes::ReligionGroup),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "religion_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "religion_percentage",
            Scope(Scopes::Religion),
            Scopes::Value,
        ),
        (Scopes::Country, "religion_percentage_in_country", Scope(Scopes::Religion), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Province)
                .union(Scopes::ProvinceDefinition)
                .union(Scopes::Area)
                .union(Scopes::Region)
                .union(Scopes::SubContinent)
                .union(Scopes::Continent)
                .union(Scopes::ScriptedGeography),
            "religion_population",
            Scope(Scopes::Religion),
            Scopes::Value,
        ),
        (Scopes::Country, "religion_population_in_country", Scope(Scopes::Religion), Scopes::Value),
        (Scopes::Religion, "religious_view_impact", Scope(Scopes::Religion), Scopes::Value),
        (
            Scopes::Country,
            "relocate_market_utility",
            Multiple(&[Scope(Scopes::Location), Scope(Scopes::Location)]),
            Scopes::Value,
        ),
        (
            Scopes::Location.union(Scopes::Country).union(Scopes::Character),
            "remove_static_modifier_utility",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "request_relation_acceptance",
            Multiple(&[Scope(Scopes::RelationType), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "resolution_opinion",
            Multiple(&[
                Scope(Scopes::InternationalOrganization),
                Scope(Scopes::Resolution),
                Scope(Scopes::Country),
            ]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "reverse_country_interaction_acceptance",
            Multiple(&[Scope(Scopes::RelationType), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (Scopes::Religion, "reverse_religious_view", Scope(Scopes::Religion), Scopes::Value),
        (Scopes::Religion, "reverse_religious_view_impact", Scope(Scopes::Religion), Scopes::Value),
        (
            Scopes::Country,
            "reverse_request_relation_acceptance",
            Multiple(&[Scope(Scopes::RelationType), Scope(Scopes::Country)]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "short_term_trigger_currency_utility",
            Multiple(&[UncheckedValue, UncheckedValue, UncheckedValue]), // TODO: this one is strange
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "special_status_power",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::SpecialStatus)]),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "special_status_power_fraction",
            Multiple(&[Scope(Scopes::Country), Scope(Scopes::SpecialStatus)]),
            Scopes::Value,
        ),
        (Scopes::Country, "spy_network", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "threat_level_to", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "topography_count", Scope(Scopes::Topography), Scopes::Value),
        (Scopes::Country, "topography_percent", Scope(Scopes::Topography), Scopes::Value),
        (
            Scopes::Country,
            "total_effective_goods_production_buildings",
            Scope(Scopes::Goods),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "total_payment_contribution",
            Multiple(&[Scope(Scopes::Country), Item(Item::InternationalOrganizationPayment)]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "total_population_in_international_organization",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "total_population_in_international_organization_percentage",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "total_special_status_power",
            Scope(Scopes::SpecialStatus),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "total_special_status_power_fraction",
            Scope(Scopes::SpecialStatus),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization,
            "total_unique_special_status_power",
            Scope(Scopes::SpecialStatus),
            Scopes::Value,
        ),
        (Scopes::Country, "trust", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "union_length_days", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::Location
                .union(Scopes::Country)
                .union(Scopes::Unit)
                .union(Scopes::Character)
                .union(Scopes::Dynasty)
                .union(Scopes::Religion)
                .union(Scopes::Province)
                .union(Scopes::Rebels)
                .union(Scopes::Mercenary)
                .union(Scopes::InternationalOrganization),
            "unit_modifier_strength",
            Item(Item::Modifier),
            Scopes::Value,
        ),
        (Scopes::Country, "vegetation_count", Scope(Scopes::Vegetation), Scopes::Value),
        (Scopes::Country, "vegetation_percent", Scope(Scopes::Vegetation), Scopes::Value),
        (
            Scopes::Country,
            "vote_impact_in_resolution",
            Multiple(&[Scope(Scopes::InternationalOrganization), Scope(Scopes::Resolution)]),
            Scopes::Value,
        ),
        (
            Scopes::Country,
            "vote_percentage_impact_in_resolution",
            Multiple(&[Scope(Scopes::InternationalOrganization), Scope(Scopes::Resolution)]),
            Scopes::Value,
        ),
        (
            Scopes::InternationalOrganization.union(Scopes::Situation),
            "votes_for_resolution",
            Multiple(&[Scope(Scopes::Resolution), Scope(Scopes::all_but_none())]),
            Scopes::Value,
        ),
        (Scopes::Country, "war_enthusiasm", Scope(Scopes::War), Scopes::Value),
        (Scopes::Country, "war_score_in_war", Scope(Scopes::War), Scopes::Value),
        (Scopes::Country, "war_score_in_war_whole_side", Scope(Scopes::War), Scopes::Value),
        (Scopes::War, "war_score_of_country", Scope(Scopes::Country), Scopes::Value),
        (Scopes::War, "war_score_of_country_side", Scope(Scopes::Country), Scopes::Value),
        (Scopes::Country, "war_score_versus", Scope(Scopes::Country), Scopes::Value),
        (
            Scopes::all(),
            "world_culture_group_percentage",
            Scope(Scopes::CultureGroup),
            Scopes::Value,
        ),
        (
            Scopes::all(),
            "world_culture_group_population",
            Scope(Scopes::CultureGroup),
            Scopes::Value,
        ),
        (Scopes::all(), "world_culture_percentage", Scope(Scopes::Culture), Scopes::Value),
        (Scopes::all(), "world_culture_population", Scope(Scopes::Culture), Scopes::Value),
        (
            Scopes::all(),
            "world_religion_group_percentage",
            Scope(Scopes::ReligionGroup),
            Scopes::Value,
        ),
        (
            Scopes::all(),
            "world_religion_group_population",
            Scope(Scopes::ReligionGroup),
            Scopes::Value,
        ),
        (Scopes::all(), "world_religion_percentage", Scope(Scopes::Religion), Scopes::Value),
        (Scopes::all(), "world_religion_population", Scope(Scopes::Religion), Scopes::Value),
        (
            Scopes::Country,
            "years_in_international_organization",
            Scope(Scopes::InternationalOrganization),
            Scopes::Value,
        ),
    ]
};
