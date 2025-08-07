use std::io::Write;

use ansiterm::{ANSIString, ANSIStrings};
use unicode_width::UnicodeWidthChar;

use crate::fileset::FileKind;
use crate::game::Game;
use crate::report::errors::Errors;
use crate::report::output_style::Styled;
use crate::report::report_struct::pointer_indentation;
use crate::report::{LogReportMetadata, LogReportPointers, PointedMessage, Severity};

/// Source lines printed in the output have leading tab characters replaced by this number of spaces.
const SPACES_PER_TAB: usize = 4;
/// Source lines that have more than this amount of leading whitespace (after tab replacement) have their whitespace truncated.
const MAX_IDLE_SPACE: usize = 16;

/// Log the report.
pub fn log_report<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    report: &LogReportMetadata,
    pointers: &LogReportPointers,
    additional: usize,
) {
    let indentation = pointer_indentation(pointers);
    // Log error lvl and message:
    log_line_title(errors, output, report);

    // Log the pointers:
    let iterator = pointers.iter();
    let mut previous = None;
    for pointer in iterator {
        log_pointer(errors, output, previous, pointer, indentation, report.severity);
        previous = Some(pointer);
    }

    // Log the additional count, if it's more than zero
    if additional > 0 {
        log_count(errors, output, indentation, additional);
    }

    // Log the info line, if one exists.
    if let Some(info) = &report.info {
        log_line_info(errors, output, indentation, info);
    }

    // Log the info line, if one exists.
    if let Some(wiki) = &report.wiki {
        log_line_wiki(errors, output, indentation, wiki);
    }

    // Write a blank line to visually separate reports:
    _ = writeln!(output);
}

fn log_pointer<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    previous: Option<&PointedMessage>,
    pointer: &PointedMessage,
    indentation: usize,
    severity: Severity,
) {
    if previous.is_none() || !previous.unwrap().loc.same_file(pointer.loc) {
        // This pointer is not the same as the previous pointer. Print file location as well:
        log_line_file_location(errors, output, pointer, indentation);
    }
    if pointer.loc.line == 0 {
        // Line being zero means the location is an entire file,
        // not any particular location within the file.
        return;
    }
    if let Some(line) = errors.cache.get_line(pointer.loc) {
        let (line, removed, spaces) = line_spacing(line);
        log_line_from_source(errors, output, pointer, indentation, line, spaces);
        log_line_carets(errors, output, pointer, indentation, line, removed, spaces, severity);
    }
}

/// Log the first line of a report, containing the severity level and the error message.
fn log_line_title<O: Write + Send>(errors: &Errors, output: &mut O, report: &LogReportMetadata) {
    let line: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(Styled::Tag(report.severity, true))
            .paint(format!("{}", report.severity)),
        errors.styles.style(Styled::Tag(report.severity, false)).paint(format!("({})", report.key)),
        errors.styles.style(Styled::Default).paint(": "),
        errors.styles.style(Styled::ErrorMessage).paint(report.msg.clone()),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line));
}

/// Log the optional info line that is part of the overall report.
fn log_line_info<O: Write + Send>(errors: &Errors, output: &mut O, indentation: usize, info: &str) {
    let line_info: &[ANSIString<'static>] = &[
        errors.styles.style(Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Location).paint("="),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::InfoTag).paint("Info:"),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Info).paint(info.to_string()),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line_info));
}

/// Log the optional info line that is part of the overall report.
fn log_line_wiki<O: Write + Send>(errors: &Errors, output: &mut O, indentation: usize, wiki: &str) {
    let line_info: &[ANSIString<'static>] = &[
        errors.styles.style(Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Location).paint("="),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::InfoTag).paint("Wiki:"),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Info).paint(wiki.to_string()),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line_info));
}

/// Log the additional number of this error that were found in other locations
fn log_count<O: Write + Send>(errors: &Errors, output: &mut O, indentation: usize, count: usize) {
    let line_count: &[ANSIString<'static>] = &[
        errors.styles.style(Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(Styled::Location).paint("-->"),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Location).paint(format!("and {count} other locations")),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line_count));
}

/// Log the line containing the location's mod name and filename.
fn log_line_file_location<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    pointer: &PointedMessage,
    indentation: usize,
) {
    let line_filename: &[ANSIString<'static>] = &[
        errors.styles.style(Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(Styled::Location).paint("-->"),
        errors.styles.style(Styled::Default).paint(" "),
        errors
            .styles
            .style(Styled::Location)
            .paint(format!("[{}]", kind_tag(errors, pointer.loc.kind))),
        errors.styles.style(Styled::Default).paint(" "),
        errors
            .styles
            .style(Styled::Location)
            .paint(format!("{}", pointer.loc.pathname().display())),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line_filename));
}

/// Print a line from the source file.
fn log_line_from_source<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    pointer: &PointedMessage,
    indentation: usize,
    line: &str,
    spaces: usize,
) {
    let line_from_source: &[ANSIString<'static>] = &[
        errors.styles.style(Styled::Location).paint(format!("{:indentation$}", pointer.loc.line,)),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Location).paint("|"),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::SourceText).paint(format!("{:spaces$}{line}", "")),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line_from_source));
}

#[allow(clippy::too_many_arguments)]
fn log_line_carets<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    pointer: &PointedMessage,
    indentation: usize,
    line: &str,
    removed: usize,
    spaces: usize,
    severity: Severity,
) {
    if pointer.length == 0 {
        return;
    }

    let mut spacing = String::new();
    for c in line.chars().take((pointer.loc.column as usize).saturating_sub(removed + 1)) {
        // There might still be tabs in the non-leading space
        if c == '\t' {
            spacing.push('\t');
        } else {
            for _ in 0..c.width().unwrap_or(0) {
                spacing.push(' ');
            }
        }
    }

    // A line containing the carets that point upwards at the source line.
    let line_carets: &[ANSIString] = &[
        errors.styles.style(Styled::Default).paint(format!("{:indentation$}", "")),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Location).paint("|"),
        errors.styles.style(Styled::Default).paint(format!(
            "{:width$}{spacing}",
            "",
            width = spaces + 1
        )),
        errors.styles.style(Styled::Tag(severity, true)).paint(format!(
            "{:^^width$}",
            "",
            width = pointer.length
        )),
        errors.styles.style(Styled::Default).paint(" "),
        errors
            .styles
            .style(Styled::Tag(severity, true))
            .paint(pointer.msg.as_deref().map_or("", |_| "<-- ")),
        errors
            .styles
            .style(Styled::Tag(severity, true))
            .paint(pointer.msg.as_deref().unwrap_or("")),
    ];
    _ = writeln!(output, "{}", ANSIStrings(line_carets));
}

pub(crate) fn kind_tag<'a>(errors: &'a Errors<'a>, kind: FileKind) -> &'a str {
    match kind {
        FileKind::Internal => "Internal",
        FileKind::Clausewitz => "Clausewitz",
        FileKind::Jomini => "Jomini",
        FileKind::Vanilla => match Game::game() {
            #[cfg(feature = "ck3")]
            Game::Ck3 => "CK3",
            #[cfg(feature = "vic3")]
            Game::Vic3 => "Vic3",
            #[cfg(feature = "imperator")]
            Game::Imperator => "Imperator",
            #[cfg(feature = "hoi4")]
            Game::Hoi4 => "Hoi4",
        },
        FileKind::Dlc(idx) => &errors.loaded_dlcs_labels[idx as usize],
        FileKind::LoadedMod(idx) => &errors.loaded_mods_labels[idx as usize],
        FileKind::Mod => "MOD",
    }
}

/// Removes the leading spaces and tabs from `line` and returns it,
/// together with how many character positions were removed and how many spaces should be substituted.
fn line_spacing(line: &str) -> (&str, usize, usize) {
    let mut remove = 0;
    let mut spaces = 0;
    for c in line.chars() {
        if c == ' ' {
            spaces += 1;
        } else if c == '\t' {
            spaces += SPACES_PER_TAB;
        } else {
            break;
        }
        remove += 1;
    }
    spaces = spaces.min(MAX_IDLE_SPACE);
    (&line[remove..], remove, spaces)
}
