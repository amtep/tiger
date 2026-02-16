use std::io::Write;

use serde_json::json;

use crate::fileset::FileStage;
use crate::report::errors::Errors;
use crate::report::writer::kind_tag;
use crate::report::{LogReportMetadata, LogReportPointers};

/// Log the report in JSON format.
pub fn log_report_json<O: Write + Send>(
    errors: &Errors,
    output: &mut O,
    report: &LogReportMetadata,
    pointers: &LogReportPointers,
) {
    let pointers: Vec<_> = pointers
        .iter()
        .map(|pointer| {
            let path = pointer.loc.pathname();
            let fullpath = pointer.loc.fullpath();
            json!({
                "path": path,
                "from": kind_tag(errors, pointer.loc.kind),
                "stage": stage_desc(pointer.loc.stage),
                "fullpath": fullpath,
                "linenr": if pointer.loc.line == 0 { None } else { Some(pointer.loc.line) },
                "column": if pointer.loc.column == 0 { None } else { Some(pointer.loc.column) },
                "length": if pointer.length == 0 { None } else { Some(pointer.length) },
                "line": errors.cache.get_line(pointer.loc),
                "tag": pointer.msg,
            })
        })
        .collect();
    let report = json!({
        "severity": report.severity,
        "confidence": report.confidence,
        "key": report.key,
        "message": &report.msg,
        "info": &report.info,
        "wiki": &report.wiki,
        "locations": pointers,
    });

    if let Err(e) = serde_json::to_writer_pretty(output, &report) {
        eprintln!("JSON error: {e:#}");
    }
}

fn stage_desc(stage: FileStage) -> Option<&'static str> {
    match stage {
        #[cfg(feature = "eu5")]
        FileStage::LoadingScreen => Some("loading_screen"),
        #[cfg(feature = "eu5")]
        FileStage::MainMenu => Some("main_menu"),
        #[cfg(feature = "eu5")]
        FileStage::InGame => Some("in_game"),
        FileStage::NoStage => None,
    }
}
