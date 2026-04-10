use std::sync::LazyLock;

pub mod loca_line;
pub mod util;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::pedantic)]
    #[allow(clippy::if_then_some_else_none)]
    game_concepts,
    "parse/game_concepts.rs"
);

pub static GAME_CONCEPTS_PARSER: LazyLock<game_concepts::ConceptsParser> =
    LazyLock::new(game_concepts::ConceptsParser::new);

#[test]
fn test_parse_game_concepts() {
    let content = r#"vassal = {
	alias = { vassals vassalize vassalization vassal_possessive vassals_possessive vassalage }
	parent = ruler
	texture = "gfx/interface/icons/icon_vassal.dds"
        }

        direct_vassal = {
	texture = "gfx/interface/icons/icon_vassal.dds"
	alias = { direct_vassals }
	parent = vassal
        }

        powerful_vassal = {
	texture = "gfx/interface/icons/portraits/powerful_vassal.dds"
	framesize = { 40 40 }

	frame = 1
	alias = { powerful_vassals powerful_vassal_powerful powerful_vassal_possessive }
	parent = vassal
        }

        strong_vassal = {
	texture = "gfx/interface/icons/icon_vassal.dds"
	alias = { strong_vassals }
	parent = vassal
        }

        vassal_stance = {
	parent = vassal
	alias = { vassal_stances preferred_heir unpreferred_heir preferred_heirs unpreferred_heirs }
	texture = "gfx/interface/icons/icon_vassal.dds"
        }

        ruler = {
	alias = { rulers rulerpossessive }
        }

        title = {
	texture = "gfx/interface/icons/message_feed/titles.dds"
	alias = { title_i titles titlepossessive }
        }

        usurp_title = {
	texture = "gfx/interface/icons/message_feed/titles.dds"
	alias = { usurp usurpation usurped }
        }

        domain_limit = {
	texture = "gfx/interface/icons/icon_domain.dds"
        }

        recently_acquired_holding = {
	alias = { recently_acquired_holdings }
	texture = "gfx/interface/icons/map_icons/onmap_holding_icon.dds"
        }

        primary_title = {
	texture = "gfx/interface/icons/message_feed/titles.dds"
	parent = title
	alias = { primary_titles }
        }

        income = {
	texture = "gfx/interface/icons/icon_gold.dds"
        }
        taxes = {
	alias = { tax taxation }
	texture = "gfx/interface/icons/council_task_types/task_collect_taxes.dds"
        }
        levies = {
	parent = soldiers
	alias = { levy }
	texture = "gfx/interface/icons/icon_levies.dds"
        }

        vassal_obligations = {
	texture = "gfx/interface/icons/icon_contract_modification.dds"
	framesize = { 60 60 }
	frame = 1
	alias = { obligations obligation }
        }

    "#;
    let concepts = game_concepts::ConceptsParser::new().parse(content).unwrap();
    assert_eq!(
        concepts.iter().map(String::as_str).collect::<Vec<&str>>(),
        vec![
            "vassals",
            "vassalize",
            "vassalization",
            "vassal_possessive",
            "vassals_possessive",
            "vassalage",
            "vassal",
            "direct_vassals",
            "direct_vassal",
            "powerful_vassals",
            "powerful_vassal_powerful",
            "powerful_vassal_possessive",
            "powerful_vassal",
            "strong_vassals",
            "strong_vassal",
            "vassal_stances",
            "preferred_heir",
            "unpreferred_heir",
            "preferred_heirs",
            "unpreferred_heirs",
            "vassal_stance",
            "rulers",
            "rulerpossessive",
            "ruler",
            "title_i",
            "titles",
            "titlepossessive",
            "title",
            "usurp",
            "usurpation",
            "usurped",
            "usurp_title",
            "domain_limit",
            "recently_acquired_holdings",
            "recently_acquired_holding",
            "primary_titles",
            "primary_title",
            "income",
            "tax",
            "taxation",
            "taxes",
            "levy",
            "levies",
            "obligations",
            "obligation",
            "vassal_obligations"
        ]
    );
}
