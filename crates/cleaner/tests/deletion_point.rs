//! The single deletion point: the lint that forbids the deleting functions (`std::fs`'s and `fresher-macos`'s
//! `remove_at`) is expected exactly once in the workspace, at `crates/cleaner/src/remove/permanent.rs`, and allowed
//! nowhere; the system's deleting calls themselves are named only where `remove_at` makes its one call, in
//! `crates/macos/src/ffi/beneath.rs` (no other crate may hold unsafe code to call them).

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
    /// Where `remove_at` calls the system, relative to the workspace.
    pub(crate) const CALL: &str = "crates/macos/src/ffi/beneath.rs";
    /// The system's deleting calls, spelled in two halves so that this file does not name them.
    pub(crate) const SYSTEM: [&str; 3] =
        [concat!("libc::", "unlink"), concat!("libc::", "rmdir"), concat!("libc::", "remove")];

    /// The files that name the lint, or silence a group that holds it, where they may not: anywhere but the deletion
    /// point, which may name it once. Whitespace is ignored.
    pub(crate) fn offenders(root: &Path) -> Vec<String> {
        let mut found = Vec::new();
        for (path, text) in rust_files(root) {
            let compact: String = text.split_whitespace().collect();
            let named = compact.matches(LINT).count();
            let grouped = GROUPS.iter().any(|group| compact.contains(group));
            let allowed = usize::from(path == POINT);
            if named > allowed || grouped {
                found.push(path);
            }
        }
        found.sort();
        found
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

mod negative {
    use std::fs;

    use crate::sources::{CALL, LINT, POINT, SYSTEM, offenders, system_calls};

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

    use crate::sources::{CALL, LINT, POINT, SYSTEM, offenders, system_calls};
    use crate::support::workspace;

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
