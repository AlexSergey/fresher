//! The real disk is reached only from the app's composition, and the dry run only from it and the cleaner's own
//! tests: the tokens that allow them are named nowhere else in the workspace's sources.

mod support;

#[cfg(test)]
mod sources {
    use std::path::Path;

    use crate::support::rust_files;

    /// A token, the files that may make it, and the files that may name it without making it, relative to the
    /// workspace.
    pub(crate) struct Rule {
        pub(crate) token: &'static str,
        makers: &'static [&'static str],
        namers: &'static [&'static str],
    }

    /// The real disk's token, spelled in two halves so that this file does not name it: made by its definition and
    /// the app's composition, named by the cleaner's re-export.
    pub(crate) const REAL_DISK: Rule = Rule {
        token: concat!("RealDisk", "Token"),
        makers: &["crates/cleaner/src/reach.rs", "app/tauri/src/compose.rs"],
        namers: &["crates/cleaner/src/lib.rs"],
    };

    /// The dry run's token: made by its definition, the app's composition and the cleaner's tests, named by the
    /// cleaner's re-export.
    pub(crate) const DRY_RUN: Rule = Rule {
        token: concat!("DryRun", "Token"),
        makers: &[
            "crates/cleaner/src/reach.rs",
            "app/tauri/src/compose.rs",
            "crates/cleaner/src/test_support.rs",
            "crates/cleaner/tests/never.rs",
        ],
        namers: &["crates/cleaner/src/lib.rs"],
    };

    /// The files that make the rule's token, or name it, where they may not. Whitespace is ignored, so
    /// `Token :: new` counts too.
    pub(crate) fn offenders(root: &Path, rule: &Rule) -> Vec<String> {
        let constructor = format!("{}::new", rule.token);
        let mut found = Vec::new();
        for (path, text) in rust_files(root) {
            let compact: String = text.split_whitespace().collect();
            let makes = compact.contains(&constructor);
            let names = text.contains(rule.token);
            let may_make = rule.makers.contains(&path.as_str());
            if (makes && !may_make) || (names && !may_make && !rule.namers.contains(&path.as_str())) {
                found.push(path);
            }
        }
        found.sort();
        found
    }

    /// Writes each `(file, text)` below a new temporary folder, as a copy of the workspace would hold it.
    pub(crate) fn planted(files: &[(&str, String)]) -> tempfile::TempDir {
        let copy = tempfile::tempdir().unwrap();
        for (file, text) in files {
            let path = copy.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        copy
    }
}

mod negative {
    use crate::sources::{DRY_RUN, REAL_DISK, offenders, planted};

    #[test]
    fn the_token_appears_only_in_the_composition() {
        let token = REAL_DISK.token;
        let copy = planted(&[
            ("app/tauri/src/compose.rs", format!("let token = {token}::new();")),
            ("crates/engine/src/planted.rs", format!("let token = {token} :: new();")),
            ("modules/demo/src/planted.rs", format!("fn take(_: {token}) {{}}")),
            ("crates/cleaner/src/planted.rs", format!("let token = {token}::new();")),
        ]);

        assert_eq!(
            offenders(copy.path(), &REAL_DISK),
            ["crates/cleaner/src/planted.rs", "crates/engine/src/planted.rs", "modules/demo/src/planted.rs"]
        );
    }

    #[test]
    fn the_dry_run_token_appears_only_in_the_composition_and_the_cleaner_s_tests() {
        let token = DRY_RUN.token;
        let copy = planted(&[
            ("app/tauri/src/compose.rs", format!("let token = {token}::new();")),
            ("crates/cleaner/src/test_support.rs", format!("let token = crate::{token}::new();")),
            ("crates/engine/src/planted.rs", format!("let token = {token} :: new();")),
            ("modules/demo/src/planted.rs", format!("fn take(_: {token}) {{}}")),
            ("crates/cleaner/src/planted.rs", format!("let token = {token}::new();")),
            ("app/tauri/src/dev_runs/planted.rs", format!("let token = {token}::new();")),
        ]);

        assert_eq!(
            offenders(copy.path(), &DRY_RUN),
            [
                "app/tauri/src/dev_runs/planted.rs",
                "crates/cleaner/src/planted.rs",
                "crates/engine/src/planted.rs",
                "modules/demo/src/planted.rs",
            ]
        );
    }
}

mod positive {
    use crate::sources::{DRY_RUN, REAL_DISK, offenders};
    use crate::support::{rust_files, workspace};

    #[test]
    fn the_workspace_makes_the_token_only_in_the_composition() {
        assert!(rust_files(&workspace()).len() > 50, "the workspace's sources were not found");
        assert_eq!(offenders(&workspace(), &REAL_DISK), Vec::<String>::new());
    }

    #[test]
    fn the_workspace_makes_the_dry_run_token_only_in_the_composition_and_the_cleaner_s_tests() {
        assert!(rust_files(&workspace()).len() > 50, "the workspace's sources were not found");
        assert_eq!(offenders(&workspace(), &DRY_RUN), Vec::<String>::new());
    }
}
