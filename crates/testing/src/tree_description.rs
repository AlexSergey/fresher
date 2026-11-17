use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::{MetadataExt as _, symlink};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::Deserialize;

use crate::app_bundle::AppBundle;
use crate::fixture_tree::{FixtureError, write_marker};

/// The "now" of a fixture home, in seconds since the Unix epoch: an `age` is counted back from it, so a built tree
/// never depends on the wall clock.
pub const FIXTURE_NOW: u64 = 1_767_225_600;

const MAX_FILLED_FILE: u64 = 64 * 1024 * 1024;
const MAX_FILLED_TREE: u64 = 512 * 1024 * 1024;
const FILL_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

/// A tree of files, folders, links, sandbox markers and app bundles as plain data (the format: docs/testing.md, "Fixture homes").
///
/// It is checked when parsed or merged and built with [`TreeDescription::build_at`] into any existing folder.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TreeDescription {
    #[serde(default, rename = "dir")]
    dirs: Vec<DirSpec>,
    #[serde(default, rename = "file")]
    files: Vec<FileSpec>,
    #[serde(default, rename = "symlink")]
    symlinks: Vec<SymlinkSpec>,
    #[serde(default, rename = "hardlink")]
    hardlinks: Vec<HardlinkSpec>,
    #[serde(default, rename = "marker")]
    markers: Vec<MarkerSpec>,
    #[serde(default, rename = "app")]
    apps: Vec<AppBundle>,
}

/// A folder; its time, when given, is set once everything inside it is written.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DirSpec {
    path: PathBuf,
    /// Seconds since the Unix epoch.
    modified: Option<u64>,
    /// `"<n>d"` or `"<n>h"` before the "now" of the build; never together with `modified`.
    age: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSpec {
    path: PathBuf,
    #[serde(default)]
    size: u64,
    /// Seconds since the Unix epoch.
    modified: Option<u64>,
    /// `"<n>d"` or `"<n>h"` before the "now" of the build; never together with `modified`.
    age: Option<String>,
    #[serde(default)]
    fill: Fill,
}

/// What a file holds: nothing (sparse, costs no disk) or real bytes of a fixed pattern.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Fill {
    #[default]
    Sparse,
    Data,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SymlinkSpec {
    path: PathBuf,
    target: PathBuf,
}

/// A second name of a file of the tree; `target` is relative to the root too.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HardlinkSpec {
    path: PathBuf,
    target: PathBuf,
}

/// A folder that receives the sandbox marker.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MarkerSpec {
    path: PathBuf,
}

impl TreeDescription {
    /// Parses and checks a TOML description.
    pub fn from_toml_str(text: &str) -> Result<Self, FixtureError> {
        let description: Self = toml::from_str(text)
            .map_err(|source| FixtureError::Parse { path: PathBuf::from("<inline>"), source: Box::new(source) })?;
        description.check()?;
        Ok(description)
    }

    /// Joins two descriptions; a path written by both is refused.
    pub fn merge(mut self, other: Self) -> Result<Self, FixtureError> {
        self.dirs.extend(other.dirs);
        self.files.extend(other.files);
        self.symlinks.extend(other.symlinks);
        self.hardlinks.extend(other.hardlinks);
        self.markers.extend(other.markers);
        self.apps.extend(other.apps);
        self.check()?;
        Ok(self)
    }

    /// The paths the tree writes (its folders, files, links and app bundles), relative to its root as written.
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        (self.dirs.iter().map(|spec| spec.path.as_path()))
            .chain(self.files.iter().map(|spec| spec.path.as_path()))
            .chain(self.symlinks.iter().map(|spec| spec.path.as_path()))
            .chain(self.hardlinks.iter().map(|spec| spec.path.as_path()))
            .chain(self.apps.iter().map(|app| app.path.as_path()))
    }

    /// The app bundles the tree writes, in the order written.
    pub fn apps(&self) -> &[AppBundle] {
        &self.apps
    }

    /// The files the tree writes, in the order written, relative to its root as written.
    pub fn files(&self) -> impl Iterator<Item = &Path> {
        self.files.iter().map(|spec| spec.path.as_path())
    }

    /// The targets of the hard links, relative to the tree's root as written; a symlink's target is any path.
    pub fn hardlink_targets(&self) -> impl Iterator<Item = &Path> {
        self.hardlinks.iter().map(|spec| spec.target.as_path())
    }

    /// The folders that receive the sandbox marker, relative to the tree's root as written.
    pub fn markers(&self) -> impl Iterator<Item = &Path> {
        self.markers.iter().map(|spec| spec.path.as_path())
    }

    /// Builds the tree inside the existing `folder`; an `age` is counted back from `now`.
    ///
    /// Refuses a path that leaves the folder or passes through a link. Built again on a folder it built before, it
    /// tops up: rewrites its files and markers, writes what is missing and keeps a link that is already as described.
    pub fn build_at(&self, folder: &Path, now: SystemTime) -> Result<(), FixtureError> {
        self.check()?;
        for dir in &self.dirs {
            let path = resolve(folder, &dir.path)?;
            fs::create_dir_all(&path).map_err(|source| FixtureError::Create { path, source })?;
        }
        for file in &self.files {
            create_file(folder, file, now)?;
        }
        for link in &self.symlinks {
            let path = match resolve(folder, &link.path) {
                Err(FixtureError::PathThroughLink { path })
                    if path == folder.join(&link.path) && fs::read_link(&path).is_ok_and(|to| to == link.target) =>
                {
                    continue;
                }
                resolved => resolved?,
            };
            create_parent(&path)?;
            symlink(&link.target, &path).map_err(|source| FixtureError::Create { path, source })?;
        }
        for link in &self.hardlinks {
            let (path, target) = (resolve(folder, &link.path)?, resolve(folder, &link.target)?);
            if same_file(&path, &target) {
                continue;
            }
            create_parent(&path)?;
            fs::hard_link(&target, &path).map_err(|source| FixtureError::Create { path, source })?;
        }
        for marker in &self.markers {
            let path = resolve(folder, &marker.path)?;
            fs::create_dir_all(&path).map_err(|source| FixtureError::Create { path: path.clone(), source })?;
            write_marker(&path)?;
        }
        for app in &self.apps {
            app.write_at(folder)?;
        }
        self.set_folder_times(folder, now)
    }

    /// Sets the time of every folder that names one, the deepest first: writing inside a folder changes its time, so
    /// this comes last.
    fn set_folder_times(&self, folder: &Path, now: SystemTime) -> Result<(), FixtureError> {
        let mut dated: Vec<&DirSpec> =
            self.dirs.iter().filter(|dir| dir.age.is_some() || dir.modified.is_some()).collect();
        dated.sort_by_key(|dir| std::cmp::Reverse(dir.path.components().count()));
        for dir in dated {
            let path = resolve(folder, &dir.path)?;
            let create_error = |source| FixtureError::Create { path: path.clone(), source };
            if let Some(time) = time_of(&dir.path, dir.age.as_deref(), dir.modified, now)? {
                File::open(&path).and_then(|opened| opened.set_modified(time)).map_err(create_error)?;
            }
        }
        Ok(())
    }

    fn check(&self) -> Result<(), FixtureError> {
        let mut written = HashSet::new();
        for path in self.paths() {
            if !written.insert(path) {
                return Err(FixtureError::DuplicatePath { path: path.to_path_buf() });
            }
        }
        for app in &self.apps {
            app.check()?;
        }
        let entries = (self.dirs.iter().map(|spec| &spec.path))
            .chain(self.files.iter().map(|spec| &spec.path))
            .chain(self.symlinks.iter().map(|spec| &spec.path))
            .chain(self.hardlinks.iter().map(|spec| &spec.path))
            .chain(self.markers.iter().map(|spec| &spec.path));
        for path in entries {
            // The bundle rewrites what it holds, so an entry inside it would be overwritten or never read.
            if let Some(app) = self.apps.iter().find(|app| path.starts_with(&app.path)) {
                return Err(FixtureError::InsideApp { path: path.clone(), app: app.path.clone() });
            }
        }
        for dir in &self.dirs {
            check_time(&dir.path, dir.age.as_deref(), dir.modified)?;
        }
        let mut filled = 0u64;
        for file in &self.files {
            check_time(&file.path, file.age.as_deref(), file.modified)?;
            if file.fill == Fill::Data {
                filled = filled.saturating_add(file.size);
                if file.size > MAX_FILLED_FILE || filled > MAX_FILLED_TREE {
                    return Err(FixtureError::FillOverCap { path: file.path.clone() });
                }
            }
        }
        Ok(())
    }
}

/// `"30d"` or `"12h"` as a duration.
fn parse_age(text: &str) -> Option<Duration> {
    let (count, per_unit) = match (text.strip_suffix('d'), text.strip_suffix('h')) {
        (Some(count), _) => (count, 24),
        (None, Some(count)) => (count, 1),
        (None, None) => return None,
    };
    let hours = count.parse::<u64>().ok()?.checked_mul(per_unit)?;
    Some(Duration::from_hours(hours))
}

/// Refuses an `age` together with `modified`, and an age not of the form `"<n>d"` or `"<n>h"`.
fn check_time(path: &Path, age: Option<&str>, modified: Option<u64>) -> Result<(), FixtureError> {
    if age.is_some() && modified.is_some() {
        return Err(FixtureError::AgeWithModified { path: path.to_path_buf() });
    }
    if let Some(age) = age {
        parse_age(age).ok_or_else(|| FixtureError::InvalidAge { path: path.to_path_buf() })?;
    }
    Ok(())
}

/// The time an entry is given: its age counted back from `now`, or its `modified` seconds; `None` for neither.
fn time_of(
    path: &Path,
    age: Option<&str>,
    modified: Option<u64>,
    now: SystemTime,
) -> Result<Option<SystemTime>, FixtureError> {
    Ok(match (age, modified) {
        (Some(age), _) => Some(
            parse_age(age)
                .and_then(|age| now.checked_sub(age))
                .ok_or_else(|| FixtureError::InvalidAge { path: path.to_path_buf() })?,
        ),
        (None, Some(seconds)) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
        (None, None) => None,
    })
}

fn create_file(folder: &Path, spec: &FileSpec, now: SystemTime) -> Result<(), FixtureError> {
    let path = resolve(folder, &spec.path)?;
    let modified = time_of(&spec.path, spec.age.as_deref(), spec.modified, now)?;
    create_parent(&path)?;
    let create_error = |source| FixtureError::Create { path: path.clone(), source };
    let mut file = File::create(&path).map_err(create_error)?;
    match spec.fill {
        Fill::Sparse => file.set_len(spec.size).map_err(create_error)?,
        // Synced, so the blocks are allocated before a walk reads the file's size on disk.
        Fill::Data => write_pattern(&mut file, spec.size).and_then(|()| file.sync_all()).map_err(create_error)?,
    }
    if let Some(time) = modified {
        file.set_modified(time).map_err(create_error)?;
    }
    Ok(())
}

/// Writes `size` bytes of a fixed pseudo-random pattern: the same bytes every time, and not compressible to nothing.
fn write_pattern(file: &mut File, size: u64) -> std::io::Result<()> {
    let mut state = FILL_SEED;
    let mut chunk = vec![0u8; 64 * 1024];
    let mut left = size;
    while left > 0 {
        for bytes in chunk.chunks_mut(8) {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let word = state.to_le_bytes();
            bytes.copy_from_slice(&word[..bytes.len()]);
        }
        let take = usize::try_from(left).map_or(chunk.len(), |left| left.min(chunk.len()));
        file.write_all(&chunk[..take])?;
        left -= take as u64;
    }
    Ok(())
}

/// The path below `folder`; it must be relative, stay inside and pass through no link.
pub(crate) fn resolve(folder: &Path, relative: &Path) -> Result<PathBuf, FixtureError> {
    let inside = relative.components().all(|component| matches!(component, Component::Normal(_)));
    if !inside || relative.as_os_str().is_empty() {
        return Err(FixtureError::PathOutsideRoot { path: relative.to_path_buf() });
    }
    let mut path = folder.to_path_buf();
    for component in relative.components() {
        path.push(component);
        if fs::symlink_metadata(&path).is_ok_and(|metadata| metadata.is_symlink()) {
            return Err(FixtureError::PathThroughLink { path });
        }
    }
    Ok(path)
}

/// Whether both paths name one file (a hard link already made); `false` when either is missing.
fn same_file(first: &Path, second: &Path) -> bool {
    match (fs::symlink_metadata(first), fs::symlink_metadata(second)) {
        (Ok(first), Ok(second)) => first.dev() == second.dev() && first.ino() == second.ino(),
        _ => false,
    }
}

fn create_parent(path: &Path) -> Result<(), FixtureError> {
    match path.parent() {
        Some(parent) => {
            fs::create_dir_all(parent).map_err(|source| FixtureError::Create { path: parent.into(), source })
        }
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fmt::Write;
        use std::time::SystemTime;

        use crate::{FixtureError, TreeDescription};

        fn parse(text: &str) -> Result<TreeDescription, FixtureError> {
            TreeDescription::from_toml_str(text)
        }

        #[test]
        fn age_and_modified_together_are_refused() {
            let error = parse("[[file]]\npath = \"a\"\nage = \"1d\"\nmodified = 5\n").unwrap_err();

            assert!(matches!(error, FixtureError::AgeWithModified { .. }));
        }

        #[test]
        fn an_age_without_a_known_unit_is_refused() {
            for age in ["30", "d", "-1d", "3w", "", "5д", "d5"] {
                for kind in ["file", "dir"] {
                    let error = parse(&format!("[[{kind}]]\npath = \"a\"\nage = \"{age}\"\n")).unwrap_err();

                    assert!(matches!(error, FixtureError::InvalidAge { .. }), "{kind} {age}");
                }
            }
        }

        #[test]
        fn a_folders_age_and_modified_together_are_refused() {
            let error = parse("[[dir]]\npath = \"a\"\nage = \"1d\"\nmodified = 5\n").unwrap_err();

            assert!(matches!(error, FixtureError::AgeWithModified { .. }));
        }

        #[test]
        fn an_entry_inside_a_declared_app_is_refused() {
            let app = "[[app]]\npath = \"Applications/Tool.app\"\nbundle-id = \"com.example.Tool\"\nname = \"Tool\"\n";
            for entry in [
                "[[file]]\npath = \"Applications/Tool.app/Contents/Info.plist\"\n",
                "[[dir]]\npath = \"Applications/Tool.app/Contents/Resources\"\n",
                "[[marker]]\npath = \"Applications/Tool.app/Contents\"\n",
            ] {
                let error = TreeDescription::from_toml_str(&format!("{app}{entry}")).unwrap_err();

                assert!(matches!(error, FixtureError::InsideApp { .. }), "{entry}: {error}");
            }
            let beside = format!("{app}[[file]]\npath = \"Applications/Tool.app.bak\"\n");
            assert!(TreeDescription::from_toml_str(&beside).is_ok());
        }

        #[test]
        fn a_fill_over_the_cap_is_refused() {
            let one = "[[file]]\npath = \"a\"\nsize = 67108865\nfill = \"data\"\n";
            let mut many = String::new();
            for n in 0..9 {
                writeln!(many, "[[file]]\npath = \"f{n}\"\nsize = 67108864\nfill = \"data\"").unwrap();
            }
            for text in [one.to_string(), many] {
                assert!(matches!(parse(&text).unwrap_err(), FixtureError::FillOverCap { .. }));
            }
        }

        #[test]
        fn a_sparse_file_may_exceed_the_fill_cap() {
            assert!(parse("[[file]]\npath = \"a\"\nsize = 10737418240\n").is_ok());
        }

        #[test]
        fn a_marker_outside_the_root_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            for path in ["../out", "/etc"] {
                let description = parse(&format!("[[marker]]\npath = \"{path}\"\n")).unwrap();

                let error = description.build_at(folder.path(), SystemTime::UNIX_EPOCH).unwrap_err();

                assert!(matches!(error, FixtureError::PathOutsideRoot { .. }), "{path}");
            }
        }

        #[test]
        fn merging_two_descriptions_that_write_one_path_is_refused() {
            let first = parse("[[file]]\npath = \"a\"\n").unwrap();
            let second = parse("[[dir]]\npath = \"a\"\n").unwrap();

            let error = first.merge(second).unwrap_err();

            assert!(matches!(error, FixtureError::DuplicatePath { .. }));
        }

        #[test]
        fn an_app_that_is_not_a_bundle_is_refused() {
            let app =
                |path: &str, id: &str| format!("[[app]]\npath = \"{path}\"\nbundle-id = \"{id}\"\nname = \"T\"\n");

            assert!(matches!(parse(&app("Applications/T", "com.example.T")), Err(FixtureError::AppNotABundle { .. })));
            assert!(matches!(parse(&app("Applications/../T.app", "a.b")), Err(FixtureError::PathOutsideRoot { .. })));
            assert!(matches!(
                parse(&app("Applications/T.app", "not an id")),
                Err(FixtureError::InvalidBundleId { .. })
            ));
            assert!(matches!(
                parse(&format!("{}colour = 1\n", app("A/T.app", "a.b"))),
                Err(FixtureError::Parse { .. })
            ));
        }

        #[test]
        fn an_app_and_a_file_at_one_path_are_refused() {
            let first = parse("[[app]]\npath = \"A/T.app\"\nbundle-id = \"a.b\"\nname = \"T\"\n").unwrap();
            let second = parse("[[dir]]\npath = \"A/T.app\"\n").unwrap();

            assert!(matches!(first.clone().merge(first.clone()), Err(FixtureError::DuplicatePath { .. })));
            assert!(matches!(first.merge(second), Err(FixtureError::DuplicatePath { .. })));
        }

        #[test]
        fn build_at_refuses_a_path_through_a_link() {
            let outside = tempfile::tempdir().unwrap();
            let folder = tempfile::tempdir().unwrap();
            std::os::unix::fs::symlink(outside.path(), folder.path().join("home")).unwrap();
            let description = parse("[[file]]\npath = \"home/a\"\nsize = 1\n").unwrap();

            let error = description.build_at(folder.path(), SystemTime::UNIX_EPOCH).unwrap_err();

            assert!(matches!(error, FixtureError::PathThroughLink { .. }));
            assert!(!outside.path().join("a").exists());
        }

        #[test]
        fn build_at_replaces_no_link_of_another_target() {
            let folder = tempfile::tempdir().unwrap();
            std::os::unix::fs::symlink("/tmp", folder.path().join("out")).unwrap();
            let description = parse("[[symlink]]\npath = \"out\"\ntarget = \"/etc\"\n").unwrap();

            let error = description.build_at(folder.path(), SystemTime::UNIX_EPOCH).unwrap_err();

            assert!(matches!(error, FixtureError::PathThroughLink { .. }));
            assert_eq!(std::fs::read_link(folder.path().join("out")).unwrap(), std::path::Path::new("/tmp"));
        }
    }

    mod positive {
        use std::os::unix::fs::MetadataExt;
        use std::time::{Duration, SystemTime};

        use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

        use crate::{FIXTURE_NOW, FixtureTree, TreeDescription, TreeReport};

        fn now() -> SystemTime {
            SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW)
        }

        // FixtureTree and build_at agree on every fixture home.
        #[test]
        fn every_existing_fixture_builds_as_before() {
            let homes = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/homes");
            let mut built = 0;
            for entry in std::fs::read_dir(homes).unwrap() {
                let path = entry.unwrap().path();
                let tree = FixtureTree::from_toml(&path).unwrap();
                let again = tempfile::tempdir().unwrap();
                let text = std::fs::read_to_string(&path).unwrap();
                TreeDescription::from_toml_str(&text).unwrap().build_at(again.path(), now()).unwrap();

                assert_eq!(TreeReport::of(tree.root()).unwrap(), TreeReport::of(again.path()).unwrap());
                built += 1;
            }
            assert!(built >= 8);
        }

        #[test]
        fn an_age_is_counted_back_from_now() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"d\"\nage = \"30d\"\n[[file]]\npath = \"h\"\nage = \"5h\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            let modified = |name: &str| std::fs::metadata(folder.path().join(name)).unwrap().modified().unwrap();
            assert_eq!(modified("d"), now() - Duration::from_hours(30 * 24));
            assert_eq!(modified("h"), now() - Duration::from_hours(5));
        }

        #[test]
        fn a_folders_age_is_its_time_after_everything_inside_is_written() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[dir]]\npath = \"a\"\nage = \"40d\"\n[[dir]]\npath = \"a/b\"\nmodified = 1700000000\n\
                        [[file]]\npath = \"a/b/c\"\nage = \"1d\"\n[[file]]\npath = \"a/d\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            let modified = |name: &str| std::fs::metadata(folder.path().join(name)).unwrap().modified().unwrap();
            assert_eq!(modified("a"), now() - Duration::from_hours(40 * 24));
            assert_eq!(modified("a/b"), SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000));
            assert_eq!(modified("a/b/c"), now() - Duration::from_hours(24));
        }

        #[test]
        fn filled_files_take_their_blocks() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"a\"\nsize = 1000000\nfill = \"data\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            let metadata = std::fs::metadata(folder.path().join("a")).unwrap();
            assert_eq!(metadata.len(), 1_000_000);
            assert!(metadata.blocks() * 512 >= 1_000_000);
        }

        #[test]
        fn a_fill_is_the_same_bytes_every_time() {
            let text = "[[file]]\npath = \"a\"\nsize = 100001\nfill = \"data\"\n";
            let bytes = || {
                let folder = tempfile::tempdir().unwrap();
                TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();
                std::fs::read(folder.path().join("a")).unwrap()
            };

            assert_eq!(bytes(), bytes());
        }

        #[test]
        fn a_marker_holds_the_signature() {
            let folder = tempfile::tempdir().unwrap();

            TreeDescription::from_toml_str("[[marker]]\npath = \"home/work\"\n")
                .unwrap()
                .build_at(folder.path(), now())
                .unwrap();

            let marker = std::fs::read_to_string(folder.path().join("home/work").join(SANDBOX_MARKER)).unwrap();
            assert_eq!(marker.lines().next(), Some(SANDBOX_SIGNATURE));
        }

        #[test]
        fn building_twice_tops_up_and_resets_the_files() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"a/b\"\nsize = 10\nage = \"1d\"\n[[symlink]]\npath = \"a/out\"\ntarget = \"/etc\"\n\
                        [[hardlink]]\npath = \"a/c\"\ntarget = \"a/b\"\n[[marker]]\npath = \"a\"\n";
            let description = TreeDescription::from_toml_str(text).unwrap();
            description.build_at(folder.path(), now()).unwrap();
            let first = TreeReport::of(folder.path()).unwrap();
            std::fs::write(folder.path().join("a/b"), "changed").unwrap();

            description.build_at(folder.path(), now()).unwrap();

            assert_eq!(TreeReport::of(folder.path()).unwrap(), first);
            let modified = std::fs::metadata(folder.path().join("a/c")).unwrap().modified().unwrap();
            assert_eq!(modified, now() - Duration::from_hours(24));
        }

        #[test]
        fn an_app_is_built_as_a_bundle() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[app]]\npath = \"Applications/Tool.app\"\nbundle-id = \"com.example.Tool\"\nname = \"Tool\"\n\
                        version = \"1.2\"\nbuild = \"12\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            let contents = folder.path().join("Applications/Tool.app/Contents");
            let plist = std::fs::read_to_string(contents.join("Info.plist")).unwrap();
            assert!(plist.contains("<key>CFBundleIdentifier</key>\n\t<string>com.example.Tool</string>"), "{plist}");
            assert!(plist.contains("<key>CFBundleVersion</key>\n\t<string>12</string>"), "{plist}");
            assert_eq!(std::fs::read_dir(contents.join("MacOS")).unwrap().count(), 0);
        }

        #[test]
        fn merged_descriptions_build_together() {
            let folder = tempfile::tempdir().unwrap();
            let first = TreeDescription::from_toml_str("[[file]]\npath = \"a\"\n").unwrap();
            let second = TreeDescription::from_toml_str("[[file]]\npath = \"b\"\n").unwrap();

            first.merge(second).unwrap().build_at(folder.path(), now()).unwrap();

            assert!(folder.path().join("a").is_file() && folder.path().join("b").is_file());
        }
    }
}
