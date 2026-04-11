use std::sync::LazyLock;

pub mod loca_line;
pub mod util;

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::pedantic)]
    #[allow(clippy::if_then_some_else_none)]
    game_concepts,
    "parse/game_concepts.rs"
);

lalrpop_util::lalrpop_mod!(
    #[allow(clippy::pedantic)]
    #[allow(clippy::if_then_some_else_none)]
    #[allow(clippy::type_complexity)]
    loca_key_text,
    "parse/loca_key_text.rs"
);

pub static GAME_CONCEPTS_PARSER: LazyLock<game_concepts::ConceptsParser> =
    LazyLock::new(game_concepts::ConceptsParser::new);

pub static LOCA_KEY_TEXTS_PARSER: LazyLock<loca_key_text::KeyTextsParser> =
    LazyLock::new(loca_key_text::KeyTextsParser::new);

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
        concepts,
        vec![
            vec![
                "vassals",
                "vassalize",
                "vassalization",
                "vassal_possessive",
                "vassals_possessive",
                "vassalage",
                "vassal"
            ],
            vec!["direct_vassals", "direct_vassal"],
            vec![
                "powerful_vassals",
                "powerful_vassal_powerful",
                "powerful_vassal_possessive",
                "powerful_vassal"
            ],
            vec!["strong_vassals", "strong_vassal"],
            vec![
                "vassal_stances",
                "preferred_heir",
                "unpreferred_heir",
                "preferred_heirs",
                "unpreferred_heirs",
                "vassal_stance"
            ],
            vec!["rulers", "rulerpossessive", "ruler"],
            vec!["title_i", "titles", "titlepossessive", "title"],
            vec!["usurp", "usurpation", "usurped", "usurp_title"],
            vec!["domain_limit"],
            vec!["recently_acquired_holdings", "recently_acquired_holding"],
            vec!["primary_titles", "primary_title"],
            vec!["income"],
            vec!["tax", "taxation", "taxes"],
            vec!["levy", "levies"],
            vec!["obligations", "obligation", "vassal_obligations"]
        ]
    );
}

#[test]
fn test_parse_loca_key_text() {
    let content = r#"l_english:

     ##################################################
     # Misc
     board_games.0000.t:0 "The Greatest Game: [SCOPE.Custom('BG_GameType')|U]"
     board_games.0001.t:0 "$board_games.0000.t$"

     ##################################################
     # Opponent Winning
     board_games.0001.desc.opponent_winning.diplomacy.low:0 ""You know," muses my opponent, "I may not be much of a talker, but I'm alarmingly good at [SCOPE.Custom('BG_GameType')].""
     board_games.0001.desc.opponent_winning.diplomacy.medium:0 ""Ahhhh, if only public speaking was as easy as beating you at [SCOPE.Custom('BG_GameType')]," remarks my opponent."
     board_games.0001.desc.opponent_winning.diplomacy.high:0 ""I genuinely don't know which is easier," laments my opponent, "beating you at [SCOPE.Custom('BG_GameType')] or telling the tale of your embarrassing loss.""
     board_games.0001.desc.opponent_winning.martial.low:0 ""If war was as easy as [SCOPE.Custom('BG_GameType')], I'd rule the world by now," laughs my opponent. "But then, that's just me.""
     board_games.0001.desc.opponent_winning.martial.medium:0 ""Mmmm, you're not much of a strategist, are you?" chortles my opponent."
     board_games.0001.desc.opponent_winning.martial.high:0 ""The battlefield, the board, it's all just a case of that most important thing," warbles my opponent. [bg_opponent.GetSheHe|U] taps [bg_opponent.GetHerHis] forehead with a knuckle for emphasis."
     "#;
    let concepts = loca_key_text::KeyTextsParser::new().parse(content).unwrap();
    assert_eq!(
        concepts
            .iter()
            .map(|(k, v, t)| (k.as_str(), *v, t.as_str()))
            .collect::<Vec<(&str, Option<u16>, &str)>>(),
        vec![
            (
                "board_games.0000.t",
                Some(0),
                "\"The Greatest Game: [SCOPE.Custom('BG_GameType')|U]\""
            ),
            ("board_games.0001.t", Some(0), "\"$board_games.0000.t$\""),
            (
                "board_games.0001.desc.opponent_winning.diplomacy.low",
                Some(0),
                "\"\"You know,\" muses my opponent, \"I may not be much of a talker, but I'm alarmingly good at [SCOPE.Custom('BG_GameType')].\"\""
            ),
            (
                "board_games.0001.desc.opponent_winning.diplomacy.medium",
                Some(0),
                "\"\"Ahhhh, if only public speaking was as easy as beating you at [SCOPE.Custom('BG_GameType')],\" remarks my opponent.\""
            ),
            (
                "board_games.0001.desc.opponent_winning.diplomacy.high",
                Some(0),
                "\"\"I genuinely don't know which is easier,\" laments my opponent, \"beating you at [SCOPE.Custom('BG_GameType')] or telling the tale of your embarrassing loss.\"\""
            ),
            (
                "board_games.0001.desc.opponent_winning.martial.low",
                Some(0),
                "\"\"If war was as easy as [SCOPE.Custom('BG_GameType')], I'd rule the world by now,\" laughs my opponent. \"But then, that's just me.\"\""
            ),
            (
                "board_games.0001.desc.opponent_winning.martial.medium",
                Some(0),
                "\"\"Mmmm, you're not much of a strategist, are you?\" chortles my opponent.\""
            ),
            (
                "board_games.0001.desc.opponent_winning.martial.high",
                Some(0),
                "\"\"The battlefield, the board, it's all just a case of that most important thing,\" warbles my opponent. [bg_opponent.GetSheHe|U] taps [bg_opponent.GetHerHis] forehead with a knuckle for emphasis.\""
            )
        ]
    );
}
