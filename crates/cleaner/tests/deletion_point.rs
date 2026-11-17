//! The single deletion point: the lint that forbids the deleting functions (`std::fs`'s and `fresher-macos`'s
//! `remove_at`) is expected exactly once in the workspace, at `crates/cleaner/src/remove/permanent.rs`, and allowed
//! nowhere. The one exception is the home lookup: the same lint forbids looking up the home folder, and the two files
//! that look it up (`HOME_LOOKUPS`) may each expect it once, for that lookup only. The system's deleting calls
//! themselves are named only where `remove_at` makes its one call, in `crates/macos/src/ffi/beneath.rs` (no other crate
//! may hold unsafe code to call them).

mod support;

#[cfg(test)]
mod sources {
    use std::path::Path;

    use crate::support::rust_files;

    /// The lint's name, spelled in two halves so that this file does not name it.
    pub(crate) const LINT: &str = concat!("disallowed", "_methods");
    /// The lint groups that hold it: silencing one silences the lint too.
    const GROUPS: [&str; 2] = [concat!("clippy", "::style"), concat!("clippy", "::all")];
    /// The deletion point, relative to the workspace.
    pub(crate) const POINT: &str = "crates/cleaner/src/remove/permanent.rs";
    /// The files that look up the home folder, relative to the workspace: each may expect the lint once, for a
    /// `let home = ...home_dir()...;` statement right below it, with a reason that starts with `HOME_REASON`.
    pub(crate) const HOME_LOOKUPS: [(&str, &str); 2] = [
        ("app/tauri/src/lib.rs", "the app's setup: everything after it gets the home injected"),
        ("crates/sandbox/src/main.rs", "the sandbox generator: only for the real Mac's protection list"),
    ];
    /// How the reason of a home lookup's expectation starts.
    pub(crate) const HOME_REASON: &str = "the home folder is looked up once";
    /// Where `remove_at` calls the system, relative to the workspace.
    pub(crate) const CALL: &str = "crates/macos/src/ffi/beneath.rs";
    /// The system's deleting calls, spelled in two halves so that this file does not name them.
    pub(crate) const SYSTEM: [&str; 3] =
        [concat!("libc::", "unlink"), concat!("libc::", "rmdir"), concat!("libc::", "remove")];

    /// The files that name the lint, or silence a group that holds it, where they may not: anywhere but the deletion
    /// point, which may name it once, and the home lookups, which may name it once for the lookup. Whitespace is
    /// ignored.
    pub(crate) fn offenders(root: &Path) -> Vec<String> {
        let mut found = Vec::new();
        for (path, text) in rust_files(root) {
            let compact: String = text.split_whitespace().collect();
            let named = compact.matches(LINT).count();
            let grouped = GROUPS.iter().any(|group| compact.contains(group));
            let home_lookup = HOME_LOOKUPS.iter().any(|&(file, _)| path == file) && is_home_lookup(&compact);
            let allowed = usize::from(path == POINT || home_lookup);
            if named > allowed || grouped {
                found.push(path);
            }
        }
        found.sort();
        found
    }

    /// Whether every naming of the lint in `compact` (whitespace removed) is the expectation of a home lookup:
    /// `#[expect(clippy::<lint>, reason = "<HOME_REASON>...")]` directly followed by a `let home = ` statement that
    /// calls `home_dir()`.
    fn is_home_lookup(compact: &str) -> bool {
        let opening = concat!("#[expect(clippy", "::");
        let reason: String = format!("{LINT},reason=\"{}", HOME_REASON.split_whitespace().collect::<String>());
        compact.match_indices(LINT).all(|(at, _)| {
            let (Some(before), Some(after)) = (compact.get(..at), compact.get(at..)) else { return false };
            let Some(rest) = after.strip_prefix(reason.as_str()) else { return false };
            let Some((_, below)) = rest.split_once("\")]") else { return false };
            let statement = below.split(';').next().unwrap_or_default();
            before.ends_with(opening) && is_lookup_statement(statement)
        })
    }

    /// Whether the compact `statement` is exactly `let home = <path>home_dir()`, then optionally
    /// `.ok_or("<text>")`, then optionally `?`: `<path>` is names joined by `::` or `.`, each call without arguments.
    fn is_lookup_statement(statement: &str) -> bool {
        let Some((path, tail)) = statement.strip_prefix("lethome=").and_then(|rest| rest.split_once("home_dir()"))
        else {
            return false;
        };
        let names = path.replace("()", "");
        let plain_path = (names.is_empty() || names.ends_with("::") || names.ends_with('.'))
            && names.chars().all(|char| char.is_ascii_alphanumeric() || matches!(char, '_' | ':' | '.'));
        let tail = tail.strip_suffix('?').unwrap_or(tail);
        let plain_tail = tail.is_empty()
            || tail
                .strip_prefix(".ok_or(\"")
                .and_then(|text| text.strip_suffix("\")"))
                .is_some_and(|text| !text.contains('"'));
        plain_path && plain_tail
    }

    /// The files that name a deleting call of the system, and how often, where they may not: anywhere but `CALL`,
    /// which may name `unlinkat` once. Whitespace is ignored.
    pub(crate) fn system_calls(root: &Path) -> Vec<(String, usize)> {
        let mut found: Vec<(String, usize)> = rust_files(root)
            .into_iter()
            .filter_map(|(path, text)| {
                let compact: String = text.split_whitespace().collect();
                let named: usize = SYSTEM.iter().map(|call| compact.matches(call).count()).sum();
                let allowed = usize::from(path == CALL);
                (named > allowed).then_some((path, named))
            })
            .collect();
        found.sort();
        found
    }
}

/// Writes `files` (relative path, text) under `root`.
#[cfg(test)]
fn plant(root: &std::path::Path, files: Vec<(String, String)>) {
    for (file, text) in files {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
}

/// A home lookup's expectation with `reason`, above `statement`.
#[cfg(test)]
fn home_lookup(reason: &str, statement: &str) -> String {
    format!("#[expect(clippy::{}, reason = \"{reason}\")]\n{statement}\n", sources::LINT)
}

mod negative {
    use std::fs;

    use crate::sources::{CALL, HOME_LOOKUPS, HOME_REASON, LINT, POINT, SYSTEM, offenders, system_calls};
    use crate::{home_lookup, plant};

    #[test]
    fn a_home_lookup_expectation_holds_only_in_its_files_for_the_lookup() {
        let copy = tempfile::tempdir().unwrap();
        let lookup = home_lookup(&format!("{HOME_REASON}, here"), "let home = std::env::home_dir();");
        let (app, generator) = (HOME_LOOKUPS[0].0.to_owned(), HOME_LOOKUPS[1].0.to_owned());
        plant(
            copy.path(),
            vec![
                ("crates/engine/src/planted.rs".to_owned(), lookup.clone()),
                (app, home_lookup("another reason", "let home = std::env::home_dir();")),
                (generator, format!("{lookup}{lookup}")),
            ],
        );
        let not_a_lookup = [
            home_lookup(HOME_REASON, "let folder = std::env::home_dir();"),
            home_lookup(HOME_REASON, "let home = std::env::temp_dir(); home_dir();"),
            home_lookup(HOME_REASON, "let home = std::env::home_dir().map(|h| { std::fs::remove_dir_all(&h); h });"),
            home_lookup(HOME_REASON, "let home = remove_dir_all(x).ok().and(home_dir());"),
            home_lookup(HOME_REASON, "let home = std::env::home_dir().ok_or(remove(x))?;"),
            format!("#[allow(clippy::{LINT}, reason = \"{HOME_REASON}\")]\nlet home = std::env::home_dir();"),
        ];

        assert_eq!(
            offenders(copy.path()),
            ["app/tauri/src/lib.rs", "crates/engine/src/planted.rs", "crates/sandbox/src/main.rs"]
        );
        for text in not_a_lookup {
            fs::write(copy.path().join(HOME_LOOKUPS[1].0), &text).unwrap();
            assert!(offenders(copy.path()).contains(&HOME_LOOKUPS[1].0.to_owned()), "{text}");
        }
    }

    #[test]
    fn the_lint_is_expected_only_at_the_deletion_point() {
        let copy = tempfile::tempdir().unwrap();
        let expect = format!("#[expect(clippy :: {LINT}, reason = \"x\")]\nfn f() {{}}\n");
        for (file, text) in [
            (POINT.to_owned(), format!("{expect}{expect}")),
            ("crates/engine/src/planted.rs".to_owned(), expect.clone()),
            ("modules/demo/src/planted.rs".to_owned(), format!("#![allow(clippy::{LINT})]")),
            ("app/tauri/src/planted.rs".to_owned(), format!("#![allow({})]", concat!("clippy", "::all"))),
            ("crates/store/src/clean.rs".to_owned(), "fn f() {}".to_owned()),
        ] {
            let path = copy.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }

        assert_eq!(
            offenders(copy.path()),
            [
                "app/tauri/src/planted.rs",
                "crates/cleaner/src/remove/permanent.rs",
                "crates/engine/src/planted.rs",
                "modules/demo/src/planted.rs"
            ]
        );
    }

    #[test]
    fn the_system_is_asked_to_delete_only_where_remove_at_calls_it() {
        let copy = tempfile::tempdir().unwrap();
        let call = |name: &str| format!("{name}(folder, name, 0);\n");
        for (file, text) in [
            (CALL.to_owned(), format!("{}{}", call(SYSTEM[0]), call(SYSTEM[1]))),
            ("crates/macos/src/trash.rs".to_owned(), call(&format!("{} at", SYSTEM[0]))),
            ("crates/macos/src/ffi/rename.rs".to_owned(), call(&format!("{}file", SYSTEM[2]))),
            ("crates/macos/src/ffi/volume.rs".to_owned(), "fn f() {}".to_owned()),
        ] {
            let path = copy.path().join(file);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }

        assert_eq!(
            system_calls(copy.path()),
            [
                (CALL.to_owned(), 2),
                ("crates/macos/src/ffi/rename.rs".to_owned(), 1),
                ("crates/macos/src/trash.rs".to_owned(), 1)
            ]
        );
    }
}

mod positive {
    use std::fs;

    use crate::sources::{CALL, HOME_LOOKUPS, HOME_REASON, LINT, POINT, SYSTEM, offenders, system_calls};
    use crate::support::workspace;
    use crate::{home_lookup, plant};

    #[test]
    fn each_home_lookup_file_may_expect_the_lint_once_for_the_lookup() {
        let copy = tempfile::tempdir().unwrap();
        let files = HOME_LOOKUPS
            .iter()
            .map(|&(file, _)| (file.to_owned(), home_lookup(HOME_REASON, "let home = app.path().home_dir()?;")))
            .collect();
        plant(copy.path(), files);

        assert_eq!(offenders(copy.path()), Vec::<String>::new());
        for statement in ["let home = home_dir();", "let home = std::env::home_dir().ok_or(\"no home\")?;"] {
            fs::write(copy.path().join(HOME_LOOKUPS[1].0), home_lookup(HOME_REASON, statement)).unwrap();
            assert_eq!(offenders(copy.path()), Vec::<String>::new(), "{statement}");
        }
    }

    #[test]
    fn the_workspace_looks_up_the_home_once_in_each_home_lookup_file() {
        for (file, _) in HOME_LOOKUPS {
            let text = fs::read_to_string(workspace().join(file)).unwrap();
            assert_eq!(text.matches(LINT).count(), 1, "{file}");
        }
    }

    #[test]
    fn the_workspace_expects_the_lint_once_at_the_deletion_point() {
        let point = fs::read_to_string(workspace().join(POINT)).unwrap();

        assert_eq!(point.matches(LINT).count(), 1);
        assert_eq!(offenders(&workspace()), Vec::<String>::new());
    }

    #[test]
    fn the_workspace_asks_the_system_to_delete_once_where_remove_at_calls_it() {
        let call = fs::read_to_string(workspace().join(CALL)).unwrap();

        assert_eq!(SYSTEM.iter().map(|name| call.matches(name).count()).sum::<usize>(), 1);
        assert_eq!(system_calls(&workspace()), Vec::<(String, usize)>::new());
    }
}
