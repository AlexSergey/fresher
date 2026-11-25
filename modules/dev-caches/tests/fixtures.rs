//! The dev-caches module on its fixture homes: what it must not claim or must keep first, then what it must offer, and
//! its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-dev-caches` rewrites `fixtures/golden/dev-caches.json`; the
//! diff is reviewed.

#[cfg(test)]
mod support;

use fresher_testing::ModuleRun;

/// Each left-out of the run as its category and reason, sorted.
fn left_out(run: &ModuleRun) -> Vec<(&str, &str)> {
    let mut reported: Vec<(&str, &str)> =
        run.left_out.iter().map(|item| (item.category.as_str(), item.reason.as_str())).collect();
    reported.sort_unstable();
    reported
}

/// The identities of the run's findings, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

/// The derived data projects of `dev-caches-caches`.
const DEMO: &str = "Demo-abcdefghijklmnopqrstuvwxyzab";
const MY_APP: &str = "My-App-bcdefghijklmnopqrstuvwxyzabc";

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::FakeRunningApps;

    use crate::left_out;
    use crate::support::{kept, kept_with, not_folders, similar_names, unsigned};

    /// Look-alike names, the other stores of the tools, their settings, credentials and programs, Homebrew's own data
    /// beside its downloads, and derived data folders that are no project.
    #[test]
    fn names_that_only_look_like_a_tools_cache_are_never_claimed() {
        let (_tree, run) = similar_names();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    /// Each place holds the tool's files but not its sign: an invalid tag, a missing tag or a missing name.
    #[test]
    fn a_place_without_its_sign_is_never_claimed() {
        let (_tree, run) = unsigned();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    /// Go's build cache a file; the `CocoaPods` cache and a derived data project links to folders that hold the
    /// sign; a derived data project a file. With no finding and no left-out, nothing names a link or its target: what
    /// the cleanup leaves of them is proven by the sandbox and the core e2e.
    #[test]
    fn a_file_or_a_link_at_a_place_is_never_claimed() {
        let (_tree, run) = not_folders();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    /// npm's cache holds a socket; a file of Gradle's caches is the system's; Cargo's unpacked sources carry the
    /// user's lock; the derived data project waits while Xcode is open.
    #[test]
    fn a_cache_holding_a_socket_another_owners_entry_or_a_lock_or_of_an_open_app_is_kept() {
        let (_tree, run) = kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                ("dev-caches.cargo", "dev-caches.left-out.not-yours"),
                ("dev-caches.gradle", "dev-caches.left-out.not-yours"),
                ("dev-caches.npm", "dev-caches.left-out.special"),
                ("dev-caches.xcode-derived-data", "dev-caches.left-out.app-open"),
            ]
        );
    }

    #[test]
    fn open_apps_that_cannot_be_listed_keep_the_derived_data_and_change_nothing_else() {
        let (_tree, run) = kept_with(FakeRunningApps::failing("no process list"));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                ("dev-caches.cargo", "dev-caches.left-out.not-yours"),
                ("dev-caches.gradle", "dev-caches.left-out.not-yours"),
                ("dev-caches.npm", "dev-caches.left-out.special"),
                ("dev-caches.xcode-derived-data", "dev-caches.left-out.no-system"),
            ]
        );
    }
}

mod positive {
    use fresher_core::{BundleId, DangerLevel, Removal, Subject, TextArg};

    use crate::support::{System, caches, fixtures, kept, not_folders, scan, similar_names, unsigned};
    use crate::{DEMO, MY_APP, identities, left_out};

    /// Per finding: its identity, its category, its danger, the tool its reason names, whether the tool downloads
    /// (rather than builds) it again, and its folder in the home.
    const EXPECTED: [(&str, &str, DangerLevel, &str, bool, &str); 9] = [
        ("cargo-cache/cache", "dev-caches.cargo", DangerLevel::Low, "Cargo", true, ".cargo/registry/cache"),
        ("cargo-src/src", "dev-caches.cargo", DangerLevel::Low, "Cargo", true, ".cargo/registry/src"),
        (
            "cocoapods/CocoaPods",
            "dev-caches.cocoapods",
            DangerLevel::Low,
            "CocoaPods",
            true,
            "Library/Caches/CocoaPods",
        ),
        ("go-build/go-build", "dev-caches.go-build", DangerLevel::Safe, "Go", false, "Library/Caches/go-build"),
        ("gradle/caches", "dev-caches.gradle", DangerLevel::Low, "Gradle", true, ".gradle/caches"),
        (
            "homebrew/downloads",
            "dev-caches.homebrew",
            DangerLevel::Low,
            "Homebrew",
            true,
            "Library/Caches/Homebrew/downloads",
        ),
        ("npm/_cacache", "dev-caches.npm", DangerLevel::Low, "npm", true, ".npm/_cacache"),
        ("pip/pip", "dev-caches.pip", DangerLevel::Low, "pip", true, "Library/Caches/pip"),
        ("yarn/Yarn", "dev-caches.yarn", DangerLevel::Low, "Yarn", true, "Library/Caches/Yarn"),
    ];

    #[test]
    fn every_tools_cache_is_offered_in_its_category_at_its_danger_with_its_title_and_reason() {
        let (tree, run) = caches();

        let mut expected: Vec<String> = EXPECTED.iter().map(|(identity, ..)| (*identity).to_owned()).collect();
        expected.extend([format!("xcode-derived-data/{DEMO}"), format!("xcode-derived-data/{MY_APP}")]);
        expected.sort_unstable();
        assert_eq!(identities(&run), expected);
        assert_eq!(left_out(&run), []);
        for (identity, category, danger, tool, download, place) in EXPECTED {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            let row = identity.split('/').next().unwrap();
            let reason =
                if download { "dev-caches.finding.reason-download" } else { "dev-caches.finding.reason-build" };
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{identity}");
            assert_eq!(found.title().key().as_str(), format!("dev-caches.title.{row}"), "{identity}");
            assert_eq!(found.reason().key().as_str(), reason, "{identity}");
            assert_eq!(found.reason().args().get("tool"), Some(&TextArg::Text(tool.into())), "{identity}");
            assert_eq!(found.app(), None, "{identity}");
            let folder = tree.root().join(place);
            assert_eq!(found.subject(), &Subject::Path(folder.clone()), "{identity}");
            assert_eq!(found.removal(), &Removal::MoveToTrash(folder), "{identity}");
        }
    }

    #[test]
    fn each_derived_data_project_is_offered_for_xcode_titled_by_its_project() {
        let (tree, run) = caches();

        for (name, project) in [(DEMO, "Demo"), (MY_APP, "My-App")] {
            let identity = format!("xcode-derived-data/{name}");
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            let category = "dev-caches.xcode-derived-data";
            assert_eq!((found.category().as_str(), found.danger()), (category, DangerLevel::Safe), "{name}");
            assert_eq!(found.title().key().as_str(), "dev-caches.title.xcode-derived-data", "{name}");
            assert_eq!(found.title().args().get("project"), Some(&TextArg::Text(project.into())), "{name}");
            assert_eq!(found.reason().key().as_str(), "dev-caches.finding.reason-build", "{name}");
            assert_eq!(found.reason().args().get("tool"), Some(&TextArg::Text("Xcode".into())), "{name}");
            assert_eq!(found.app().map(BundleId::as_str), Some("com.apple.dt.Xcode"), "{name}");
            let folder = tree.root().join("Library/Developer/Xcode/DerivedData").join(name);
            assert_eq!(found.removal(), &Removal::MoveToTrash(folder), "{name}");
        }
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        let homes = [
            "dev-caches-similar-names",
            "dev-caches-unsigned",
            "dev-caches-not-folders",
            "dev-caches-kept",
            "dev-caches-caches",
        ];
        for name in homes {
            let (_tree, walked) = scan(name, System::plain());
            let (_tree, reversed) = scan(name, System::plain().reversed());

            assert_eq!(walked.golden().unwrap(), reversed.golden().unwrap(), "{name}");
        }
    }

    /// The report of every home with its default system (the kept home with Xcode open), by home.
    #[test]
    fn the_report_matches_the_golden_file() {
        let runs = [
            ("similar-names", similar_names().1),
            ("unsigned", unsigned().1),
            ("not-folders", not_folders().1),
            ("kept", kept().1),
            ("caches", caches().1),
        ];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/dev-caches.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
