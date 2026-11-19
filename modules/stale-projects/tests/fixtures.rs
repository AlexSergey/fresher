//! The stale-projects module on its fixture homes: what it must not claim or must keep first, then what it must
//! offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-stale-projects` rewrites `fixtures/golden/stale-projects.json`;
//! the diff is reviewed.

#[cfg(test)]
mod support;

use std::collections::BTreeMap;

use fresher_testing::ModuleRun;

const DEPENDENCIES: &str = "stale-projects.dependencies";
const BUILD_OUTPUT: &str = "stale-projects.build-output";
const PYTHON: &str = "stale-projects.python-environments";
const CACHE: &str = "stale-projects.rust-build-cache";

/// How many items the run left out and their logical bytes, by category and reason.
fn left_out(run: &ModuleRun) -> Vec<(&str, &str, usize, u64)> {
    let mut sums: BTreeMap<(&str, &str), (usize, u64)> = BTreeMap::new();
    for item in &run.left_out {
        let reason = item.reason.as_str().trim_start_matches("stale-projects.left-out.");
        let sum = sums.entry((item.category.as_str(), reason)).or_default();
        *sum = (sum.0 + 1, sum.1 + item.sizes.logical.bytes());
    }
    sums.into_iter().map(|((category, reason), (count, bytes))| (category, reason, count, bytes)).collect()
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};

    use crate::support::{System, kept, kept_with, not_claimed, offered_with};
    use crate::{BUILD_OUTPUT, CACHE, DEPENDENCIES, PYTHON, left_out};

    #[test]
    fn nothing_in_the_not_claimed_home_is_claimed_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn each_kept_item_is_left_out_with_its_reason() {
        let (_tree, run) = kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (BUILD_OUTPUT, "recent", 1, 512),
                (DEPENDENCIES, "recent", 4, 2048),
                (DEPENDENCIES, "unread", 1, 512),
                (PYTHON, "recent", 1, 512),
                (CACHE, "cache-recent", 1, 512),
            ]
        );
    }

    #[test]
    fn a_restored_run_keeps_everything_to_scan_again() {
        let (_tree, run) = kept_with(System::kept().restored());

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                (BUILD_OUTPUT, "scan-again", 1, 512),
                (DEPENDENCIES, "scan-again", 5, 2560),
                (PYTHON, "scan-again", 1, 512),
                (CACHE, "scan-again", 1, 512)
            ]
        );
    }

    #[test]
    fn a_restored_run_of_the_offered_home_offers_nothing() {
        let (_tree, run) = offered_with(System::plain().restored());

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(run.left_out.len(), 17);
        assert!(run.left_out.iter().all(|item| item.reason.as_str() == "stale-projects.left-out.scan-again"));
    }
}

mod positive {
    use std::collections::BTreeMap;

    use fresher_core::{DangerLevel, Removal, StoredSettings, Subject, Text, TextArg, TextKey};

    use crate::support::{System, fixtures, kept, not_claimed, offered, offered_with, scan_with};
    use crate::{BUILD_OUTPUT, CACHE, DEPENDENCIES, PYTHON, identities, left_out};

    const DEPS: &str = "stale-projects.reason.dependencies";
    const BUILD: &str = "stale-projects.reason.build-output";
    const VENV: &str = "stale-projects.reason.python-environments";
    const CACHE_REASON: &str = "stale-projects.reason.rust-build-cache";

    /// Per finding: its identity, its category, its danger, the key of its reason, its project and its folder in the
    /// project (the title's two parts).
    const EXPECTED: [(&str, &str, DangerLevel, &str, &str, &str); 15] = [
        ("build-output/Code/android-old/.gradle", BUILD_OUTPUT, DangerLevel::Low, BUILD, "android-old", ".gradle"),
        ("build-output/Code/android-old/build", BUILD_OUTPUT, DangerLevel::Low, BUILD, "android-old", "build"),
        ("build-output/Code/dart-old/.dart_tool", BUILD_OUTPUT, DangerLevel::Low, BUILD, "dart-old", ".dart_tool"),
        ("build-output/Code/dart-old/build", BUILD_OUTPUT, DangerLevel::Low, BUILD, "dart-old", "build"),
        ("build-output/Code/java-old/target", BUILD_OUTPUT, DangerLevel::Low, BUILD, "java-old", "target"),
        ("build-output/Code/swift-old/.build", BUILD_OUTPUT, DangerLevel::Low, BUILD, "swift-old", ".build"),
        ("build-output/Code/tool-old/target", BUILD_OUTPUT, DangerLevel::Low, BUILD, "tool-old", "target"),
        ("dependencies/Code/shop/node_modules", DEPENDENCIES, DangerLevel::Low, DEPS, "shop", "node_modules"),
        (
            "dependencies/Code/shop/packages/web/node_modules",
            DEPENDENCIES,
            DangerLevel::Low,
            DEPS,
            "shop",
            "packages/web/node_modules",
        ),
        ("dependencies/Code/web-old/node_modules", DEPENDENCIES, DangerLevel::Low, DEPS, "web-old", "node_modules"),
        ("python-environments/Code/py-old/.venv", PYTHON, DangerLevel::Medium, VENV, "py-old", ".venv"),
        ("python-environments/Code/py-venv/venv", PYTHON, DangerLevel::Medium, VENV, "py-venv", "venv"),
        (
            "rust-build-cache/Code/fresher/target/aarch64-apple-darwin/debug/incremental",
            CACHE,
            DangerLevel::Low,
            CACHE_REASON,
            "fresher",
            "target/aarch64-apple-darwin/debug/incremental",
        ),
        (
            "rust-build-cache/Code/fresher/target/debug/incremental",
            CACHE,
            DangerLevel::Low,
            CACHE_REASON,
            "fresher",
            "target/debug/incremental",
        ),
        (
            "rust-build-cache/Code/fresher/target/llvm-cov-target",
            CACHE,
            DangerLevel::Low,
            CACHE_REASON,
            "fresher",
            "target/llvm-cov-target",
        ),
    ];

    /// The offers come through the real walk and the projects the engine finds: a mismatch between the walk's paths
    /// and the module's would show here as nothing offered.
    #[test]
    fn every_category_is_offered_at_its_danger() {
        let (tree, run) = offered();

        let mut expected: Vec<&str> = EXPECTED.iter().map(|(identity, ..)| *identity).collect();
        expected.sort_unstable();
        assert_eq!(identities(&run), expected);
        for (identity, category, danger, reason, project, folder) in EXPECTED {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{identity}");
            assert_eq!(found.reason().key().as_str(), reason, "{identity}");
            let title = Text::new(TextKey::new("stale-projects.title").unwrap())
                .with("project", TextArg::Text(project.into()))
                .with("folder", TextArg::Text(folder.into()));
            assert_eq!(found.title(), &title, "{identity}");
            let path = tree.root().join(identity.split_once('/').unwrap().1);
            assert_eq!(found.subject(), &Subject::Path(path.clone()), "{identity}");
            assert_eq!(found.removal(), &Removal::MoveToTrash(path), "{identity}");
        }
    }

    #[test]
    fn an_active_projects_target_is_left_out_with_its_sizes_less_the_three_caches() {
        let (_tree, run) = offered();

        assert_eq!(left_out(&run), [(BUILD_OUTPUT, "recent", 1, 8192)]);
    }

    #[test]
    fn a_stale_projects_target_is_offered_once_and_its_cache_not_again() {
        let (_tree, run) = offered();

        let tool_old: Vec<&str> =
            identities(&run).into_iter().filter(|identity| identity.contains("tool-old")).collect();

        assert_eq!(tool_old, ["build-output/Code/tool-old/target"]);
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        let (_tree, walked) = offered();
        let (_tree, reversed) = offered_with(System::plain().reversed());

        assert_eq!(reversed.golden().unwrap(), walked.golden().unwrap());
    }

    #[test]
    fn the_setting_moves_the_days() {
        let settings =
            StoredSettings { values: BTreeMap::from([("stale-projects.unused-for-days".into(), 500.into())]) };

        let (_tree, run) = scan_with("stale-projects-offered", System::plain(), settings);

        // The 400-day projects are kept; only the caches, not tied to the setting, are offered.
        assert_eq!(
            identities(&run),
            [
                "rust-build-cache/Code/fresher/target/aarch64-apple-darwin/debug/incremental",
                "rust-build-cache/Code/fresher/target/debug/incremental",
                "rust-build-cache/Code/fresher/target/llvm-cov-target",
                "rust-build-cache/Code/tool-old/target/debug/incremental",
            ]
        );
        assert_eq!(run.left_out.len(), 13);
        assert!(left_out(&run).iter().all(|(_, reason, ..)| *reason == "recent"));
    }

    /// The report of every home, by home.
    #[test]
    fn the_report_matches_the_golden_file() {
        let runs = [("not-claimed", not_claimed().1), ("kept", kept().1), ("offered", offered().1)];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/stale-projects.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
