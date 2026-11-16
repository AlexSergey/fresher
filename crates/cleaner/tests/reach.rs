//! The real disk is reached only from the app's composition: the token that allows it is named nowhere else in the
//! workspace's sources.

mod support;

#[cfg(test)]
mod sources {
    use std::path::Path;

    use crate::support::rust_files;

    /// The token's name, spelled in two halves so that this file does not name it.
    pub(crate) const TOKEN: &str = concat!("RealDisk", "Token");

    /// The files that may make the token (its definition and the app's composition), relative to the workspace.
    const MAKERS: [&str; 2] = ["crates/cleaner/src/reach.rs", "app/tauri/src/compose.rs"];
    /// The files that may name it without making it: the cleaner's re-export.
    const NAMERS: [&str; 1] = ["crates/cleaner/src/lib.rs"];

    /// The files that make the token, or name it, where they may not. Whitespace is ignored, so `Token :: new`
    /// counts too.
    pub(crate) fn offenders(root: &Path) -> Vec<String> {
        let constructor = format!("{TOKEN}::new");
        let mut found = Vec::new();
        for (path, text) in rust_files(root) {
            let compact: String = text.split_whitespace().collect();
            let makes = compact.contains(&constructor);
            let names = text.contains(TOKEN);
            if (makes && !MAKERS.contains(&path.as_str()))
                || (names && !MAKERS.contains(&path.as_str()) && !NAMERS.contains(&path.as_str()))
            {
                found.push(path);
            }
        }
        found.sort();
        found
    }
}

mod negative {
    use std::fs;

    use crate::sources::{TOKEN, offenders};

    #[test]
    fn the_token_appears_only_in_the_composition() {
        let copy = tempfile::tempdir().unwrap();
        for (folder, text) in [
            ("app/tauri/src", format!("let token = {TOKEN}::new();")),
            ("crates/engine/src", format!("let token = {TOKEN} :: new();")),
            ("modules/demo/src", format!("fn take(_: {TOKEN}) {{}}")),
            ("crates/cleaner/src", format!("let token = {TOKEN}::new();")),
        ] {
            fs::create_dir_all(copy.path().join(folder)).unwrap();
            let file = if folder == "app/tauri/src" { "compose.rs" } else { "planted.rs" };
            fs::write(copy.path().join(folder).join(file), text).unwrap();
        }

        assert_eq!(
            offenders(copy.path()),
            ["crates/cleaner/src/planted.rs", "crates/engine/src/planted.rs", "modules/demo/src/planted.rs"]
        );
    }
}

mod positive {
    use crate::sources::offenders;
    use crate::support::{rust_files, workspace};

    #[test]
    fn the_workspace_makes_the_token_only_in_the_composition() {
        assert!(rust_files(&workspace()).len() > 50, "the workspace's sources were not found");
        assert_eq!(offenders(&workspace()), Vec::<String>::new());
    }
}
