/**
 * @file Paradox Localization
 * @author tiger
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: "pdx_localization",

  extras: ($) => [/[ \t]/],

  conflicts: ($) => [[$.text]],

  rules: {
    source_file: ($) => seq($.top_key, repeat($._lines), optional(choice($.comment, $.definition))),

    top_key: ($) => seq($.identifier, ":", optional($.comment), $._terminator),

    _lines: ($) => seq(choice($.comment, $.definition), $._terminator),

    definition: ($) =>
      seq($.identifier, ":", optional(alias(/\d+/, $.version)), $.text, optional($.comment)),

    identifier: ($) => /[A-Za-z_.0-9]+/,

    text: ($) => seq($.string, repeat(seq(choice($.macro, $.expression, $.format), $.string))),

    macro: ($) => seq("$", /[^#\$\[\]\r\n]*/, "$"),

    expression: ($) => seq("[", /[^#\[\]\r\n]*/, "]"),

    format: ($) =>
      seq("#", alias(/[^#\s]+/, $.format_tag), " ", alias(/[^#\r\n]*/, $.format_content), "#!"),

    string: ($) => /[^#\$\[\]\r\n]*/,

    comment: ($) => seq("#", /[^#\$\[\]\r\n]*/),

    _terminator: ($) => repeat1(choice("\n", "\r\n")),
  },
});
