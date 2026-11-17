//! Check 1 of the guard: how far the cleaner reaches. A development build or a test acts only below a sandbox
//! folder; the real disk needs a token that only a release build can make.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fresher_core::{Protected, Protection};

/// How far the cleaner may act.
#[derive(Debug)]
pub enum Reach {
    /// Only below a folder that holds the sandbox marker: every development build and every test.
    Sandbox,
    /// Only strictly inside one sandbox folder, checked when it was made: a development build in sandbox mode.
    SandboxFolder(SandboxFolder),
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

/// The one folder a cleaner in sandbox mode may act in: canonical, reached through no link, holding a valid marker
/// of its user, and not refused by the real Mac's protection list.
#[derive(Debug)]
pub struct SandboxFolder {
    path: PathBuf,
    user: u32,
}

/// Why a folder cannot be a sandbox.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("{path} cannot be read")]
    Io { path: PathBuf, source: io::Error },
    #[error("{path} is a link")]
    ThroughLink { path: PathBuf },
    #[error("{path} is not a folder")]
    NotAFolder { path: PathBuf },
    #[error("{folder} holds no valid sandbox marker")]
    NoMarker { folder: PathBuf },
    #[error("{folder} is protected on this Mac: a protected place, an ancestor of one or a whole folder")]
    Protected { folder: PathBuf },
}

impl SandboxFolder {
    /// The sandbox at `path` for `user`. `real` is the real Mac's protection list: the folder must be neither a
    /// protected place, nor an ancestor of one, nor a whole folder by it.
    pub fn new(path: &Path, user: u32, real: &Protection) -> Result<Self, SandboxError> {
        // Rebuilt from its components: a trailing `/` would make the `lstat` below follow a link. The generator's
        // `canonical_folder` (`crates/sandbox/src/layout.rs`) checks a sandbox folder the same way: change both.
        let path = &path.components().collect::<PathBuf>();
        let entry = fs::symlink_metadata(path).map_err(|source| SandboxError::Io { path: path.into(), source })?;
        if entry.is_symlink() {
            return Err(SandboxError::ThroughLink { path: path.into() });
        }
        if !entry.is_dir() {
            return Err(SandboxError::NotAFolder { path: path.into() });
        }
        let folder = fs::canonicalize(path).map_err(|source| SandboxError::Io { path: path.into(), source })?;
        if refuses(real, &folder) {
            return Err(SandboxError::Protected { folder });
        }
        if !marker::holds_marker(&folder, user) {
            return Err(SandboxError::NoMarker { folder });
        }
        Ok(Self { path: folder, user })
    }

    /// The sandbox folder, canonical.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether the item at `relative` below the canonical `root` is strictly inside the folder by components, while
    /// the folder still holds its marker.
    fn admits(&self, root: &Path, relative: &[OsString]) -> bool {
        let item = relative.iter().fold(root.to_path_buf(), |path, name| path.join(name));
        let inside = item.strip_prefix(&self.path).is_ok_and(|rest| !rest.as_os_str().is_empty());
        inside && marker::holds_marker(&self.path, self.user)
    }
}

impl Reach {
    /// Check 1 for an item at `relative` below the canonical `root`: in the sandbox, the root or a folder between it
    /// and the item must hold a valid marker. `protection` is the system's own list: a marker counts only in a folder
    /// it does not protect, so never in the home folder, a hard-blocked folder or an ancestor of a protected place.
    /// A sandbox folder admits only what is strictly inside it, while its marker is still valid.
    pub(crate) fn admits(&self, root: &Path, relative: &[OsString], user: u32, protection: &Protection) -> bool {
        match self {
            Self::RealDisk(_) => true,
            Self::Sandbox => sandbox_folders(root, relative).any(|folder| holds_marker(&folder, user, protection)),
            Self::SandboxFolder(folder) => folder.admits(root, relative),
        }
    }

    /// Check 1 for a tool's item, which has no path: a tool acts on the real system, so never in the sandbox.
    pub(crate) const fn admits_tools(&self) -> bool {
        matches!(self, Self::RealDisk(_))
    }

    /// Check 1 for a path the roots could not place: only the real disk lets it reach the later checks.
    pub(crate) const fn admits_unplaced(&self) -> bool {
        matches!(self, Self::RealDisk(_))
    }
}

/// The root, then each folder between it and the item; never the item itself.
fn sandbox_folders<'a>(root: &'a Path, relative: &'a [OsString]) -> impl Iterator<Item = PathBuf> + 'a {
    let parents = relative.len().saturating_sub(1);
    (0..=parents).map(move |depth| relative.iter().take(depth).fold(root.to_path_buf(), |path, name| path.join(name)))
}

/// Whether `protection` refuses `folder` as a place for a marker: a protected place, an ancestor of one or a whole
/// folder.
fn refuses(protection: &Protection, folder: &Path) -> bool {
    matches!(protection.check(folder), Err(Protected::Never | Protected::Ancestor | Protected::WholeFolder))
}

/// Whether `folder` holds a valid marker of `user` in a folder the system's protection does not refuse.
fn holds_marker(folder: &Path, user: u32, protection: &Protection) -> bool {
    !refuses(protection, folder) && marker::holds_marker(folder, user)
}

/// The marker reader `fresher-macos` shares with the sandbox generator; off macOS no marker counts (fail closed).
mod marker {
    #[cfg(target_os = "macos")]
    pub(super) use fresher_macos::sandbox_marker::holds_marker;

    #[cfg(not(target_os = "macos"))]
    pub(super) const fn holds_marker(_folder: &std::path::Path, _user: u32) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::{Path, PathBuf};

    use fresher_core::Protection;
    use fresher_testing::FixtureTree;

    use super::{Reach, SandboxFolder};

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

    /// A tree with the sandbox folder `box/` (marked), its home, a sibling `box2/` and a folder `other/`.
    fn boxed() -> FixtureTree {
        let tree = FixtureTree::from_toml_str(
            "[[file]]\npath = \"box/home/cache/a.bin\"\n\n[[file]]\npath = \"box2/b.bin\"\n\n[[dir]]\npath = \"other\"\n",
        )
        .unwrap();
        tree.mark_sandbox(Path::new("box")).unwrap();
        tree
    }

    /// The reach of the sandbox folder `box/` of the tree, checked against no real protection.
    fn folder_reach(tree: &FixtureTree) -> Reach {
        Reach::SandboxFolder(SandboxFolder::new(&tree.root().join("box"), user(tree), &Protection::default()).unwrap())
    }

    /// Check 1 under `reach` for `item` below the root `root`, both relative to the tree.
    fn admits_in(reach: &Reach, tree: &FixtureTree, root: &str, item: &str) -> bool {
        reach.admits(&canonical(tree, root), &components(item), user(tree), &Protection::default())
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::Path;

        use fresher_core::{Protection, SANDBOX_MARKER, SANDBOX_SIGNATURE};

        use super::{Reach, admits, admits_in, boxed, canonical, components, folder_reach, tree, user};
        use crate::reach::{SandboxError, SandboxFolder};

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

        #[test]
        fn a_folder_without_a_marker_is_not_a_sandbox() {
            let tree = boxed();

            let error = SandboxFolder::new(&tree.root().join("other"), user(&tree), &Protection::default());

            assert!(matches!(error, Err(SandboxError::NoMarker { .. })));
        }

        #[test]
        fn a_folder_through_a_link_is_not_a_sandbox() {
            let tree = boxed();
            symlink(tree.root().join("box"), tree.root().join("alias")).unwrap();

            let error = SandboxFolder::new(&tree.root().join("alias"), user(&tree), &Protection::default());

            assert!(matches!(error, Err(SandboxError::ThroughLink { .. })));
        }

        #[test]
        fn a_folder_through_a_link_with_a_trailing_slash_is_not_a_sandbox() {
            let tree = boxed();
            symlink(tree.root().join("box"), tree.root().join("alias")).unwrap();

            let error = SandboxFolder::new(&tree.root().join("alias").join(""), user(&tree), &Protection::default());

            assert!(matches!(error, Err(SandboxError::ThroughLink { .. })));
        }

        #[test]
        fn the_real_home_is_not_a_sandbox() {
            let tree = boxed();
            let home = canonical(&tree, "box");
            let real = Protection::new(vec![home.join("Library/Mobile Documents")], vec![home], vec![]).unwrap();

            let error = SandboxFolder::new(&tree.root().join("box"), user(&tree), &real);

            assert!(matches!(error, Err(SandboxError::Protected { .. })));
        }

        #[test]
        fn a_hard_blocked_folder_is_not_a_sandbox() {
            let tree = boxed();
            tree.mark_sandbox(Path::new("box/home")).unwrap();
            let real = Protection::new(vec![], vec![canonical(&tree, "box/home")], vec![]).unwrap();

            let error = SandboxFolder::new(&tree.root().join("box/home"), user(&tree), &real);

            assert!(matches!(error, Err(SandboxError::Protected { .. })));
        }

        #[test]
        fn an_item_outside_the_folder_is_not_admitted() {
            let tree = boxed();
            let reach = folder_reach(&tree);

            assert!(!admits_in(&reach, &tree, "", "box2/b.bin"));
            assert!(!admits_in(&reach, &tree, "box2", "b.bin"));
            assert!(!admits_in(&reach, &tree, "", "other"));
        }

        #[test]
        fn the_folder_itself_is_not_admitted() {
            let tree = boxed();
            let reach = folder_reach(&tree);

            assert!(!admits_in(&reach, &tree, "", "box"));
            assert!(!admits_in(&reach, &tree, "box", ""));
            assert!(!admits_in(&reach, &tree, "", ""));
        }

        #[test]
        fn an_item_is_refused_once_the_marker_is_gone() {
            let tree = boxed();
            let reach = folder_reach(&tree);
            let marker = tree.root().join("box").join(SANDBOX_MARKER);
            fs::rename(&marker, tree.root().join("other").join(SANDBOX_MARKER)).unwrap();

            assert!(!admits_in(&reach, &tree, "box", "home/cache/a.bin"));
        }

        #[test]
        fn a_tool_is_never_admitted() {
            let tree = boxed();

            assert!(!folder_reach(&tree).admits_tools());
            assert!(!folder_reach(&tree).admits_unplaced());
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::{Action, Depth, Protection, SkipReason};

        use super::{admits, admits_in, boxed, folder_reach, tree};
        use crate::guard::{Purpose, Unverified, verify};
        use crate::paths::Roots;
        use crate::reach::{Reach, SandboxFolder};
        use crate::test_support::{Setup, plan_item};

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

        #[test]
        fn an_item_inside_the_folder_is_admitted() {
            let tree = boxed();
            let reach = folder_reach(&tree);

            assert!(admits_in(&reach, &tree, "", "box/home/cache/a.bin"));
            assert!(admits_in(&reach, &tree, "box/home", "cache"));
            assert!(admits_in(&reach, &tree, "box", "home"));
        }

        /// Through the guard, as sandbox mode composes it: the scan root is the sandbox's home, which the joined list
        /// makes a whole folder, and the marker sits above it.
        #[test]
        fn a_marker_above_a_whole_home_still_opens_the_folder() {
            let mut setup = Setup::new_without_sandbox("[[file]]\npath = \"box/home/cache/a.bin\"\n");
            setup.tree.mark_sandbox(Path::new("box")).unwrap();
            let home = setup.physical("box/home");
            let sandbox = Protection::new(vec![home.join("Documents")], vec![home], vec![]).unwrap();
            setup.system = Protection::default().joined(&sandbox).unwrap();
            let root = vec![setup.at("box/home")];
            setup.roots = Roots::new(&root, &root);
            let item = plan_item(1, Action::Remove(setup.at("box/home/cache/a.bin")), 4096);
            let folder = SandboxFolder::new(&setup.at("box"), setup.user, &Protection::default()).unwrap();

            let under_marker = verify(&item, &setup.cx(Purpose::Rehearse), Depth::Quick);
            setup.reach = Reach::SandboxFolder(folder);
            let in_folder = verify(&item, &setup.cx(Purpose::Rehearse), Depth::Quick);

            assert!(matches!(under_marker, Err(Unverified::Skipped(SkipReason::NotInSandbox))));
            assert!(in_folder.is_ok());
        }
    }
}
