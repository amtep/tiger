//! Collect error reports and then write them out.

use std::borrow::Cow;
use std::cell::RefCell;
use std::cmp::{Ordering, min_by};
use std::fs::read;
use std::io::Write;
use std::iter::{empty, once};
use std::mem::take;
use std::ops::{Bound, RangeBounds};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex, MutexGuard};

use encoding_rs::{UTF_8, WINDOWS_1252};

use crate::helpers::{TigerHashMap, TigerHashSet};
use crate::macros::MACRO_MAP;
use crate::parse::ignore::IgnoreFilter;
use crate::report::error_loc::ErrorLoc;
use crate::report::filter::ReportFilter;
use crate::report::suppress::{Suppression, SuppressionKey};
use crate::report::writer::{log_report, log_summary};
use crate::report::writer_json::log_report_json;
use crate::report::{
    ErrorKey, FilterRule, LogReport, LogReportMetadata, LogReportPointers, LogReportStyle,
    OutputStyle, PointedMessage,
};
use crate::set;
use crate::token::Loc;

/// Error types that should be logged once when consolidating reports
static LOG_ONCE: LazyLock<TigerHashSet<ErrorKey>> = LazyLock::new(|| {
    set!([
        ErrorKey::MissingFile,
        ErrorKey::MissingItem,
        ErrorKey::MissingLocalization,
        ErrorKey::MissingPerspective,
        ErrorKey::MissingSound,
    ])
});

static ERRORS: LazyLock<Mutex<Errors>> = LazyLock::new(|| Mutex::new(Errors::default()));

#[allow(missing_debug_implementations)]
#[derive(Default)]
pub struct Errors<'a> {
    /// Extra loaded mods' error tags.
    pub(crate) loaded_mods_labels: Vec<String>,

    /// Loaded DLCs' error tags.
    pub(crate) loaded_dlcs_labels: Vec<String>,

    pub(crate) cache: Cache,

    /// Determines whether a report should be printed.
    pub(crate) filter: ReportFilter,

    /// Output color and style configuration.
    pub(crate) styles: OutputStyle,

    pub(crate) suppress: TigerHashMap<SuppressionKey<'a>, Vec<Suppression>>,
    // The range is decomposed into its start and end bounds in order to
    // avoid dyn shenanigans with the RangeBounds trait.
    ignore: TigerHashMap<&'a Path, Vec<IgnoreEntry>>,

    /// All reports that passed the checks, stored here to be sorted before being emitted all at once.
    /// The "abbreviated" reports don't participate in this. They are still emitted immediately.
    /// It's a `HashSet` because duplicate reports are fairly common due to macro expansion and other revalidations.
    storage: TigerHashMap<LogReportMetadata, TigerHashSet<LogReportPointers>>,
}

impl Errors<'_> {
    fn should_suppress(&self, report: &LogReportMetadata, pointers: &LogReportPointers) -> bool {
        let key = SuppressionKey { key: report.key, message: Cow::Borrowed(&report.msg) };
        if let Some(v) = self.suppress.get(&key) {
            for suppression in v {
                if suppression.len() != pointers.len() {
                    continue;
                }
                for (s, p) in suppression.iter().zip(pointers.iter()) {
                    if s.path == p.loc.pathname()
                        && s.tag == p.msg
                        && s.line.as_deref() == self.cache.get_line(p.loc)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn should_ignore(&self, report: &LogReportMetadata, pointers: &LogReportPointers) -> bool {
        for p in pointers {
            if let Some(vec) = self.ignore.get(p.loc.pathname()) {
                for entry in vec {
                    if (entry.start, entry.end).contains(&p.loc.line)
                        && entry.filter.matches(report.key, &report.msg)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Perform some checks to see whether the report should actually be logged.
    /// If yes, it will add it to the storage.
    fn push_report(&mut self, report: LogReportMetadata, pointers: LogReportPointers) {
        if !self.filter.should_print_report(&report, &pointers)
            || self.should_suppress(&report, &pointers)
        {
            return;
        }
        self.storage.entry(report).or_default().insert(pointers);
    }

    /// Extract the stored reports, sort them, and return them as a vector.
    pub fn flatten_reports(
        &self,
        consolidate: bool,
    ) -> Vec<(&LogReportMetadata, Cow<'_, LogReportPointers>, usize)> {
        let mut reports: Vec<_> = self
            .storage
            .iter()
            .flat_map(|(report, occurrences)| -> Box<dyn Iterator<Item = _>> {
                let mut iterator =
                    occurrences.iter().filter(|pointers| !self.should_ignore(report, pointers));
                match report.style {
                    LogReportStyle::Full => {
                        if consolidate && LOG_ONCE.contains(&report.key) {
                            if let Some(initial) = iterator.next() {
                                let (pointers, additional_count) = iterator.fold(
                                    (initial, 0usize),
                                    |(first_occurrence, count), e| {
                                        (
                                            min_by(first_occurrence, e, |a, b| {
                                                a.iter().map(|e| e.loc).cmp(b.iter().map(|e| e.loc))
                                            }),
                                            count + 1,
                                        )
                                    },
                                );
                                Box::new(once((report, Cow::Borrowed(pointers), additional_count)))
                            } else {
                                Box::new(empty())
                            }
                        } else {
                            Box::new(
                                iterator.map(move |pointers| (report, Cow::Borrowed(pointers), 0)),
                            )
                        }
                    }
                    LogReportStyle::Abbreviated => {
                        let mut pointers: Vec<_> = iterator.map(|o| o[0].clone()).collect();
                        pointers.sort_unstable_by_key(|p| p.loc);
                        Box::new(once((report, Cow::Owned(pointers), 0)))
                    }
                }
            })
            .collect();
        reports.sort_unstable_by(|(a, ap, _), (b, bp, _)| {
            // Severity in descending order
            let mut cmp = b.severity.cmp(&a.severity);
            if cmp != Ordering::Equal {
                return cmp;
            }
            // Confidence in descending order too
            cmp = b.confidence.cmp(&a.confidence);
            if cmp != Ordering::Equal {
                return cmp;
            }
            // If severity and confidence are the same, order by loc.
            cmp = ap.iter().map(|e| e.loc).cmp(bp.iter().map(|e| e.loc));
            // Fallback: order by message text.
            if cmp == Ordering::Equal {
                cmp = a.msg.cmp(&b.msg);
            }
            cmp
        });
        reports
    }

    /// Print the stored reports.
    /// Set `json` if they should be printed as a JSON array. Otherwise they are printed in the
    /// default output format.
    ///
    /// Note that the default output format is not stable across versions. It is meant for human
    /// readability and occasionally gets changed to improve that.
    ///
    /// Reports matched by `#tiger-ignore` directives will not be printed.
    ///
    /// Returns true iff any reports were printed.
    pub fn emit_reports<O: Write + Send>(
        &mut self,
        output: &mut O,
        json: bool,
        consolidate: bool,
        summary: bool,
    ) -> bool {
        let reports = self.flatten_reports(consolidate);
        let result = !reports.is_empty();
        if json {
            _ = writeln!(output, "[");
            let mut first = true;
            for (report, pointers, _) in &reports {
                if !first {
                    _ = writeln!(output, ",");
                }
                first = false;
                log_report_json(self, output, report, pointers);
            }
            _ = writeln!(output, "\n]");
        } else {
            for (report, pointers, additional) in &reports {
                log_report(self, output, report, pointers, *additional);
            }
            if summary {
                log_summary(output, &self.styles, &reports);
            }
        }
        self.storage.clear();
        result
    }

    pub fn store_source_file(&mut self, fullpath: PathBuf, source: &'static str) {
        self.cache.filecache.borrow_mut().insert(fullpath, source);
    }

    /// Get a mutable lock on the global ERRORS struct.
    ///
    /// # Panics
    /// May panic when the mutex has been poisoned by another thread.
    pub fn get_mut() -> MutexGuard<'static, Errors<'static>> {
        ERRORS.lock().unwrap()
    }

    /// Like [`Errors::get_mut`] but intended for read-only access.
    ///
    /// Currently there is no difference, but if the locking mechanism changes there may be a
    /// difference.
    ///
    /// # Panics
    /// May panic when the mutex has been poisoned by another thread.
    pub fn get() -> MutexGuard<'static, Errors<'static>> {
        ERRORS.lock().unwrap()
    }
}

#[derive(Debug, Default)]
pub(crate) struct Cache {
    /// Files that have been read in to get the lines where errors occurred.
    /// Cached here to avoid duplicate I/O and UTF-8 parsing.
    filecache: RefCell<TigerHashMap<PathBuf, &'static str>>,

    /// Files that have been linesplit, cached to avoid doing that work again
    linecache: RefCell<TigerHashMap<PathBuf, Vec<&'static str>>>,
}

impl Cache {
    /// Fetch the contents of a single line from a script file.
    pub(crate) fn get_line(&self, loc: Loc) -> Option<&'static str> {
        let mut filecache = self.filecache.borrow_mut();
        let mut linecache = self.linecache.borrow_mut();

        if loc.line == 0 {
            return None;
        }
        let fullpath = loc.fullpath();
        if let Some(lines) = linecache.get(fullpath) {
            return lines.get(loc.line as usize - 1).copied();
        }
        if let Some(contents) = filecache.get(fullpath) {
            let lines: Vec<_> = contents.lines().collect();
            let line = lines.get(loc.line as usize - 1).copied();
            linecache.insert(fullpath.to_path_buf(), lines);
            return line;
        }
        let bytes = read(fullpath).ok()?;
        // Try decoding it as UTF-8. If that succeeds without errors, use it, otherwise fall back
        // to WINDOWS_1252. The decode method will do BOM stripping.
        let contents = match UTF_8.decode(&bytes) {
            (contents, _, false) => contents,
            (_, _, true) => WINDOWS_1252.decode(&bytes).0,
        };
        let contents = contents.into_owned().leak();
        filecache.insert(fullpath.to_path_buf(), contents);

        let lines: Vec<_> = contents.lines().collect();
        let line = lines.get(loc.line as usize - 1).copied();
        linecache.insert(fullpath.to_path_buf(), lines);
        line
    }
}

#[derive(Debug, Clone)]
struct IgnoreEntry {
    start: Bound<u32>,
    end: Bound<u32>,
    filter: IgnoreFilter,
}

/// Record a secondary mod to be loaded before the one being validated.
/// `label` is what it should be called in the error reports; ideally only a few characters long.
pub fn add_loaded_mod_root(label: String) {
    let mut errors = Errors::get_mut();
    errors.loaded_mods_labels.push(label);
}

/// Record a DLC directory from the vanilla installation.
/// `label` is what it should be called in the error reports.
pub fn add_loaded_dlc_root(label: String) {
    let mut errors = Errors::get_mut();
    errors.loaded_dlcs_labels.push(label);
}

/// Store an error report to be emitted when [`emit_reports`] is called.
pub fn log((report, pointers): LogReport) {
    let pointers = pointed_msg_expansion(pointers);
    Errors::get_mut().push_report(report, pointers);
}

/// Expand `Vec<PointedMessage>`.
/// That is; for each `PointedMessage`, follow its location's link until such link is no
/// longer available, adding a newly created `PointedMessage` to the returned `Vec` for each linked
/// location.
fn pointed_msg_expansion(pointers: Vec<PointedMessage>) -> Vec<PointedMessage> {
    pointers
        .into_iter()
        .flat_map(|p| {
            let mut next_loc = Some(p.loc);
            let mut first = true;
            std::iter::from_fn(move || match next_loc {
                Some(mut stack) => {
                    next_loc = stack.link_idx.and_then(|idx| MACRO_MAP.get_loc(idx));
                    stack.link_idx = None;
                    let next = if first {
                        PointedMessage { loc: stack, length: p.length, msg: p.msg.clone() }
                    } else {
                        PointedMessage { loc: stack, length: 1, msg: Some("from here".into()) }
                    };
                    first = false;
                    Some(next)
                }
                None => None,
            })
        })
        .collect()
}

/// Tests whether the report might be printed. If false, the report will definitely not be printed.
pub fn will_maybe_log<E: ErrorLoc>(eloc: E, key: ErrorKey) -> bool {
    Errors::get().filter.should_maybe_print(key, eloc.into_loc())
}

/// Print all the stored reports to the error output.
/// Set `json` if they should be printed as a JSON array. Otherwise they are printed in the
/// default output format.
///
/// Note that the default output format is not stable across versions. It is meant for human
/// readability and occasionally gets changed to improve that.
///
/// Returns true iff any reports were printed.
pub fn emit_reports<O: Write + Send>(
    output: &mut O,
    json: bool,
    consolidate: bool,
    summary: bool,
) -> bool {
    Errors::get_mut().emit_reports(output, json, consolidate, summary)
}

/// Extract the stored reports, sort them, and return them as a hashmap with the occurrences for
/// each instance of metadata split out.
///
/// The stored reports will be left empty.
pub fn take_reports() -> TigerHashMap<LogReportMetadata, TigerHashSet<LogReportPointers>> {
    take(&mut Errors::get_mut().storage)
}

pub fn store_source_file(fullpath: PathBuf, source: &'static str) {
    Errors::get_mut().store_source_file(fullpath, source);
}

pub fn register_ignore_filter<R>(pathname: &'static Path, lines: R, filter: IgnoreFilter)
where
    R: RangeBounds<u32>,
{
    let start = lines.start_bound().cloned();
    let end = lines.end_bound().cloned();
    let entry = IgnoreEntry { start, end, filter };
    Errors::get_mut().ignore.entry(pathname).or_default().push(entry);
}

// =================================================================================================
// =============== Configuration (Output style):
// =================================================================================================

/// Override the default `OutputStyle`. (Controls ansi colors)
pub fn set_output_style(style: OutputStyle) {
    Errors::get_mut().styles = style;
}

/// Disable color in the output.
pub fn disable_ansi_colors() {
    Errors::get_mut().styles = OutputStyle::no_color();
}

// =================================================================================================
// =============== Configuration (Filter):
// =================================================================================================

/// Configure the error reporter to show errors that are in the base game code.
/// Normally those are filtered out, to only show errors that involve the mod's code.
pub fn set_show_vanilla(v: bool) {
    Errors::get_mut().filter.show_vanilla = v;
}

/// Configure the error reporter to show errors that are in extra loaded mods.
/// Normally those are filtered out, to only show errors that involve the mod's code.
pub fn set_show_loaded_mods(v: bool) {
    Errors::get_mut().filter.show_loaded_mods = v;
}

/// Configure the error reporter to only show errors that match this [`FilterRule`].
pub(crate) fn set_predicate(predicate: FilterRule) {
    Errors::get_mut().filter.predicate = predicate;
}
