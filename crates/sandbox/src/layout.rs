use std::fs;
use std::path::{Path, PathBuf};

use fresher_core::{KnownFolder, Protection, ProtectionError, SANDBOX_MARKER};
use fresher_testing::FakeKnownFolders;

use crate::SandboxError;

/// The app's data folder in sandbox mode, at the sandbox's top.
pub(crate) const DATA_FOLDER: &str = ".fresher-data";
/// The checklist of a run by hand, at the sandbox's top.
pub(crate) const CHECKLIST: &str = "EXPECTED.md";
/// The world of the case files (the moment of the generation and the apps that are open), at the sandbox's top.
pub(crate) const WORLD: &str = "world.toml";
/// The folder of every user's temporary and cache folders, relative to the sandbox, as macOS lays it out from `/`.
const USER_FOLDERS: &str = "private/var/folders";
/// The sandbox user's own temporary folder.
pub(crate) const USER_TEMP: &str = "private/var/folders/xx/sandbox/T";
/// The temporary folder every user shares.
pub(crate) const SHARED_TEMP: &str = "private/tmp";
/// The system's temporary folder, which the system keeps.
const SYSTEM_TEMP: &str = "private/var/tmp";
/// Inside the user's own `T`: the system's own temporary items.
const TEMPORARY_ITEMS: &str = "TemporaryItems";
/// The folders of a sandbox, relative to it: the fake home, its Trash, the apps folders, the app's data folder, and
/// the temporary places as macOS lays them out from `/` (the user's triplet, its `T` among them, and `private/tmp`).
pub(crate) const PARTS: [&str; 10] = [
    "home",
    "home/.Trash",
    "Applications",
    "System/Applications",
    DATA_FOLDER,
    "private/var/folders/xx/sandbox/C",
    "private/var/folders/xx/sandbox/0",
    USER_TEMP,
    "private/var/folders/xx/sandbox/X",
    SHARED_TEMP,
];

/// A sandbox folder: the marker at its top, the fake home `home/` and the app's places around it
/// (docs/plans/007-safe-development/00-design.md, "The sandbox folder").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sandbox {
    path: PathBuf,
}

impl Sandbox {
    /// The sandbox at `path`, checked: `path` is a folder and no link, it holds a valid marker of `user` (a regular
    /// file opened through no link whose first line is the signature) and every part of the layout is a folder.
    pub fn open(path: &Path, user: u32) -> Result<Self, SandboxError> {
        let sandbox = Self { path: canonical_folder(path)? };
        if !holds_marker(&sandbox.path, user) {
            return Err(SandboxError::NoMarker { folder: sandbox.path });
        }
        for part in PARTS {
            let path = sandbox.path.join(part);
            if !fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.is_dir()) {
                return Err(SandboxError::MissingPart { path });
            }
        }
        Ok(sandbox)
    }

    /// The sandbox at an already checked canonical folder.
    pub(crate) const fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The sandbox folder, canonical.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The marker file at the top of the sandbox.
    pub fn marker(&self) -> PathBuf {
        self.path.join(SANDBOX_MARKER)
    }

    /// The fake home: `~` of the scan roots in sandbox mode.
    pub fn home(&self) -> PathBuf {
        self.path.join("home")
    }

    /// The sandbox's Trash, `home/.Trash`.
    pub fn trash(&self) -> PathBuf {
        self.home().join(".Trash")
    }

    /// The app's data folder in sandbox mode: settings, scan cache and cleanup journal.
    pub fn data_folder(&self) -> PathBuf {
        self.path.join(DATA_FOLDER)
    }

    /// The checklist of a run by hand.
    pub fn checklist(&self) -> PathBuf {
        self.path.join(CHECKLIST)
    }

    /// The world of the case files: the moment of the generation and the apps that are open in sandbox mode.
    pub fn world(&self) -> PathBuf {
        self.path.join(WORLD)
    }

    /// The sandbox user's own temporary folder, `private/var/folders/xx/sandbox/T`.
    pub fn user_temp(&self) -> PathBuf {
        self.path.join(USER_TEMP)
    }

    /// The sandbox's shared temporary folder, `private/tmp`.
    pub fn shared_temp(&self) -> PathBuf {
        self.path.join(SHARED_TEMP)
    }

    /// The known folders of the sandbox: the user's under `home/`, the apps folders at `Applications/` and
    /// `System/Applications/`, the temporary folders under `private/`: never the Mac's.
    pub fn known_folders(&self) -> FakeKnownFolders {
        FakeKnownFolders::home_layout(self.home())
            .with(KnownFolder::Applications, self.path.join("Applications"))
            .with(KnownFolder::SystemApplications, self.path.join("System").join("Applications"))
            .with(KnownFolder::UserTemp, self.user_temp())
            .with(KnownFolder::SharedTemp, self.shared_temp())
    }

    /// The protection of the sandbox's temporary places, as the Mac's list protects the real ones: every user's
    /// folders and the system's temporary folder never, but the user's own `T`, carved out and kept whole, and its
    /// `TemporaryItems` never; the shared temporary folder kept whole. A list of its own, joined to the others: the
    /// sandbox's `T` is no user's temporary folder of the Mac, so the Mac's list never carves it.
    pub fn places_protection(&self) -> Result<Protection, ProtectionError> {
        let temp = self.user_temp();
        Protection::new(
            vec![self.path.join(USER_FOLDERS), self.path.join(SYSTEM_TEMP), temp.join(TEMPORARY_ITEMS)],
            vec![temp.clone(), self.shared_temp()],
            vec![temp],
        )
    }
}

/// The canonical form of `path`, which must be an existing folder and no link itself. The cleaner's
/// `SandboxFolder::new` (`crates/cleaner/src/reach.rs`) checks a sandbox folder the same way: change both together.
pub(crate) fn canonical_folder(path: &Path) -> Result<PathBuf, SandboxError> {
    // Rebuilt from its components: a trailing `/` would make the `lstat` below follow a link.
    let path = &path.components().collect::<PathBuf>();
    let metadata = fs::symlink_metadata(path).map_err(|source| SandboxError::Io { path: path.into(), source })?;
    if metadata.is_symlink() {
        return Err(SandboxError::ThroughLink { path: path.into() });
    }
    if !metadata.is_dir() {
        return Err(SandboxError::NotAFolder { path: path.into() });
    }
    fs::canonicalize(path).map_err(|source| SandboxError::Io { path: path.into(), source })
}

/// Whether the canonical `folder` holds a valid marker of `user`: the one reader the cleaner's reach uses too.
#[cfg(target_os = "macos")]
pub(crate) use fresher_macos::sandbox_marker::holds_marker;

/// Off macOS no marker counts (fail closed).
#[cfg(not(target_os = "macos"))]
pub(crate) const fn holds_marker(_folder: &Path, _user: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::Path;

    use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

    use super::PARTS;

    fn user(folder: &Path) -> u32 {
        fs::metadata(folder).unwrap().uid()
    }

    /// A folder with every part of the layout and, when `marker` is given, that marker.
    fn laid_out(marker: Option<&str>) -> tempfile::TempDir {
        laid_out_but(marker, "")
    }

    /// A folder with every part of the layout but `missing`.
    fn laid_out_but(marker: Option<&str>, missing: &str) -> tempfile::TempDir {
        let folder = tempfile::tempdir().unwrap();
        for part in PARTS.into_iter().filter(|part| *part != missing) {
            fs::create_dir_all(folder.path().join(part)).unwrap();
        }
        if let Some(text) = marker {
            fs::write(folder.path().join(SANDBOX_MARKER), text).unwrap();
        }
        folder
    }

    fn signed() -> String {
        format!("{SANDBOX_SIGNATURE}\ncase files: trial.toml\n")
    }

    mod negative {
        use std::os::unix::fs::symlink;

        use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

        use fresher_core::Protected;

        use super::{laid_out, laid_out_but, signed, user};
        use crate::{Sandbox, SandboxError};

        #[test]
        fn a_folder_without_a_marker_does_not_open() {
            let folder = laid_out(None);

            let error = Sandbox::open(folder.path(), user(folder.path())).unwrap_err();

            assert!(matches!(error, SandboxError::NoMarker { .. }));
        }

        #[test]
        fn a_marker_with_another_first_line_does_not_count() {
            for text in
                ["", "fresher-sandbox 0\n", &format!("{SANDBOX_SIGNATURE}x\n"), &format!("x{SANDBOX_SIGNATURE}")]
            {
                let folder = laid_out(Some(text));

                let error = Sandbox::open(folder.path(), user(folder.path())).unwrap_err();

                assert!(matches!(error, SandboxError::NoMarker { .. }), "{text:?}");
            }
        }

        #[test]
        fn a_marker_of_another_user_or_through_a_link_does_not_count() {
            let folder = laid_out(Some(&signed()));
            let other = user(folder.path()).wrapping_add(1);
            assert!(matches!(Sandbox::open(folder.path(), other).unwrap_err(), SandboxError::NoMarker { .. }));

            let linked = laid_out(None);
            symlink(folder.path().join(SANDBOX_MARKER), linked.path().join(SANDBOX_MARKER)).unwrap();
            let error = Sandbox::open(linked.path(), user(linked.path())).unwrap_err();
            assert!(matches!(error, SandboxError::NoMarker { .. }));
        }

        #[test]
        fn a_sandbox_that_is_a_link_does_not_open() {
            let folder = laid_out(Some(&signed()));
            let outer = tempfile::tempdir().unwrap();
            let link = outer.path().join("sandbox");
            symlink(folder.path(), &link).unwrap();

            let error = Sandbox::open(&link, user(folder.path())).unwrap_err();

            assert!(matches!(error, SandboxError::ThroughLink { .. }));
        }

        #[test]
        fn a_sandbox_through_a_link_with_a_trailing_slash_does_not_open() {
            let folder = laid_out(Some(&signed()));
            let outer = tempfile::tempdir().unwrap();
            let link = outer.path().join("sandbox");
            symlink(folder.path(), &link).unwrap();

            let error = Sandbox::open(&link.join(""), user(folder.path())).unwrap_err();

            assert!(matches!(error, SandboxError::ThroughLink { .. }));
        }

        #[test]
        fn the_sandboxs_temporary_places_are_protected_as_the_macs() {
            let folder = laid_out(Some(&signed()));
            let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();
            let protection = sandbox.places_protection().unwrap();

            for (path, answer) in [
                // Its `TemporaryItems` is never: `T` is refused as its ancestor, before it is refused whole.
                ("private/var/folders/xx/sandbox/T", Protected::Ancestor),
                ("private/tmp", Protected::WholeFolder),
                ("private/var/folders/xx/sandbox/T/TemporaryItems", Protected::Never),
                ("private/var/folders/xx/sandbox/T/TemporaryItems/doc", Protected::Never),
                ("private/var/folders/xx/sandbox/C/cache", Protected::Never),
                ("private/var/folders/xx/sandbox/0/data", Protected::Never),
                ("private/var/folders/xx/sandbox/X/code", Protected::Never),
                ("private/var/folders/yy/other/T/item", Protected::Never),
                ("private/var/tmp/item", Protected::Never),
                ("private/var/folders", Protected::Never),
                ("private", Protected::Ancestor),
            ] {
                assert_eq!(protection.check(&sandbox.path().join(path)), Err(answer), "{path}");
            }
        }

        #[test]
        fn a_missing_or_linked_part_does_not_open() {
            let folder = laid_out_but(Some(&signed()), "System/Applications");
            let error = Sandbox::open(folder.path(), user(folder.path())).unwrap_err();
            assert!(matches!(error, SandboxError::MissingPart { path } if path.ends_with("System/Applications")));

            let linked = laid_out_but(Some(&signed()), ".fresher-data");
            symlink(folder.path(), linked.path().join(".fresher-data")).unwrap();
            let error = Sandbox::open(linked.path(), user(linked.path())).unwrap_err();
            assert!(matches!(error, SandboxError::MissingPart { path } if path.ends_with(".fresher-data")));
        }
    }

    mod positive {
        use fresher_core::{KnownFolder, KnownFolders};

        use super::{laid_out, signed, user};
        use crate::Sandbox;

        #[test]
        fn a_laid_out_folder_with_a_marker_opens_with_its_known_folders() {
            let folder = laid_out(Some(&signed()));

            let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();

            let root = std::fs::canonicalize(folder.path()).unwrap();
            assert_eq!(sandbox.path(), root);
            assert_eq!(sandbox.trash(), root.join("home/.Trash"));
            assert_eq!(sandbox.data_folder(), root.join(".fresher-data"));
            let folders = sandbox.known_folders();
            let locate = |folder| folders.locate(folder);
            assert_eq!(locate(KnownFolder::Home), Some(root.join("home")));
            assert_eq!(locate(KnownFolder::Trash), Some(sandbox.trash()));
            assert_eq!(locate(KnownFolder::Applications), Some(root.join("Applications")));
            assert_eq!(locate(KnownFolder::SystemApplications), Some(root.join("System/Applications")));
            assert_eq!(locate(KnownFolder::UserTemp), Some(root.join("private/var/folders/xx/sandbox/T")));
            assert_eq!(locate(KnownFolder::SharedTemp), Some(root.join("private/tmp")));
        }

        #[test]
        fn only_items_inside_the_sandboxs_temporary_places_may_go() {
            let folder = laid_out(Some(&signed()));
            let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();
            let protection = sandbox.places_protection().unwrap();

            for inside in ["private/var/folders/xx/sandbox/T/item", "private/tmp/item", "private/tmp/a/b"] {
                assert_eq!(protection.check(&sandbox.path().join(inside)), Ok(()), "{inside}");
            }
        }
    }
}
