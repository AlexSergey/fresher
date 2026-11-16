//! Check 1 of the guard: how far the cleaner reaches. A development build or a test acts only below a sandbox
//! folder; the real disk needs a token that only a release build can make.

use std::ffi::OsString;
use std::io::Read as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use fresher_core::{Protected, Protection, SANDBOX_MARKER, SANDBOX_SIGNATURE};

use crate::paths::{LastLink, open_no_link};

/// How far the cleaner may act.
#[derive(Debug)]
pub enum Reach {
    /// Only below a folder that holds the sandbox marker: every development build and every test.
    Sandbox,
    /// Anywhere the other checks allow: the release app.
    RealDisk(RealDiskToken),
}

/// The licence to act on the real disk. A debug build cannot make one, and only the app's composition names it
/// (checked by `tests/reach.rs`).
#[derive(Debug)]
pub struct RealDiskToken(());

impl RealDiskToken {
    /// The token, in a release build only.
    #[cfg(not(debug_assertions))]
    #[expect(clippy::new_without_default, reason = "a `Default` would be one more way to make the token")]
    pub const fn new() -> Self {
        Self(())
    }
}

impl Reach {
    /// Check 1 for an item at `relative` below the canonical `root`: in the sandbox, the root or a folder between it
    /// and the item must hold a valid marker. `protection` is the system's own list: a marker counts only in a folder
    /// it does not protect, so never in the home folder, a hard-blocked folder or an ancestor of a protected place.
    pub(crate) fn admits(&self, root: &Path, relative: &[OsString], user: u32, protection: &Protection) -> bool {
        match self {
            Self::RealDisk(_) => true,
            Self::Sandbox => sandbox_folders(root, relative).any(|folder| holds_marker(&folder, user, protection)),
        }
    }

    /// Check 1 for a tool's item, which has no path: a tool acts on the real system, so never in the sandbox.
    pub(crate) const fn admits_tools(&self) -> bool {
        matches!(self, Self::RealDisk(_))
    }
}

/// The root, then each folder between it and the item; never the item itself.
fn sandbox_folders<'a>(root: &'a Path, relative: &'a [OsString]) -> impl Iterator<Item = PathBuf> + 'a {
    let parents = relative.len().saturating_sub(1);
    (0..=parents).map(move |depth| relative.iter().take(depth).fold(root.to_path_buf(), |path, name| path.join(name)))
}

/// Whether `folder` holds a marker: a regular file of the user, opened through no link, whose first line is the
/// signature, in a folder the system's protection does not refuse.
fn holds_marker(folder: &Path, user: u32, protection: &Protection) -> bool {
    if matches!(protection.check(folder), Err(Protected::Never | Protected::Ancestor | Protected::WholeFolder)) {
        return false;
    }
    let Ok(marker) = open_no_link(&folder.join(SANDBOX_MARKER), LastLink::Refuse) else {
        return false;
    };
    let Ok(metadata) = marker.metadata() else {
        return false;
    };
    if !metadata.is_file() || metadata.uid() != user {
        return false;
    }
    let signature = SANDBOX_SIGNATURE.as_bytes();
    let mut first = Vec::with_capacity(signature.len() + 1);
    if marker.take(signature.len() as u64 + 1).read_to_end(&mut first).is_err() {
        return false;
    }
    first.strip_suffix(b"\n").unwrap_or(&first) == signature
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::{Path, PathBuf};

    use fresher_core::Protection;
    use fresher_testing::FixtureTree;

    use super::Reach;

    fn tree() -> FixtureTree {
        FixtureTree::from_toml_str("[[file]]\npath = \"home/cache/a.bin\"\n\n[[dir]]\npath = \"home/kept\"\n").unwrap()
    }

    fn user(tree: &FixtureTree) -> u32 {
        fs::metadata(tree.root()).unwrap().uid()
    }

    fn canonical(tree: &FixtureTree, relative: &str) -> PathBuf {
        fs::canonicalize(tree.root().join(relative)).unwrap()
    }

    fn components(relative: &str) -> Vec<OsString> {
        Path::new(relative).iter().map(OsString::from).collect()
    }

    /// Check 1 for `item` below the root `root`, both relative to the tree.
    fn admits(tree: &FixtureTree, root: &str, item: &str, protection: &Protection) -> bool {
        Reach::Sandbox.admits(&canonical(tree, root), &components(item), user(tree), protection)
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::Path;

        use fresher_core::{Protection, SANDBOX_MARKER, SANDBOX_SIGNATURE};

        use super::{Reach, admits, canonical, components, tree, user};

        #[test]
        fn an_item_with_no_marker_above_it_is_not_in_the_sandbox() {
            let tree = tree();

            assert!(!admits(&tree, "home", "cache/a.bin", &Protection::default()));
        }

        #[test]
        fn a_marker_that_is_a_link_does_not_count() {
            let tree = tree();
            tree.mark_sandbox(Path::new("elsewhere")).unwrap();
            symlink(tree.root().join("elsewhere").join(SANDBOX_MARKER), tree.root().join("home").join(SANDBOX_MARKER))
                .unwrap();

            assert!(!admits(&tree, "home", "cache/a.bin", &Protection::default()));
        }

        #[test]
        fn a_marker_with_the_wrong_signature_does_not_count() {
            let tree = tree();
            let marker = tree.root().join("home").join(SANDBOX_MARKER);
            for content in
                ["", "fresher-sandbox 0\n", &format!("{SANDBOX_SIGNATURE}x\n"), &format!("x{SANDBOX_SIGNATURE}")]
            {
                fs::write(&marker, content).unwrap();

                assert!(!admits(&tree, "home", "cache/a.bin", &Protection::default()), "{content:?}");
            }
        }

        #[test]
        fn a_marker_that_is_a_folder_does_not_count() {
            let tree = tree();
            fs::create_dir(tree.root().join("home").join(SANDBOX_MARKER)).unwrap();

            assert!(!admits(&tree, "home", "cache/a.bin", &Protection::default()));
        }

        #[test]
        fn a_marker_of_another_user_does_not_count() {
            let tree = tree().with_sandbox().unwrap();
            let other = user(&tree).wrapping_add(1);

            let admitted =
                Reach::Sandbox.admits(&canonical(&tree, ""), &components("home/a"), other, &Protection::default());

            assert!(!admitted);
        }

        #[test]
        fn a_marker_above_the_root_does_not_count() {
            let tree = tree().with_sandbox().unwrap();

            assert!(!admits(&tree, "home", "cache/a.bin", &Protection::default()));
        }

        #[test]
        fn a_marker_in_the_item_itself_does_not_count() {
            let tree = tree();
            tree.mark_sandbox(Path::new("home/cache")).unwrap();

            assert!(!admits(&tree, "home", "cache", &Protection::default()));
        }

        #[test]
        fn a_marker_in_a_whole_folder_does_not_count() {
            let tree = tree();
            tree.mark_sandbox(Path::new("home")).unwrap();
            let home = canonical(&tree, "home");
            let whole = Protection::new(vec![], vec![home], vec![]).unwrap();

            assert!(!admits(&tree, "home", "cache/a.bin", &whole));
        }

        #[test]
        fn a_marker_in_an_ancestor_of_a_protected_place_does_not_count() {
            let tree = tree();
            tree.mark_sandbox(Path::new("home")).unwrap();
            let never = Protection::new(vec![canonical(&tree, "home/kept")], vec![], vec![]).unwrap();

            assert!(!admits(&tree, "home", "cache/a.bin", &never));
        }

        #[test]
        fn a_tool_is_never_in_the_sandbox() {
            assert!(!Reach::Sandbox.admits_tools());
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::Protection;

        use super::{admits, tree};

        #[test]
        fn an_item_under_the_fixture_root_is_in_the_sandbox() {
            let tree = tree().with_sandbox().unwrap();

            assert!(admits(&tree, "", "home/cache/a.bin", &Protection::default()));
            assert!(admits(&tree, "", "home", &Protection::default()));
        }

        #[test]
        fn a_marker_in_a_subfolder_opens_only_that_subfolder() {
            let tree = tree();
            tree.mark_sandbox(Path::new("home/cache")).unwrap();

            assert!(admits(&tree, "", "home/cache/a.bin", &Protection::default()));
            assert!(!admits(&tree, "", "home/kept", &Protection::default()));
            assert!(!admits(&tree, "", "home/cache", &Protection::default()));
        }

        #[test]
        fn a_marker_without_a_line_end_counts() {
            let tree = tree();
            let marker = tree.root().join("home").join(fresher_core::SANDBOX_MARKER);
            std::fs::write(marker, fresher_core::SANDBOX_SIGNATURE).unwrap();

            assert!(admits(&tree, "home", "cache/a.bin", &Protection::default()));
        }
    }
}
