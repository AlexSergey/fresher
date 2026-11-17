//! The autotest of every case file of `fixtures/sandboxes/`, through the app's own sandbox composition: what is
//! offered and with which danger level, what a cleanup does and what it leaves (docs/plans/007-safe-development/
//! 00-design.md, "Autotests from the case files"). Sandbox mode exists in debug builds only, and so do these tests.
//!
//! `npm run sandbox:test` runs them; `UPDATE_GOLDEN=1 npm run sandbox:test` rewrites `fixtures/golden/sandbox-*.json`,
//! and the diff is reviewed. A new case file gets its line in `case_files!` below.

#![cfg(debug_assertions)]

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
#[path = "sandboxes/harness.rs"]
mod harness;

/// One `#[test]` per case file: `test_name => "file name without .toml"`, sorted by file name.
macro_rules! case_files {
    ($($test:ident => $file:literal),* $(,)?) => {
        /// The case files that have a test.
        const CASE_FILES: &[&str] = &[$($file),*];

        $(
            #[test]
            fn $test() {
                crate::harness::check_case_file($file);
            }
        )*
    };
}

mod negative {
    use crate::harness::run;

    /// A `trial` positive written into `home/work`, next to a marker, with this `expect` table.
    fn trial_positive(name: &str, expect: &str) -> String {
        format!(
            "[[case]]\nname = \"{name}\"\nkind = \"positive\"\n[[case.marker]]\npath = \"home/work\"\n\
             [[case.file]]\npath = \"home/work/fresher-trial-cache/blob.bin\"\nsize = 4096\n[case.expect]\n{expect}"
        )
    }

    const CACHE_IN_TRASH: &str = "item = \"home/work/fresher-trial-cache\"\ncategory = \"trial.caches\"\ndanger = \"safe\"\nafter-cleanup = \"in-trash\"\n";

    #[test]
    fn a_broken_expectation_fails_with_its_case_name() {
        let expect = CACHE_IN_TRASH.replace("trial.caches", "trial.kept").replace("safe", "low");
        let text = format!("feature = \"trial\"\nmodule = \"trial\"\n{}", trial_positive("a cache", &expect));

        let failure = run("broken", &text).unwrap_err();

        assert_eq!(failure.case, "a cache", "{failure}");
        assert!(failure.problem.contains("trial.caches"), "{failure}");
    }

    #[test]
    fn a_danger_other_than_the_findings_fails() {
        let expect = CACHE_IN_TRASH.replace("\"safe\"", "\"low\"");
        let text = format!("feature = \"trial\"\nmodule = \"trial\"\n{}", trial_positive("a raised cache", &expect));

        let failure = run("danger", &text).unwrap_err();

        assert_eq!(failure.case, "a raised cache", "{failure}");
        assert!(failure.problem.contains("offered as Safe, not as Low"), "{failure}");
    }

    #[test]
    fn a_wrong_after_cleanup_fails() {
        let expect = CACHE_IN_TRASH.replace("in-trash", "emptied");
        let text = format!("feature = \"trial\"\nmodule = \"trial\"\n{}", trial_positive("an emptied cache", &expect));

        let failure = run("after", &text).unwrap_err();

        assert_eq!(failure.case, "an emptied cache", "{failure}");
        assert!(failure.problem.contains("expected Emptied"), "{failure}");
    }

    #[test]
    fn a_negative_that_is_offered_fails() {
        let text = "feature = \"trial\"\nmodule = \"trial\"\n[[case]]\nname = \"a marked cache\"\nkind = \"negative\"\n\
                    [[case.marker]]\npath = \"home/work\"\n[[case.file]]\n\
                    path = \"home/work/fresher-trial-cache/blob.bin\"\nsize = 4096\n";

        let failure = run("offered", text).unwrap_err();

        assert_eq!(failure.case, "a marked cache", "{failure}");
        assert!(failure.problem.contains("offered: home/work/fresher-trial-cache"), "{failure}");
    }

    /// The positive's item holds the negative's folder: the scan offers nothing at or below the negative, but the
    /// cleanup takes it along.
    #[test]
    fn a_negative_that_changes_fails() {
        let negative = "[[case]]\nname = \"a file inside the cache\"\nkind = \"negative\"\n[[case.file]]\n\
                        path = \"home/work/fresher-trial-cache/inner/keep.bin\"\nsize = 4096\n";
        let text = format!(
            "feature = \"trial\"\nmodule = \"trial\"\n{negative}{}",
            trial_positive("the cache", CACHE_IN_TRASH)
        );

        let failure = run("changed", &text).unwrap_err();

        assert_eq!(failure.case, "a file inside the cache", "{failure}");
        assert!(failure.problem.contains("changed by the cleanup"), "{failure}");
    }

    #[test]
    fn a_refused_case_file_names_the_file() {
        let failure = run("unreadable", "feature = 1\n").unwrap_err();

        assert_eq!(failure.case, "unreadable", "{failure}");
    }
}

mod positive {
    use std::collections::BTreeSet;

    use fresher_sandbox::CaseFile;

    use crate::harness::{case_file_names, read_case_file, registered_modules, run};

    case_files! {
        cleanup_guards => "cleanup-guards",
        trial => "trial",
    }

    /// A module whose findings are made up rather than read from the disk: no sandbox can show them.
    const NOT_ON_DISK: &[&str] = &["demo"];

    #[test]
    fn every_case_file_has_a_test() {
        assert_eq!(CASE_FILES, case_file_names(), "every file of fixtures/sandboxes/ has a line in case_files!");
    }

    #[test]
    fn every_registered_module_has_a_case_file() {
        let covered: BTreeSet<String> = case_file_names()
            .iter()
            .filter_map(|name| CaseFile::parse(&read_case_file(name)).unwrap().module())
            .map(|module| module.as_str().to_owned())
            .collect();

        let missing: Vec<String> = registered_modules()
            .into_iter()
            .filter(|module| !NOT_ON_DISK.contains(&module.as_str()) && !covered.contains(module))
            .collect();

        assert!(missing.is_empty(), "registered modules without a case file in fixtures/sandboxes/: {missing:?}");
    }

    /// The harness's own run of a passing case file, apart from the files of `fixtures/sandboxes/`.
    #[test]
    fn a_case_file_that_holds_passes() {
        let text = r#"
feature = "trial"
module = "trial"

[[case]]
name = "a trial cache with no marker next to it is never offered"
kind = "negative"
[[case.file]]
path = "home/bare/fresher-trial-cache/blob.bin"
size = 4096

[[case]]
name = "a trial cache next to a marker goes to the Trash"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "home/work/fresher-trial-cache/blob.bin"
size = 4096
fill = "data"
[case.expect]
item = "home/work/fresher-trial-cache"
category = "trial.caches"
danger = "safe"
after-cleanup = "in-trash"

[[case]]
name = "a trial kept folder next to a marker is emptied"
kind = "positive"
[[case.marker]]
path = "home/keep"
[[case.file]]
path = "home/keep/fresher-trial-kept/old.bin"
size = 4096
[case.expect]
item = "home/keep/fresher-trial-kept"
category = "trial.kept"
danger = "low"
after-cleanup = "emptied"
"#;

        let report = run("holds", text).unwrap();

        let subjects: Vec<&str> =
            report["findings"].as_array().unwrap().iter().map(|finding| finding["subject"].as_str().unwrap()).collect();
        assert_eq!(subjects, ["home/keep/fresher-trial-kept", "home/work/fresher-trial-cache"]);
        let paths: Vec<&str> = report["treeAfter"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["path"].as_str().unwrap())
            .collect();
        assert!(paths.contains(&"home/bare/fresher-trial-cache/blob.bin"), "{paths:?}");
        assert!(paths.contains(&"home/keep/fresher-trial-kept"), "{paths:?}");
        assert!(!paths.contains(&"home/work/fresher-trial-cache"), "{paths:?}");
    }
}
