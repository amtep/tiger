use std::sync::LazyLock;

use crate::helpers::TigerHashSet;
use crate::lowercase::Lowercase;

/// A hashed version of [`SOUNDS`], for quick case-insensitive lookup.
pub static SOUNDS_SET: LazyLock<TigerHashSet<Lowercase<'static>>> = LazyLock::new(|| {
    let mut set = TigerHashSet::default();
    for sound in SOUNDS.iter().copied() {
        set.insert(Lowercase::new(sound));
    }
    set
});

// LAST UPDATED IMPERATOR VERSION 2.0.4
// Taken from the object browser
pub const SOUNDS: &[&str] = &[
    "event:/MUSIC/BaseGame/MainMenu/mx_maintheme_01",
    "event:/MUSIC/BaseGame/MainMenu/mx_maintheme_02",
    "event:/MUSIC/BaseGame/MainMenu/mx_maintheme_03",
    "event:/MUSIC/BaseGame/Moods/Idle/mx_mood_idle_01",
    "event:/MUSIC/BaseGame/Moods/Idle/mx_mood_idle_02",
    "event:/MUSIC/BaseGame/Moods/Idle/mx_mood_idle_03",
    "event:/MUSIC/BaseGame/Moods/Idle/mx_mood_idle_04",
    "event:/MUSIC/BaseGame/Moods/Idle/mx_mood_idle_05",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_01",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_02",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_03",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_04",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_05",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_06",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_07",
    "event:/MUSIC/BaseGame/Moods/War/mx_mood_war_08",
    "event:/MUSIC/BaseGame/Stingers/mx_endgame_glory",
    "event:/MUSIC/BaseGame/Stingers/mx_endgame_greatest_empire",
    "event:/MUSIC/BaseGame/Stingers/mx_endgame_lose",
    "event:/MUSIC/BaseGame/Stingers/mx_endgame_surviving",
    "event:/MUSIC/DDE/mx_mood_dde_01",
    "event:/MUSIC/DDE/mx_mood_dde_02",
    "event:/MUSIC/DDE/mx_mood_dde_03",
    "event:/MUSIC/MagnaGraecia/mx_mood_mg_01",
    "event:/MUSIC/MagnaGraecia/mx_mood_mg_02",
    "event:/MUSIC/MagnaGraecia/mx_mood_mg_03",
    "event:/MUSIC/PreOrder/mx_mood_po_01",
    "event:/MUSIC/PreOrder/mx_mood_po_02",
    "event:/MUSIC/PunicWars/mx_mood_pw_01",
    "event:/MUSIC/PunicWars/mx_mood_pw_02",
    "event:/MUSIC/PunicWars/mx_mood_pw_03",
    "event:/MUSIC/Patch_2.0/mx_mood_patch_2.0_01",
    "event:/MUSIC/Patch_2.0/mx_mood_patch_2.0_02",
    "event:/MUSIC/Patch_2.0/mx_mood_patch_2.0_03",
    "event:/SFX/Ambience/2DMapEmitters/Nature/sfx_amb_base_wind",
    "event:/SFX/Ambience/3DMapEmitters/Military/sfx_amb_3d_military_fort_hellenic",
    "event:/SFX/Ambience/3DMapEmitters/Military/sfx_amb_3d_military_fort_indian",
    "event:/SFX/Ambience/3DMapEmitters/Military/sfx_amb_3d_military_fort_persian",
    "event:/SFX/Ambience/3DMapEmitters/Military/sfx_amb_3d_military_fort_tribal",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_desert",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_field",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_fire",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_forest",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_mountain",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_ocean",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_river",
    "event:/SFX/Ambience/3DMapEmitters/Nature/sfx_amb_3d_snow",
    "event:/SFX/Ambience/3DMapEmitters/Naval/sfx_amb_3d_naval_port",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_city_african",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_city_hellenic",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_city_indian",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_city_persian",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_city_scythian",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_city_tribal",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_construction_site",
    "event:/SFX/Ambience/3DMapEmitters/Urban/sfx_amb_3d_great_work",
    "event:/SFX/Animations/Map/sfx_anim_map_oceanstorm",
    "event:/SFX/Animations/Map/sfx_anim_map_sandstorm",
    "event:/SFX/Animations/Map/sfx_anim_map_seagull",
    "event:/SFX/Animations/Map/sfx_anim_map_snowstorm",
    "event:/SFX/Animations/Map/sfx_anim_map_volcano",
    "event:/SFX/Animations/Map/sfx_anim_map_volcano_buildup",
    "event:/SFX/Animations/Map/sfx_anim_map_volcano_cooldown",
    "event:/SFX/Animations/Map/sfx_anim_map_volcano_eruption",
    "event:/SFX/Animations/Map/sfx_anim_map_volcano_lava",
    "event:/SFX/Animations/Ships/Generic/sfx_anim_ship_generic_death",
    "event:/SFX/Animations/Ships/Generic/sfx_anim_ship_generic_idle",
    "event:/SFX/Animations/Ships/Generic/sfx_anim_ship_generic_move",
    "event:/SFX/Animations/Ships/Generic/sfx_anim_ship_generic_start",
    "event:/SFX/Animations/Ships/Generic/sfx_anim_ship_vfx_combat_arrows",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_attack",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_block",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_build_border_fort",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_build_colony",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_build_road_loop",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_build_road_post",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_build_road_pre",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_cav_skirmish",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_combat_seq_01",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_combat_seq_02",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_combat_seq_03",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_combat_seq_04",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_combat_seq_05",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_counter",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_dodge",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_drill1",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_drill2",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_generic_idle",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_generic_walk",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_hit",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_raiding",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_raise_levies",
    "event:/SFX/Animations/Units/Generic/sfx_anim_unit_retreat",
    "event:/SFX/Animations/Units/Greek/sfx_anim_unit_greek_phalanx",
    "event:/SFX/Animations/Units/Indian/sfx_anim_unit_indian_padma",
    "event:/SFX/Animations/Units/Roman/sfx_anim_unit_rom_spear_block",
    "event:/SFX/Animations/Units/Roman/sfx_anim_unit_rom_spear_counter",
    "event:/SFX/Animations/Units/Roman/sfx_anim_unit_rom_spear_dodge",
    "event:/SFX/Animations/Units/Roman/sfx_anim_unit_rom_spear_hit",
    "event:/SFX/Events/Generic/sfx_event_generic_civic",
    "event:/SFX/Events/Generic/sfx_event_generic_deified_ruler_death",
    "event:/SFX/Events/Generic/sfx_event_generic_diplomacy",
    "event:/SFX/Events/Generic/sfx_event_generic_happy",
    "event:/SFX/Events/Generic/sfx_event_generic_legion_dishonor",
    "event:/SFX/Events/Generic/sfx_event_generic_legion_honor",
    "event:/SFX/Events/Generic/sfx_event_generic_marching",
    "event:/SFX/Events/Generic/sfx_event_generic_marketplace",
    "event:/SFX/Events/Generic/sfx_event_generic_naval",
    "event:/SFX/Events/Generic/sfx_event_generic_religion",
    "event:/SFX/Events/Generic/sfx_event_generic_sad",
    "event:/SFX/Events/Generic/sfx_event_generic_scholar",
    "event:/SFX/Events/Generic/sfx_event_generic_senate",
    "event:/SFX/Events/Generic/sfx_event_generic_start_up",
    "event:/SFX/Events/Generic/sfx_event_generic_trade",
    "event:/SFX/Events/Generic/sfx_event_generic_uncategorized",
    "event:/SFX/Events/Generic/sfx_event_generic_unrest",
    "event:/SFX/Events/Generic/sfx_event_generic_volcano",
    "event:/SFX/Events/Generic/sfx_event_generic_war",
    "event:/SFX/Events/Generic/sfx_event_generic_wonder",
    "event:/SFX/Gameplay/Combat/sfx_gameplay_combat_screen_land",
    "event:/SFX/Gameplay/Combat/sfx_gameplay_combat_screen_naval",
    "event:/SFX/Gameplay/Combat/sfx_gameplay_combat_screen_sea",
    "event:/SFX/Gameplay/Combat/sfx_gameplay_combat_screen_siege",
    "event:/SFX/Gameplay/Combat/sfx_gameplay_combat_screen_siege_order_assault",
    "event:/SFX/UI/Actions/sfx_ui_action_convert_currency",
    "event:/SFX/UI/Actions/sfx_ui_action_enact_decision",
    "event:/SFX/UI/Actions/sfx_ui_action_endorse_party",
    "event:/SFX/UI/Actions/sfx_ui_action_goverment_interactions",
    "event:/SFX/UI/Actions/sfx_ui_action_governor_policy_changed",
    "event:/SFX/UI/Actions/sfx_ui_action_invoke_devotio",
    "event:/SFX/UI/Actions/sfx_ui_action_invoke_omen",
    "event:/SFX/UI/Actions/sfx_ui_action_pass_law",
    "event:/SFX/UI/Actions/sfx_ui_action_sacrifice",
    "event:/SFX/UI/Actions/sfx_ui_action_select_idea",
    "event:/SFX/UI/Actions/sfx_ui_action_select_idea_civic",
    "event:/SFX/UI/Actions/sfx_ui_action_select_idea_military",
    "event:/SFX/UI/Actions/sfx_ui_action_select_idea_oratory",
    "event:/SFX/UI/Actions/sfx_ui_action_select_idea_religious",
    "event:/SFX/UI/Actions/sfx_ui_action_tax_down",
    "event:/SFX/UI/Actions/sfx_ui_action_tax_up",
    "event:/SFX/UI/Actions/sfx_ui_action_unlock_invention",
    "event:/SFX/UI/Actions/sfx_ui_action_unlock_military_tradition",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_corruption",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_diplomacy",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_economy",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_politics",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_punishment",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_reward",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_scholar",
    "event:/SFX/UI/Character/Generic/sfx_ui_character_war",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_accuse_crime",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_appoint_dictator",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_arrange_marriage",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_banish",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_befriend",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_bribe",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_execute",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_give_free_hands",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_grant_office",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_grant_position",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_hold_games",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_hold_triumph",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_imprison",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_loan_repay",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_loan_request",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_release_prisoner",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_remove_office",
    "event:/SFX/UI/Character/Unique/sfx_ui_character_smear_reputation",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_enforce_peace",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_fabricate_claim",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_form_alliance",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_guarantee",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_intervene_war",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_military_access",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_offer_accept",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_offer_decline",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_peace",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_peace_minor",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_sell_city",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_send_gift",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_sue_for_peace",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_support_rebels",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_threaten_war",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_trade",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_trade_access",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_trade_league",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_trade_minor",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_trade_new_route",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_tribute",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_war_declare_war",
    "event:/SFX/UI/Diplomacy/sfx_ui_diplomacy_war_minor",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_accordion",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_cancel",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_checkbox",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_close",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_decrement",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_drop_down",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_goto",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_increment",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_oversound",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_primary",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_radio",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_secondary",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_sort",
    "event:/SFX/UI/Generic/2.0/sfx_ui_button_switch_out",
    "event:/SFX/UI/Generic/2.0/sfx_ui_snapshot_windows",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_outliner_close",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_outliner_open",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_queued_events_close",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_queued_events_open",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_sidebar",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_tab_close",
    "event:/SFX/UI/Generic/2.0/sfx_ui_window_tab_open",
    "event:/SFX/UI/Generic/sfx_ui_flag_burning",
    "event:/SFX/UI/Generic/sfx_ui_generic_click",
    "event:/SFX/UI/Generic/sfx_ui_generic_close",
    "event:/SFX/UI/Generic/sfx_ui_generic_confirm",
    "event:/SFX/UI/Generic/sfx_ui_generic_decrement",
    "event:/SFX/UI/Generic/sfx_ui_generic_error",
    "event:/SFX/UI/Generic/sfx_ui_generic_errorhoof",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_options",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_pause",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_play",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_saved",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_speed_down",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_speed_up",
    "event:/SFX/UI/Generic/sfx_ui_generic_game_start",
    "event:/SFX/UI/Generic/sfx_ui_generic_increment",
    "event:/SFX/UI/Generic/sfx_ui_generic_map_mode",
    "event:/SFX/UI/Generic/sfx_ui_generic_mouse_over",
    "event:/SFX/UI/Generic/sfx_ui_generic_new_game",
    "event:/SFX/UI/Generic/sfx_ui_generic_select",
    "event:/SFX/UI/Generic/sfx_ui_generic_show_score",
    "event:/SFX/UI/Generic/sfx_ui_generic_sub_tab",
    "event:/SFX/UI/Generic/sfx_ui_generic_tab",
    "event:/SFX/UI/Generic/sfx_ui_nation_release_subject",
    "event:/SFX/UI/GreatWork/sfx_ui_great_work_open_panel",
    "event:/SFX/UI/GreatWork/sfx_ui_great_work_start_construction",
    "event:/SFX/UI/GreatWork/sfx_ui_great_work_swap_material",
    "event:/SFX/UI/Missions/sfx_ui_mission_accept",
    "event:/SFX/UI/Notifications/sfx_ui_achievement_unlocked",
    "event:/SFX/UI/Notifications/sfx_ui_notification_action_needed",
    "event:/SFX/UI/Notifications/sfx_ui_notification_alert",
    "event:/SFX/UI/Notifications/sfx_ui_notification_barbarian_spawned",
    "event:/SFX/UI/Notifications/sfx_ui_notification_message_appears",
    "event:/SFX/UI/Notifications/sfx_ui_notification_money_in",
    "event:/SFX/UI/Notifications/sfx_ui_notification_money_out",
    "event:/SFX/UI/Notifications/sfx_ui_notification_naval_out_range",
    "event:/SFX/UI/Notifications/sfx_ui_notification_pirate_raiding",
    "event:/SFX/UI/Province/sfx_ui_province_build_building",
    "event:/SFX/UI/Province/sfx_ui_province_build_unit",
    "event:/SFX/UI/Province/sfx_ui_province_destroy_building",
    "event:/SFX/UI/Province/sfx_ui_province_hide_info",
    "event:/SFX/UI/Province/sfx_ui_province_improvement",
    "event:/SFX/UI/Province/sfx_ui_province_move_capital",
    "event:/SFX/UI/Province/sfx_ui_province_pop_convert",
    "event:/SFX/UI/Province/sfx_ui_province_pop_move",
    "event:/SFX/UI/Province/sfx_ui_province_pop_promote",
    "event:/SFX/UI/Province/sfx_ui_province_select",
    "event:/SFX/UI/Province/sfx_ui_province_select_window",
    "event:/SFX/UI/Province/sfx_ui_province_show_info",
    "event:/SFX/UI/Religion/sfx_ui_religion_generic_sacrifice",
    "event:/SFX/UI/Religion/sfx_ui_religion_holy_site_create",
    "event:/SFX/UI/Religion/sfx_ui_religion_holy_site_desecrate",
    "event:/SFX/UI/Religion/sfx_ui_religion_ruler_deify",
    "event:/SFX/UI/Religion/sfx_ui_religion_select_deity",
    "event:/SFX/UI/Religion/sfx_ui_religion_treasure_deposit",
    "event:/SFX/UI/Religion/sfx_ui_religion_treasure_view",
    "event:/SFX/UI/Tabs/sfx_ui_tab_characters",
    "event:/SFX/UI/Tabs/sfx_ui_tab_diplomacy",
    "event:/SFX/UI/Tabs/sfx_ui_tab_economy",
    "event:/SFX/UI/Tabs/sfx_ui_tab_government",
    "event:/SFX/UI/Tabs/sfx_ui_tab_military",
    "event:/SFX/UI/Tabs/sfx_ui_tab_missions",
    "event:/SFX/UI/Tabs/sfx_ui_tab_nation_overview",
    "event:/SFX/UI/Tabs/sfx_ui_tab_religion",
    "event:/SFX/UI/Tabs/sfx_ui_tab_technology",
    "event:/SFX/UI/Unit/sfx_ui_unit_assign_to_capital",
    "event:/SFX/UI/Unit/sfx_ui_unit_attach_to",
    "event:/SFX/UI/Unit/sfx_ui_unit_build_road",
    "event:/SFX/UI/Unit/sfx_ui_unit_capture_port",
    "event:/SFX/UI/Unit/sfx_ui_unit_desecrate_holy_site",
    "event:/SFX/UI/Unit/sfx_ui_unit_force_march",
    "event:/SFX/UI/Unit/sfx_ui_unit_merge",
    "event:/SFX/UI/Unit/sfx_ui_unit_new_unit",
    "event:/SFX/UI/Unit/sfx_ui_unit_pirate_raiding",
    "event:/SFX/UI/Unit/sfx_ui_unit_port_assualt",
    "event:/SFX/UI/Unit/sfx_ui_unit_port_raid",
    "event:/SFX/UI/Unit/sfx_ui_unit_raid",
    "event:/SFX/UI/Unit/sfx_ui_unit_remove_pirate",
    "event:/SFX/UI/Unit/sfx_ui_unit_reorganization",
    "event:/SFX/UI/Unit/sfx_ui_unit_select_army",
    "event:/SFX/UI/Unit/sfx_ui_unit_select_navy",
    "event:/SFX/UI/Unit/sfx_ui_unit_set_destination",
    "event:/SFX/UI/Unit/sfx_ui_unit_split",
    "event:/SFX/UI/Unit/sfx_ui_unit_tactic_set",
    "event:/SFX/UI/Unit/sfx_ui_unit_tactic_set_defensive",
    "event:/SFX/UI/Unit/sfx_ui_unit_tactic_set_offensive",
    "event:/SFX_ui_debug_duck",
    "snapshot:/DefaultSnapshot",
    "snapshot:/Gameplay/EndGame",
    "snapshot:/Gameplay/EventPopUp",
    "snapshot:/Gameplay/EventPopUpAmbience",
    "snapshot:/Gameplay/GameStart",
    "snapshot:/States/InGame",
    "snapshot:/States/MainMenu",
    "snapshot:/States/OptionsMenu",
];
