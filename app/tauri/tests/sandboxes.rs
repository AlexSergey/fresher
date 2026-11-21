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
#[cfg(test)]
#[path = "sandboxes/left_out.rs"]
mod left_out;
#[cfg(test)]
#[path = "sandboxes/map.rs"]
mod map;

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

    /// A `demo` case file (the demo module leaves one made-up cache out on every scan) with `left_out` on its case.
    fn demo_file(left_out: &str) -> String {
        format!(
            "feature = \"demo\"\nmodule = \"demo\"\n[[case]]\nname = \"a plain file\"\nkind = \"negative\"\n\
             [[case.file]]\npath = \"home/plain/note.txt\"\nsize = 4096\n{left_out}"
        )
    }

    fn demo_left_out(reason: &str, items: u32) -> String {
        format!("[[case.left-out]]\ncategory = \"demo.caches\"\nreason = \"{reason}\"\nitems = {items}\n")
    }

    #[test]
    fn a_left_out_no_case_expects_fails_naming_it() {
        let failure = run("unexpected", &demo_file("")).unwrap_err();

        assert_eq!(failure.case, "unexpected", "{failure}");
        assert!(failure.problem.contains("`demo.caches` left out for `demo.left-out.open`: 1 items"), "{failure}");
    }

    #[test]
    fn an_expected_left_out_the_module_does_not_report_fails() {
        let text =
            demo_file(&demo_left_out("demo.left-out.open", 1)).replace("module = \"demo\"", "module = \"trial\"");
        let text = text.replace("demo.caches", "trial.caches").replace("demo.left-out.open", "trial.error.failed");

        let failure = run("missing", &text).unwrap_err();

        assert!(
            failure.problem.contains("`trial.caches` left out for `trial.error.failed`: expected 1 items, none"),
            "{failure}"
        );
    }

    #[test]
    fn a_left_out_count_that_differs_fails_with_both_numbers() {
        let failure = run("differs", &demo_file(&demo_left_out("demo.left-out.open", 2))).unwrap_err();

        assert!(failure.problem.contains("expected 2 items, got 1"), "{failure}");
    }

    #[test]
    fn a_left_out_reason_the_module_does_not_declare_is_refused() {
        let failure = run("undeclared", &demo_file(&demo_left_out("demo.left-out.closed", 1))).unwrap_err();

        assert!(failure.problem.contains("does not fit the modules"), "{failure}");
        assert!(failure.problem.contains("demo.left-out.closed"), "{failure}");
    }

    #[test]
    fn a_left_out_of_a_category_the_modules_lack_is_refused() {
        let text = demo_file(&demo_left_out("demo.left-out.open", 1)).replace("demo.caches", "demo.nothing");

        let failure = run("unknown", &text).unwrap_err();

        assert!(failure.problem.contains("demo.nothing"), "{failure}");
    }

    /// The negative's folder is inside the positive's cache: never offered itself, but explained by the cache's claim.
    #[test]
    fn a_your_folders_expectation_that_is_explained_fails_with_its_case_name() {
        let negative = "[[case]]\nname = \"a folder inside the cache\"\nkind = \"negative\"\n[[case.file]]\n\
                        path = \"home/work/fresher-trial-cache/inner/keep.bin\"\nsize = 4096\n\
                        [case.map]\npart = \"your-folders\"\n";
        let text = format!(
            "feature = \"trial\"\nmodule = \"trial\"\n{negative}{}",
            trial_positive("the cache", CACHE_IN_TRASH)
        );

        let failure = run("explained", &text).unwrap_err();

        assert_eq!(failure.case, "a folder inside the cache", "{failure}");
        assert!(failure.problem.contains("is explained: home/work/fresher-trial-cache is offered"), "{failure}");
    }

    /// The sandbox does not scan `home/Documents`, a place that prompts: the map has no node there.
    #[test]
    fn a_map_expectation_where_the_scan_does_not_go_fails() {
        let text = "feature = \"trial\"\nmodule = \"trial\"\n[[case]]\nname = \"a private note\"\nkind = \"negative\"\n\
                    [[case.file]]\npath = \"home/Documents/private/note.txt\"\nsize = 4096\n\
                    [case.map]\npart = \"your-folders\"\n";

        let failure = run("unscanned", text).unwrap_err();

        assert_eq!(failure.case, "a private note", "{failure}");
        assert!(failure.problem.contains("the scan has no node at"), "{failure}");
        assert!(failure.problem.contains("home/Documents/private"), "{failure}");
    }

    /// The case's own unclaimed file sits beside its cache, under the `at` it names as explained.
    #[test]
    fn an_explained_expectation_not_explained_in_full_fails() {
        let expect = format!("{CACHE_IN_TRASH}[case.map]\npart = \"explained\"\nat = \"home/work\"\n");
        let case = trial_positive("a cache and a note", &expect).replace(
            "[case.expect]",
            "[[case.file]]\npath = \"home/work/note.bin\"\nsize = 1048576\nfill = \"data\"\n[case.expect]",
        );
        let text = format!("feature = \"trial\"\nmodule = \"trial\"\n{case}");

        let failure = run("partly", &text).unwrap_err();

        assert_eq!(failure.case, "a cache and a note", "{failure}");
        assert!(failure.problem.contains("expected Explained"), "{failure}");
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
    use serde_json::json;

    use crate::harness::{case_file_names, read_case_file, registered_modules, run};

    case_files! {
        app_leftovers => "app-leftovers",
        app_leftovers_spotlight_off => "app-leftovers-spotlight-off",
        cleanup_guards => "cleanup-guards",
        dev_caches => "dev-caches",
        dev_caches_unsigned => "dev-caches-unsigned",
        dev_caches_xcode_open => "dev-caches-xcode-open",
        downloads => "downloads",
        downloads_browser_open => "downloads-browser-open",
        downloads_denied => "downloads-denied",
        jetbrains => "jetbrains",
        jetbrains_unknown_app_installed => "jetbrains-unknown-app-installed",
        jetbrains_unknown_app_open => "jetbrains-unknown-app-open",
        local_ai_models => "local-ai-models",
        local_ai_models_lm_studio_open => "local-ai-models-lm-studio-open",
        stale_projects => "stale-projects",
        storage_map => "storage-map",
        temp => "temp",
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

    /// A module's left-out the case file expects passes, and the report names it; the demo module leaves one out.
    #[test]
    fn an_expected_left_out_passes_and_is_reported() {
        let text = "feature = \"demo\"\nmodule = \"demo\"\n[[case]]\nname = \"a plain file\"\nkind = \"negative\"\n\
                    [[case.file]]\npath = \"home/plain/note.txt\"\nsize = 4096\n[[case.left-out]]\n\
                    category = \"demo.caches\"\nreason = \"demo.left-out.open\"\nitems = 1\n";

        let report = run("left-out", text).unwrap();

        let left_out = report["leftOut"].as_array().unwrap();
        assert_eq!(left_out.len(), 1);
        assert_eq!(left_out[0]["category"], "demo.caches");
        assert_eq!(left_out[0]["items"], 1);
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
fill = "data"
[case.map]
part = "your-folders"
at = "home/bare"

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
[case.map]
part = "explained"

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

        assert!(report.get("leftOut").is_none());
        let places = report["map"]["places"].as_array().unwrap();
        let (bare, cache) = (&places[0], &places[1]);
        // `home/bare` is all one folder, which the view opens through as the overview opens a folder row; the small
        // file is folded into its folder, so the place is checked by its own row.
        assert_eq!(bare["path"], json!(["home/bare", "home/bare/fresher-trial-cache"]));
        assert_eq!(bare["rows"], json!([{ "path": "home/bare", "logical": 4096, "allocated": 4096, "explained": 0 }]));
        // The scan does not go into the claimed cache: it is shown by its own row, explained in full.
        assert_eq!(cache["path"], json!(["home/work/fresher-trial-cache"]));
        assert_eq!(cache["rows"][0]["explained"], cache["rows"][0]["allocated"]);
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
