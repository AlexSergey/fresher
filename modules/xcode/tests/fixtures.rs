//! The xcode module on its fixture homes: what it must not claim or must keep first, then what it must offer, and its
//! golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-xcode` rewrites `fixtures/golden/xcode.json`; the diff is
//! reviewed.

#[cfg(test)]
mod support;

use std::collections::BTreeMap;

use fresher_testing::ModuleRun;

/// How many items the run left out, by reason.
fn left_out(run: &ModuleRun) -> Vec<(&str, usize)> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for item in &run.left_out {
        *counts.entry(item.reason.as_str().trim_start_matches("xcode.left-out.")).or_default() += 1;
    }
    counts.into_iter().collect()
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};

    use crate::left_out;
    use crate::support::{kept_no_system, kept_simulator_open, kept_xcode_open, not_claimed};

    #[test]
    fn nothing_in_the_not_claimed_home_is_offered_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn derived_data_and_simulator_devices_are_not_this_module_s() {
        let (tree, run) = not_claimed();

        let developer = tree.root().join("Library/Developer");
        assert!(developer.join("Xcode/DerivedData/App-abcdefghijklmnopqrstuvwxyzab/info.plist").is_file());
        assert!(developer.join("CoreSimulator/Devices").is_dir());
        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
    }

    #[test]
    fn the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason() {
        let runs = [
            ("xcode open", kept_xcode_open().1, ("app-open", 2)),
            ("simulator open", kept_simulator_open().1, ("app-open", 2)),
            ("no system", kept_no_system().1, ("no-system", 2)),
        ];

        for (name, run, reached_the_question) in runs {
            assert_eq!(run.findings, Vec::<NewFinding>::new(), "{name}");
            assert_eq!(left_out(&run), [reached_the_question, ("not-yours", 1), ("special", 1)], "{name}");
        }
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, NewFinding, Removal, Subject, TextArg};
    use fresher_testing::ModuleRun;

    use crate::left_out;
    use crate::support::{fixtures, kept_no_system, kept_simulator_open, kept_xcode_open, not_claimed, offered};

    fn find<'a>(run: &'a ModuleRun, identity: &str) -> Option<&'a NewFinding> {
        run.findings.iter().find(|finding| finding.key().identity() == identity)
    }

    #[test]
    fn the_offered_home_offers_every_kind_in_its_category() {
        let (tree, run) = offered();

        assert_eq!(left_out(&run), []);
        let developer = tree.root().join("Library/Developer");
        let expected = [
            ("ios-device-support/15.0 (19A346)", "xcode.device-support", DangerLevel::Low),
            ("ios-device-support/16.4.1 (20E252)", "xcode.device-support", DangerLevel::Low),
            ("ios-device-support/iPhone15,2 17.2 (21C62)", "xcode.device-support", DangerLevel::Low),
            ("watchos-device-support/Watch6,1 10.0 (21R355)", "xcode.device-support", DangerLevel::Low),
            ("previews", "xcode.previews", DangerLevel::Safe),
            ("simulator-caches", "xcode.simulator-caches", DangerLevel::Safe),
            ("archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive", "xcode.archives", DangerLevel::High),
        ];
        assert_eq!(run.findings.len(), expected.len());
        for (identity, category, danger) in expected {
            let found = find(&run, identity).unwrap();
            assert_eq!(found.category().as_str(), category, "{identity}");
            assert_eq!(found.danger(), danger, "{identity}");
            assert_eq!(found.app().map(BundleId::as_str), Some("com.apple.dt.Xcode"), "{identity}");
            assert!(matches!(found.subject(), Subject::Path(path) if path.starts_with(&developer)), "{identity}");
        }
    }

    #[test]
    fn the_two_newest_versions_have_the_recent_reason() {
        let (_tree, run) = offered();

        let reason = |identity: &str| find(&run, identity).unwrap().reason().key().as_str().to_owned();

        assert_eq!(reason("ios-device-support/15.0 (19A346)"), "xcode.reason.device-support");
        assert_eq!(reason("ios-device-support/16.4.1 (20E252)"), "xcode.reason.device-support-recent");
        assert_eq!(reason("ios-device-support/iPhone15,2 17.2 (21C62)"), "xcode.reason.device-support-recent");
        assert_eq!(reason("watchos-device-support/Watch6,1 10.0 (21R355)"), "xcode.reason.device-support-recent");
        let version =
            find(&run, "ios-device-support/iPhone15,2 17.2 (21C62)").unwrap().title().args().get("version").cloned();
        assert_eq!(version, Some(TextArg::Text("17.2".into())));
    }

    #[test]
    fn the_caches_are_emptied_and_the_rest_moved_to_the_trash() {
        let (tree, run) = offered();

        let developer = tree.root().join("Library/Developer");
        for finding in &run.findings {
            let identity = finding.key().identity();
            match finding.removal() {
                Removal::EmptyFolder(path) => {
                    assert!(identity == "previews" || identity == "simulator-caches", "{identity}");
                    assert!(path.starts_with(&developer), "{identity}");
                    assert_eq!(finding.subject(), &Subject::Path(path.clone()), "{identity}");
                }
                Removal::MoveToTrash(path) => {
                    assert!(identity.contains("device-support/") || identity.starts_with("archives/"), "{identity}");
                    assert!(path.starts_with(&developer), "{identity}");
                    assert_eq!(finding.subject(), &Subject::Path(path.clone()), "{identity}");
                }
                other => assert_eq!(format!("{other:?}"), "a Trash or an EmptyFolder removal"),
            }
        }
    }

    /// The report of every run, by name.
    #[test]
    fn the_golden_report_matches() {
        let runs = [
            ("not-claimed", not_claimed().1),
            ("kept-xcode-open", kept_xcode_open().1),
            ("kept-simulator-open", kept_simulator_open().1),
            ("kept-no-system", kept_no_system().1),
            ("offered", offered().1),
        ];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/xcode.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
