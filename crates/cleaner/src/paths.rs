//! Checks 2 to 6 of the guard: where an item is, by its name and on the disk. The item is placed strictly inside one
//! root by components, checked against the protected and ignored places, then opened beneath the root's canonical
//! path through no link, on the root's volume.

use std::ffi::OsString;
use std::fs::{self, File, Metadata, OpenOptions};
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Component, Path, PathBuf};

use fresher_core::{Protected, Protection, SkipReason};

/// `ELOOP`: a component of the path is a link.
const ELOOP: i32 = 62;
/// `EMLINK`: what some systems answer instead of `ELOOP` for a refused link.
const EMLINK: i32 = 31;

/// What the open does when the last component of the path is a link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LastLink {
    /// Opens the link itself: an item that is a link is checked and removed as the link.
    AsItself,
    /// Refuses it, like a link anywhere else in the path.
    Refuse,
}

/// Opens `path` for reading without following a link in any component (`O_NOFOLLOW_ANY`, plan 008/01/02), and
/// without waiting on a named pipe.
#[cfg(target_os = "macos")]
pub(crate) fn open_no_link(path: &Path, last: LastLink) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt as _;

    /// `O_NOFOLLOW_ANY` in `sys/fcntl.h`: no component of the path may be a link (`ELOOP`).
    const O_NOFOLLOW_ANY: i32 = 0x2000_0000;
    /// `O_SYMLINK`: a link as the last component is opened as the link.
    const O_SYMLINK: i32 = 0x0020_0000;
    /// `O_NONBLOCK`: a named pipe opens at once instead of waiting for a writer (proved by the guard's test
    /// `a_pipe_with_no_writer_does_not_block_the_guard`).
    const O_NONBLOCK: i32 = 0x0000_0004;

    let last = match last {
        LastLink::AsItself => O_SYMLINK,
        LastLink::Refuse => 0,
    };
    OpenOptions::new().read(true).custom_flags(O_NOFOLLOW_ANY | O_NONBLOCK | last).open(path)
}

/// Off macOS nothing opens: the cleaner acts on nothing there (fail closed).
#[cfg(not(target_os = "macos"))]
pub(crate) fn open_no_link(_path: &Path, _last: LastLink) -> io::Result<File> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

/// What an open that failed means for the item.
pub(crate) fn open_failure(error: &io::Error) -> SkipReason {
    match error.raw_os_error() {
        Some(ELOOP | EMLINK) => SkipReason::ThroughLink,
        _ if error.kind() == io::ErrorKind::NotFound => SkipReason::Vanished,
        _ => SkipReason::CheckFailed,
    }
}

/// A scan root as the plan names it, its canonical path and its volume.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Root {
    lexical: PathBuf,
    canonical: PathBuf,
    device: u64,
}

/// The roots an item may lie in: the plan's roots that are still scan roots.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Roots {
    usable: Vec<Root>,
    /// Roots still in the settings that could not be made canonical: what is under them cannot be checked.
    unreadable: Vec<PathBuf>,
    /// Every root named by the plan or the settings, in both forms: none of them, and none of their ancestors, is
    /// ever an item.
    named: Vec<PathBuf>,
}

impl Roots {
    /// The roots of `plan` that `current` (the settings now) still holds, each made canonical once: a root may sit
    /// under a link of the system, such as `/var`.
    pub(crate) fn new(plan: &[PathBuf], current: &[PathBuf]) -> Self {
        let mut roots = Self::default();
        for root in plan.iter().filter(|root| current.contains(root)) {
            match canonical_folder(root) {
                Some((canonical, device)) => {
                    roots.named.push(canonical.clone());
                    roots.usable.push(Root { lexical: root.clone(), canonical, device });
                }
                None => roots.unreadable.push(root.clone()),
            }
        }
        roots.named.extend(plan.iter().chain(current).cloned());
        roots
    }

    /// The roots in both forms, for the snapshot of open files, which may name either.
    pub(crate) fn within(&self) -> Vec<PathBuf> {
        let mut within: Vec<PathBuf> =
            self.usable.iter().flat_map(|root| [root.lexical.clone(), root.canonical.clone()]).collect();
        within.dedup();
        within
    }

    /// Check 2: `path` must be absolute and plain (no `.` or `..`), strictly inside one root by components, and
    /// neither a root nor an ancestor of one.
    pub(crate) fn place(&self, path: &Path) -> Result<Placed, SkipReason> {
        if !is_plain(path) || self.named.iter().any(|root| root.starts_with(path)) {
            return Err(SkipReason::OutsideRoots);
        }
        let inside = |root: &Path| path.strip_prefix(root).ok().filter(|relative| !relative.as_os_str().is_empty());
        let deepest = self
            .usable
            .iter()
            .filter_map(|root| inside(&root.lexical).map(|relative| (root, relative)))
            .max_by_key(|(root, _)| root.lexical.components().count());
        let Some((root, relative)) = deepest else {
            let under_unreadable = self.unreadable.iter().any(|root| inside(root).is_some());
            return Err(if under_unreadable { SkipReason::CheckFailed } else { SkipReason::OutsideRoots });
        };
        let relative: Vec<OsString> = relative.iter().map(OsString::from).collect();
        let physical = relative.iter().fold(root.canonical.clone(), |physical, name| physical.join(name));
        if self.named.iter().any(|named| named.starts_with(&physical)) {
            return Err(SkipReason::OutsideRoots);
        }
        Ok(Placed {
            lexical: path.to_path_buf(),
            physical,
            root: root.canonical.clone(),
            device: root.device,
            relative,
        })
    }
}

/// The roots as the settings write them, with `~` and `~/…` made absolute under `home`. Anything else is kept as
/// it is; a root that is still not absolute and plain is never used (`Roots::new` cannot make it canonical).
pub(crate) fn expand_home(roots: &[PathBuf], home: &Path) -> Vec<PathBuf> {
    let expand = |root: &PathBuf| match root.strip_prefix("~") {
        Ok(rest) if rest.as_os_str().is_empty() => home.to_path_buf(),
        Ok(rest) => home.join(rest),
        Err(_) => root.clone(),
    };
    roots.iter().map(expand).collect()
}

/// The canonical path of a root and its volume; `None` when it is not absolute and plain (a relative root would be
/// read against the working folder), cannot be read, or is not a folder.
fn canonical_folder(root: &Path) -> Option<(PathBuf, u64)> {
    if !is_plain(root) {
        return None;
    }
    let canonical = fs::canonicalize(root).ok()?;
    let metadata = fs::symlink_metadata(&canonical).ok()?;
    metadata.is_dir().then(|| (canonical, metadata.dev()))
}

/// Absolute, with no `.` or `..` among its names. The raw names are checked too: `Path::components` drops a `.`.
fn is_plain(path: &Path) -> bool {
    let bytes = path.as_os_str().as_encoded_bytes();
    let dotted = bytes.split(|byte| *byte == b'/').any(|name| name == b"." || name == b"..");
    path.is_absolute()
        && !dotted
        && path.components().all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
}

/// An item placed in a root: its path as the plan names it, and where it is on the disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Placed {
    /// The path as the plan names it.
    pub(crate) lexical: PathBuf,
    /// The root's canonical path joined with the item's names: no component is a link of the system.
    pub(crate) physical: PathBuf,
    /// The root's canonical path.
    pub(crate) root: PathBuf,
    /// The root's volume.
    pub(crate) device: u64,
    /// The item's names below the root.
    pub(crate) relative: Vec<OsString>,
}

impl Placed {
    /// An entry of this folder, placed in the same root.
    pub(crate) fn child(&self, name: &OsString) -> Self {
        let mut relative = self.relative.clone();
        relative.push(name.clone());
        Self {
            lexical: self.lexical.join(name),
            physical: self.physical.join(name),
            root: self.root.clone(),
            device: self.device,
            relative,
        }
    }
}

/// Checks 3 and 4: neither form of the path is protected, inside or above a protected place, a `whole` folder, or
/// ignored, by any of `protections`. The gravest answer wins.
pub(crate) fn check_protection(placed: &Placed, protections: &[&Protection]) -> Result<(), SkipReason> {
    let answers = protections
        .iter()
        .flat_map(|protection| [&placed.physical, &placed.lexical].map(|path| protection.check(path)))
        .filter_map(Result::err);
    match answers.max_by_key(|answer| gravity(*answer)) {
        None => Ok(()),
        Some(Protected::Never | Protected::Ancestor) => Err(SkipReason::Protected),
        Some(Protected::WholeFolder) => Err(SkipReason::WholeFolder),
        Some(Protected::Ignored) => Err(SkipReason::Ignored),
    }
}

const fn gravity(answer: Protected) -> u8 {
    match answer {
        Protected::Never | Protected::Ancestor => 2,
        Protected::WholeFolder => 1,
        Protected::Ignored => 0,
    }
}

/// An item opened through no link, on its root's volume, kept open until it is acted on.
#[derive(Debug)]
pub(crate) struct Handle {
    file: File,
    metadata: Metadata,
}

impl Handle {
    /// Checks 5 and 6: opens the item at its physical path, a link as itself, refusing a link in any other
    /// component, and checks it is on its root's volume.
    pub(crate) fn open(placed: &Placed) -> Result<Self, SkipReason> {
        let file = open_no_link(&placed.physical, LastLink::AsItself).map_err(|error| open_failure(&error))?;
        let metadata = file.metadata().map_err(|_| SkipReason::CheckFailed)?;
        if metadata.dev() != placed.device {
            return Err(SkipReason::OtherVolume);
        }
        Ok(Self { file, metadata })
    }

    pub(crate) const fn file(&self) -> &File {
        &self.file
    }

    /// The item's metadata, read from the handle when it was opened.
    pub(crate) const fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use fresher_core::{Protection, SkipReason};
    use fresher_testing::FixtureTree;

    use super::{Handle, Placed, Roots, check_protection};

    fn tree() -> FixtureTree {
        FixtureTree::from_toml_str(
            r#"
            [[file]]
            path = "root/cache/a.bin"
            size = 10
            [[dir]]
            path = "root/keep/inner"
            [[file]]
            path = "root2/b.bin"
            [[file]]
            path = "outside/secret.txt"
            [[symlink]]
            path = "root/link-out"
            target = "../outside"
            [[symlink]]
            path = "root/link-in"
            target = "cache"
            "#,
        )
        .unwrap()
    }

    fn at(tree: &FixtureTree, relative: &str) -> PathBuf {
        tree.root().join(relative)
    }

    fn roots(tree: &FixtureTree) -> Roots {
        let root = vec![at(tree, "root")];
        Roots::new(&root, &root)
    }

    fn place(tree: &FixtureTree, relative: &str) -> Result<Placed, SkipReason> {
        roots(tree).place(&at(tree, relative))
    }

    fn open(tree: &FixtureTree, relative: &str) -> Result<Handle, SkipReason> {
        Handle::open(&place(tree, relative)?)
    }

    fn protection(tree: &FixtureTree, never: &[&str], whole: &[&str]) -> Protection {
        let paths = |list: &[&str]| list.iter().map(|relative| at(tree, relative)).collect();
        Protection::new(paths(never), paths(whole), vec![]).unwrap()
    }

    fn protected(tree: &FixtureTree, relative: &str, protection: &Protection) -> Result<(), SkipReason> {
        check_protection(&place(tree, relative).unwrap(), &[&Protection::default(), protection])
    }

    fn canonical(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap()
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::{Path, PathBuf};

        use fresher_core::{Protection, SkipReason};

        use super::{at, canonical, open, place, protected, protection, roots, tree};
        use crate::paths::{Roots, check_protection, expand_home};
        use crate::test_support::DiskImage;

        #[test]
        fn a_path_outside_every_root_is_refused() {
            let tree = tree();

            for relative in ["outside/secret.txt", "root2/b.bin"] {
                assert_eq!(place(&tree, relative), Err(SkipReason::OutsideRoots), "{relative}");
            }
            assert_eq!(roots(&tree).place(Path::new("/etc/hosts")), Err(SkipReason::OutsideRoots));
        }

        #[test]
        fn a_root_itself_is_refused() {
            let tree = tree();

            assert_eq!(place(&tree, "root"), Err(SkipReason::OutsideRoots));
            assert_eq!(roots(&tree).place(&canonical(&at(&tree, "root"))), Err(SkipReason::OutsideRoots));
        }

        #[test]
        fn an_ancestor_of_a_root_is_refused() {
            let tree = tree();
            let both = vec![at(&tree, "root"), at(&tree, "root/keep/inner")];

            assert_eq!(Roots::new(&both, &both).place(&at(&tree, "root/keep")), Err(SkipReason::OutsideRoots));
            assert_eq!(place(&tree, ""), Err(SkipReason::OutsideRoots));
        }

        #[test]
        fn a_dotted_path_is_refused() {
            let tree = tree();
            let root = at(&tree, "root");

            for path in [
                root.join("cache/../cache/a.bin"),
                root.join("./cache/a.bin"),
                root.join("cache/./a.bin"),
                root.join("cache/.."),
                PathBuf::from(format!("{}/../outside/secret.txt", root.display())),
            ] {
                assert_eq!(roots(&tree).place(&path), Err(SkipReason::OutsideRoots), "{}", path.display());
            }
        }

        #[test]
        fn a_relative_path_is_refused() {
            let tree = tree();

            assert_eq!(roots(&tree).place(Path::new("cache/a.bin")), Err(SkipReason::OutsideRoots));
            assert_eq!(roots(&tree).place(Path::new("")), Err(SkipReason::OutsideRoots));
        }

        #[test]
        fn a_root_no_longer_in_the_settings_is_refused() {
            let tree = tree();
            let roots = Roots::new(&[at(&tree, "root")], &[at(&tree, "root2")]);

            assert_eq!(roots.place(&at(&tree, "root/cache/a.bin")), Err(SkipReason::OutsideRoots));
            assert_eq!(roots.place(&at(&tree, "root2/b.bin")), Err(SkipReason::OutsideRoots));
        }

        #[test]
        fn a_root_that_cannot_be_read_fails_the_check() {
            let tree = tree();
            let gone = vec![at(&tree, "gone")];
            let file = vec![at(&tree, "root2/b.bin")];

            assert_eq!(Roots::new(&gone, &gone).place(&at(&tree, "gone/a")), Err(SkipReason::CheckFailed));
            assert_eq!(Roots::new(&file, &file).place(&at(&tree, "root2/b.bin/a")), Err(SkipReason::CheckFailed));
        }

        #[test]
        fn a_relative_or_dotted_root_is_never_used() {
            let tree = tree();
            let relative = vec![PathBuf::from("root")];
            let dotted = vec![PathBuf::from(format!("{}/root/../root", tree.root().display()))];

            assert_eq!(
                Roots::new(&relative, &relative).place(&at(&tree, "root/cache/a.bin")),
                Err(SkipReason::OutsideRoots)
            );
            assert_eq!(
                Roots::new(&dotted, &dotted).place(&at(&tree, "root/cache/a.bin")),
                Err(SkipReason::OutsideRoots)
            );
            assert_eq!(Roots::new(&relative, &relative).within(), Vec::<PathBuf>::new());
        }

        #[test]
        fn a_tilde_root_without_its_home_expanded_holds_nothing() {
            let tree = tree();
            let tilde = vec![PathBuf::from("~/root")];

            assert_eq!(Roots::new(&tilde, &tilde).place(&at(&tree, "root/cache/a.bin")), Err(SkipReason::OutsideRoots));
            assert_eq!(expand_home(&[PathBuf::from("~other/x")], Path::new("/home")), [PathBuf::from("~other/x")]);
        }

        #[test]
        fn the_system_protection_applies_when_the_context_has_none() {
            let tree = tree();
            let system = protection(&tree, &["root/keep"], &[]);
            let placed = place(&tree, "root/keep/inner").unwrap();

            assert_eq!(check_protection(&placed, &[&system, &Protection::default()]), Err(SkipReason::Protected));
        }

        #[test]
        fn a_shared_prefix_is_not_inside() {
            let tree = tree();
            let root = vec![PathBuf::from(format!("{}", at(&tree, "root").display()))];

            let placed = Roots::new(&root, &root).place(&PathBuf::from(format!("{}2/b.bin", root[0].display())));

            assert_eq!(placed, Err(SkipReason::OutsideRoots));
        }

        #[test]
        fn a_protected_path_is_refused() {
            let tree = tree();
            let never = protection(&tree, &["root/keep"], &[]);

            assert_eq!(protected(&tree, "root/keep", &never), Err(SkipReason::Protected));
            assert_eq!(protected(&tree, "root/keep/inner", &never), Err(SkipReason::Protected));
        }

        #[test]
        fn a_protected_physical_path_is_refused() {
            let tree = tree();
            let physical = Protection::new(vec![canonical(&at(&tree, "root/keep"))], vec![], vec![]).unwrap();

            assert_eq!(protected(&tree, "root/keep/inner", &physical), Err(SkipReason::Protected));
        }

        #[test]
        fn an_ancestor_of_a_protected_path_is_refused() {
            let tree = tree();
            let never = protection(&tree, &["root/keep/inner"], &[]);

            assert_eq!(protected(&tree, "root/keep", &never), Err(SkipReason::Protected));
        }

        #[test]
        fn a_whole_folder_is_refused() {
            let tree = tree();
            let whole = protection(&tree, &[], &["root/keep/inner"]);

            assert_eq!(protected(&tree, "root/keep/inner", &whole), Err(SkipReason::WholeFolder));
            assert_eq!(protected(&tree, "root/keep", &whole), Err(SkipReason::WholeFolder));
        }

        #[test]
        fn an_ignored_path_is_refused() {
            let tree = tree();
            let ignoring = Protection::default().with_ignored(&[at(&tree, "root/cache")]).unwrap();

            assert_eq!(protected(&tree, "root/cache", &ignoring), Err(SkipReason::Ignored));
            assert_eq!(protected(&tree, "root/cache/a.bin", &ignoring), Err(SkipReason::Ignored));
        }

        #[test]
        fn the_gravest_answer_wins() {
            let tree = tree();
            let both = protection(&tree, &["root/keep/inner"], &[]).with_ignored(&[at(&tree, "root/keep")]).unwrap();

            assert_eq!(protected(&tree, "root/keep/inner", &both), Err(SkipReason::Protected));
        }

        #[test]
        fn a_link_in_the_middle_is_refused() {
            let tree = tree();

            assert_eq!(open(&tree, "root/link-in/a.bin").unwrap_err(), SkipReason::ThroughLink);
        }

        #[test]
        fn a_link_to_outside_the_root_is_refused() {
            let tree = tree();

            assert_eq!(open(&tree, "root/link-out/secret.txt").unwrap_err(), SkipReason::ThroughLink);
        }

        #[test]
        fn a_vanished_item_is_reported() {
            let tree = tree();

            assert_eq!(open(&tree, "root/cache/gone.bin").unwrap_err(), SkipReason::Vanished);
        }

        #[test]
        fn a_file_in_the_middle_fails_the_check() {
            let tree = tree();

            assert_eq!(open(&tree, "root/cache/a.bin/x").unwrap_err(), SkipReason::CheckFailed);
        }

        #[test]
        fn an_item_on_another_volume_is_refused() {
            let tree = tree();
            let Some(image) = DiskImage::mount_at(&at(&tree, "root/cache/mounted")) else { return };
            fs::write(image.mount_point().join("on-the-image"), b"x").unwrap();

            assert_eq!(open(&tree, "root/cache/mounted").unwrap_err(), SkipReason::OtherVolume);
            assert_eq!(open(&tree, "root/cache/mounted/on-the-image").unwrap_err(), SkipReason::OtherVolume);
        }

        #[test]
        fn a_link_as_the_root_is_followed_once_and_named_by_both_forms() {
            let tree = tree();
            symlink(at(&tree, "root"), at(&tree, "alias")).unwrap();
            let alias = vec![at(&tree, "alias")];

            let placed = Roots::new(&alias, &alias).place(&at(&tree, "alias/cache/a.bin")).unwrap();

            assert_eq!(placed.physical, canonical(&at(&tree, "root/cache/a.bin")));
            assert_eq!(Roots::new(&alias, &alias).place(&canonical(&at(&tree, "root"))), Err(SkipReason::OutsideRoots));
        }
    }

    mod positive {
        use std::os::unix::fs::MetadataExt as _;

        use fresher_core::Protection;

        use super::{at, canonical, open, place, protected, tree};

        #[test]
        fn an_item_inside_a_root_opens() {
            let tree = tree();

            let handle = open(&tree, "root/cache/a.bin").unwrap();

            assert_eq!(
                handle.metadata().ino(),
                std::fs::symlink_metadata(at(&tree, "root/cache/a.bin")).unwrap().ino()
            );
            assert_eq!(handle.metadata().len(), 10);
            assert!(open(&tree, "root/keep").unwrap().metadata().is_dir());
        }

        #[test]
        fn a_link_as_the_item_opens_as_the_link() {
            let tree = tree();

            let handle = open(&tree, "root/link-out").unwrap();

            assert!(handle.metadata().file_type().is_symlink());
            let link = std::fs::symlink_metadata(at(&tree, "root/link-out")).unwrap();
            assert_eq!(handle.metadata().ino(), link.ino());
        }

        #[test]
        fn a_root_under_a_system_link_is_canonical() {
            let tree = tree();
            assert!(at(&tree, "root").starts_with("/var") || at(&tree, "root").starts_with("/tmp"));

            let placed = place(&tree, "root/cache/a.bin").unwrap();

            assert!(placed.physical.starts_with("/private"));
            assert_eq!(placed.physical, canonical(&at(&tree, "root/cache/a.bin")));
            assert_eq!(placed.lexical, at(&tree, "root/cache/a.bin"));
            assert_eq!(placed.relative, ["cache", "a.bin"]);
        }

        #[test]
        fn an_unprotected_item_passes() {
            let tree = tree();

            assert_eq!(protected(&tree, "root/cache/a.bin", &Protection::default()), Ok(()));
        }

        #[test]
        fn a_root_written_with_a_tilde_is_under_the_home() {
            let tree = tree();
            let written = vec![std::path::PathBuf::from("~/root"), std::path::PathBuf::from("~")];

            let expanded = super::super::expand_home(&written, tree.root());
            let placed = super::Roots::new(&expanded[..1], &expanded).place(&at(&tree, "root/cache/a.bin")).unwrap();

            assert_eq!(expanded, [at(&tree, "root"), tree.root().to_path_buf()]);
            assert_eq!(placed.physical, canonical(&at(&tree, "root/cache/a.bin")));
        }

        #[test]
        fn the_deepest_root_places_the_item() {
            let tree = tree();
            let both = vec![at(&tree, "root"), at(&tree, "root/keep")];

            let placed = super::Roots::new(&both, &both).place(&at(&tree, "root/keep/inner")).unwrap();

            assert_eq!(placed.root, canonical(&at(&tree, "root/keep")));
            assert_eq!(placed.relative, ["inner"]);
        }

        #[test]
        fn the_snapshot_looks_under_both_forms_of_a_root() {
            let tree = tree();

            let within = super::roots(&tree).within();

            assert_eq!(within, [at(&tree, "root"), canonical(&at(&tree, "root"))]);
        }
    }
}
