use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use tiger_lib::{take_reports, Everything, LogReport};

static TEST_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn check_mod_helper(modname: &str) -> Vec<LogReport> {
    let _guard = TEST_MUTEX.lock().unwrap();

    let vanilla_dir = PathBuf::from("tests/files/ck3");
    let mod_root = PathBuf::from(format!("tests/files/{}", modname));

    let mut everything =
        Everything::new(None, Some(&vanilla_dir), None, None, &mod_root, Vec::new()).unwrap();
    everything.load_all();
    everything.validate_all();

    take_reports()
}

fn take_report_contains(
    vec: &mut Vec<LogReport>,
    pathname: &str,
    msg_contains: &str,
) -> Option<LogReport> {
    for (i, report) in vec.iter().enumerate() {
        if report.msg.contains(msg_contains)
            && report.pointers[0].loc.pathname() == PathBuf::from(pathname)
        {
            let result = (*report).clone();
            vec.remove(i);
            return Some(result);
        }
    }
    None
}

fn take_report(vec: &mut Vec<LogReport>, pathname: &str, msg: &str) -> Option<LogReport> {
    for (i, report) in vec.iter().enumerate() {
        if report.msg == msg && report.pointers[0].loc.pathname() == PathBuf::from(pathname) {
            let result = (*report).clone();
            vec.remove(i);
            return Some(result);
        }
    }
    None
}

fn take_report_pointer(
    vec: &mut Vec<LogReport>,
    pathname: &str,
    msg: &str,
    line: u32,
    column: u32,
) -> Option<LogReport> {
    for (i, report) in vec.iter().enumerate() {
        if report.msg == msg
            && report.pointers[0].loc.pathname() == PathBuf::from(pathname)
            && report.pointers[0].loc.line == line
            && report.pointers[0].loc.column == column
        {
            let result = (*report).clone();
            vec.remove(i);
            return Some(result);
        }
    }
    None
}

fn ignore_reports(vec: &mut Vec<LogReport>, pathname: &str) {
    let mut i = 0;
    while i < vec.len() {
        let report = &vec[i];
        if report.pointers[0].loc.pathname() == PathBuf::from(pathname) {
            vec.remove(i);
        } else {
            i += 1;
        }
    }
}

#[test]
fn test_mod1() {
    let mut reports = check_mod_helper("mod1");

    let report = take_report(
        &mut reports,
        "localization/english/bad_loca_name.yml",
        "could not determine language from filename",
    );
    report.expect("language from filename test");

    let decisions = "common/decisions/decision.txt";

    let report =
        take_report(&mut reports, decisions, "missing english localization key my_decision");
    report.expect("missing loca key test; decision loca key test");
    let report =
        take_report(&mut reports, decisions, "missing english localization key my_decision_desc");
    report.expect("decision loca key_desc test");
    let report = take_report(
        &mut reports,
        decisions,
        "missing english localization key my_decision_confirm",
    );
    report.expect("decision loca key_confirm test");
    let report = take_report(
        &mut reports,
        decisions,
        "missing english localization key my_decision_tooltip",
    );
    report.expect("decision loca key_tooltip test");

    let report =
        take_report(&mut reports, decisions, "missing english localization key my_decision_also");
    report.expect("decision title field test");
    let report = take_report(
        &mut reports,
        decisions,
        "missing english localization key my_decision2_description",
    );
    report.expect("decision desc field test");
    let report =
        take_report(&mut reports, decisions, "missing english localization key totally_different");
    report.expect("decision selection_tooltip field test");
    let report =
        take_report(&mut reports, decisions, "missing english localization key my_decision2_c");
    report.expect("decision confirm field test");

    let report = take_report(&mut reports, decisions, "file  does not exist");
    let report = report.expect("decision empty picture field test");
    assert!(report.pointers[0].loc.line == 10);

    let events = "events/non-dup.txt";
    let report = take_report(&mut reports, events, "required field `option` missing");
    report.expect("event required field option");
    let report = take_report_contains(&mut reports, events, "duplicate event");
    assert!(report.is_none());

    let events = "events/test-script-values.txt";
    let report = take_report_contains(&mut reports, events, "`else` with a `limit`");
    report.expect("scriptvalue else with a limit");

    dbg!(&reports);
    assert!(reports.is_empty());
}

#[test]
fn test_mod2() {
    let mut reports = check_mod_helper("mod2");

    let interactions = "common/character_interactions/interaction.txt";

    let report = take_report(
        &mut reports,
        interactions,
        "missing english localization key test_interaction",
    );
    report.expect("interaction localization key test");
    let report = take_report(
        &mut reports,
        interactions,
        "missing english localization key test_interaction_extra_icon",
    );
    report.expect("interaction localization key_extra_icon test");
    let report = take_report(&mut reports, interactions, "file gfx/also_missing does not exist");
    let report = report.expect("interaction missing extra_icon file test");
    assert!(report.pointers[0].loc.line == 3);
    let report = take_report(
        &mut reports,
        interactions,
        "file gfx/interface/icons/character_interactions/missing_icon.dds does not exist",
    );
    report.expect("interaction missing icon test");

    let report = take_report(
        &mut reports,
        interactions,
        "you can define localization `test_interaction_desc`",
    );
    report.expect("desc tip missing");

    let report = take_report(&mut reports, interactions, "required field `category` missing");
    report.expect("interaction missing category test");

    let lists = "common/on_action/test-scripted-lists.txt";
    let report =
        take_report(&mut reports, lists, "`courtier_parent` expects scope:child to be set");
    report.expect("scope check for scripted lists");

    dbg!(&reports);
    assert!(reports.is_empty());
}

#[test]
fn test_mod3() {
    let mut reports = check_mod_helper("mod3");

    let single_unmatched = "common/on_action/test-single-unmatched-quote.txt";
    let report =
        take_report_pointer(&mut reports, single_unmatched, "quoted string not closed", 3, 21);
    report.expect("single unmatched quote test");
    ignore_reports(&mut reports, single_unmatched);

    let after_comparator = "common/on_action/test-unmatched-matched-effect.txt";
    let report =
        take_report_pointer(&mut reports, after_comparator, "quoted string not closed", 3, 21);
    report.expect("unmatched quote matched after comparitor test");
    ignore_reports(&mut reports, after_comparator);

    let before_comparator = "common/on_action/test-unmatched-matched-trigger.txt";
    let report =
        take_report_pointer(&mut reports, before_comparator, "quoted string not closed", 5, 21);
    report.expect("unmatched quote matched before comparitor test");
    ignore_reports(&mut reports, before_comparator);

    dbg!(&reports);
    assert!(reports.is_empty());
}
