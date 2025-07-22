use ansiterm::{ANSIString, ANSIStrings};
use unicode_width::UnicodeWidthChar;

use crate::fileset::FileKind;
use crate::game::Game;
use crate::report::errors::Errors;
use crate::report::output_style::Styled;
use crate::report::{LogReport, PointedMessage, Severity};

/// Source lines printed in the output have leading tab characters replaced by this number of spaces.
const SPACES_PER_TAB: usize = 4;
/// Source lines that have more than this amount of leading whitespace (after tab replacement) have their whitespace truncated.
const MAX_IDLE_SPACE: usize = 16;

/// Log the report.
pub fn log_report(errors: &mut Errors, report: &LogReport) {
    let indentation = report.indentation();

    // Log error lvl and message:
    log_line_title(errors, report);

    // Log the pointers:
    let iterator = report.pointers.iter();
    let mut previous = None;
    for pointer in iterator {
        log_pointer(errors, previous, pointer, indentation, report.severity);
        previous = Some(pointer);
    }

    // Log the info line, if one exists.
    if let Some(info) = &report.info {
        log_line_info(errors, indentation, info);
    }
    // Write a blank line to visually separate reports:
    _ = writeln!(errors.output.borrow_mut());
}

fn log_pointer(
    errors: &mut Errors,
    previous: Option<&PointedMessage>,
    pointer: &PointedMessage,
    indentation: usize,
    severity: Severity,
) {
    if previous.is_none() || !previous.unwrap().loc.same_file(pointer.loc) {
        // This pointer is not the same as the previous pointer. Print file location as well:
        log_line_file_location(errors, pointer, indentation);
    }
    if pointer.loc.line == 0 {
        // Line being zero means the location is an entire file,
        // not any particular location within the file.
        return;
    }
    if let Some(line) = errors.cache.get_line(pointer.loc) {
        let (line, removed, spaces) = line_spacing(line.to_owned());
        log_line_from_source(errors, pointer, indentation, &line, spaces);
        log_line_carets(errors, pointer, indentation, &line, removed, spaces, severity);
    }
}

/// Log the first line of a report, containing the severity level and the error message.
fn log_line_title(errors: &Errors, report: &LogReport) {
    let line: &[ANSIString<'static>] = &[
        errors
            .styles
            .style(Styled::Tag(report.severity, true))
            .paint(format!("{}", report.severity)),
        errors.styles.style(Styled::Tag(report.severity, false)).paint(format!("({})", report.key)),
        errors.styles.style(Styled::Default).paint(": "),
        errors.styles.style(Styled::ErrorMessage).paint(report.msg.clone()),
    ];
    _ = writeln!(errors.output.borrow_mut(), "{}", ANSIStrings(line));
}

/// Log the optional info line that is part of the overall report.
fn log_line_info(errors: &Errors, indentation: usize, info: &str) {
    let line_info: &[ANSIString<'static>] = &[
        errors.styles.style(Styled::Default).paint(format!("{:width$}", "", width = indentation)),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Location).paint("="),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::InfoTag).paint("Info:"),
        errors.styles.style(Styled::Default).paint(" "),
        errors.styles.style(Styled::Info).paint(info.to_string()),
    ];
    _ = writeln!(errors.output.borrow_mut(), "{}", ANSIStrings(line_info));
}

/// Log the line containing the location's mod name and filename.
fn log_line_file_location(errors: &Errors, pointer: &PointedMessage, indentation: usize) {
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
    _ = writeln!(errors.output.borrow_mut(), "{}", ANSIStrings(line_filename));
}

/// Print a line from the source file.
fn log_line_from_source(
    errors: &Errors,
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
    _ = writeln!(errors.output.borrow_mut(), "{}", ANSIStrings(line_from_source));
}

fn log_line_carets(
    errors: &Errors,
    pointer: &PointedMessage,
    indentation: usize,
    line: &str,
    removed: usize,
    spaces: usize,
    severity: Severity,
) {
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
            width = pointer.length.max(1)
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
    _ = writeln!(errors.output.borrow_mut(), "{}", ANSIStrings(line_carets));
}

pub(crate) fn kind_tag(errors: &Errors, kind: FileKind) -> &str {
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
fn line_spacing(mut line: String) -> (String, usize, usize) {
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
    line.replace_range(..remove, "");
    (line, remove, spaces)
}
