use std::collections::{BTreeSet, HashSet};
use std::fs::{self, File, FileTimes, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{MetadataExt as _, OpenOptionsExt as _, symlink};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use fresher_core::Timestamp;
use serde::Deserialize;

use crate::app_bundle::AppBundle;
use crate::facts_lens::{Stated, StatedFlag, StatedKind, StatedOwner};
use crate::fixture_tree::{FixtureError, write_marker};

/// The "now" of a fixture home, in seconds since the Unix epoch: an `age` is counted back from it, so a built tree
/// never depends on the wall clock.
pub const FIXTURE_NOW: u64 = 1_767_225_600;

const MAX_FILLED_FILE: u64 = 64 * 1024 * 1024;
const MAX_FILLED_TREE: u64 = 512 * 1024 * 1024;
const FILL_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
/// What a valid `CACHEDIR.TAG` holds: the standard's signature line, then a comment. The large trees write it too.
pub(crate) const CACHEDIR_TAG_CONTENT: &[u8] =
    b"Signature: 8a477f597d28d172789f06886806bc55\n# A cache directory tag.\n";

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
    /// `"<n>d"`, `"<n>h"` or `"<n>s"` before the "now" of the build (`"-2d"`: after it); never with `modified`.
    age: Option<String>,
    /// The time of its last status change, stated, as an `age`.
    changed: Option<String>,
    /// The time of its last access, stated, as an `age`.
    accessed: Option<String>,
    /// Who owns it, stated.
    owner: Option<StatedOwner>,
    /// Its flags, stated.
    #[serde(default)]
    flags: BTreeSet<StatedFlag>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSpec {
    path: PathBuf,
    #[serde(default)]
    size: u64,
    /// Seconds since the Unix epoch.
    modified: Option<u64>,
    /// `"<n>d"`, `"<n>h"` or `"<n>s"` before the "now" of the build (`"-2d"`: after it); never with `modified`.
    age: Option<String>,
    /// Absent: sparse.
    fill: Option<Fill>,
    /// The time of its last status change, stated, as an `age`.
    changed: Option<String>,
    /// The time of its last access, stated, as an `age`.
    accessed: Option<String>,
    /// Who owns it, stated.
    owner: Option<StatedOwner>,
    /// Its flags, stated.
    #[serde(default)]
    flags: BTreeSet<StatedFlag>,
    /// A socket or a pipe, stated: written as a plain file. Never with `fill`.
    kind: Option<StatedKind>,
}

/// What a file holds: nothing (sparse, costs no disk), real bytes of a fixed pattern, or a valid cache directory tag.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Fill {
    #[default]
    Sparse,
    Data,
    /// [`CACHEDIR_TAG_CONTENT`]; never with a `size`.
    CachedirTag,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SymlinkSpec {
    path: PathBuf,
    target: PathBuf,
    /// The link's own modification and access time, as an `age`; its target is never touched.
    age: Option<String>,
    /// The time of the link's last status change, stated, as an `age`.
    changed: Option<String>,
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

    /// The facts the tree states instead of making them (docs/testing.md, "Fixture homes"), its folders', its files',
    /// its links' then its app bundles' (`changed` only), relative to its root as written; times counted back from
    /// `now`. An entry that states nothing is left out.
    pub fn stated(&self, now: SystemTime) -> Result<Vec<(PathBuf, Stated)>, FixtureError> {
        let dirs = self
            .dirs
            .iter()
            .map(|dir| (&dir.path, dir.changed.as_deref(), dir.accessed.as_deref(), dir.owner, &dir.flags, None));
        let files = self.files.iter().map(|file| {
            (&file.path, file.changed.as_deref(), file.accessed.as_deref(), file.owner, &file.flags, file.kind)
        });
        let no_flags = BTreeSet::new();
        let links = self.symlinks.iter().map(|link| (&link.path, link.changed.as_deref(), None, None, &no_flags, None));
        let apps = self.apps.iter().map(|app| (&app.path, app.changed.as_deref(), None, None, &no_flags, None));
        let mut stated = Vec::new();
        for (path, changed, accessed, owner, flags, kind) in dirs.chain(files).chain(links).chain(apps) {
            let stamp = |offset: Option<&str>| -> Result<Option<Timestamp>, FixtureError> {
                Ok(time_of(path, offset, None, now)?.map(Timestamp::from_system_time))
            };
            let facts =
                Stated { changed: stamp(changed)?, accessed: stamp(accessed)?, owner, flags: flags.clone(), kind };
            if !facts.is_empty() {
                stated.push((path.clone(), facts));
            }
        }
        Ok(stated)
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
                    path
                }
                resolved => {
                    let path = resolved?;
                    create_parent(&path)?;
                    symlink(&link.target, &path)
                        .map_err(|source| FixtureError::Create { path: path.clone(), source })?;
                    path
                }
            };
            if let Some(time) = time_of(&link.path, link.age.as_deref(), None, now)? {
                set_link_time(&path, time).map_err(|source| FixtureError::Create { path, source })?;
            }
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
            check_stated_times(&app.path, &[app.changed.as_deref()])?;
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
            check_stated_times(&dir.path, &[dir.changed.as_deref(), dir.accessed.as_deref()])?;
        }
        for link in &self.symlinks {
            check_stated_times(&link.path, &[link.age.as_deref(), link.changed.as_deref()])?;
        }
        let mut filled = 0u64;
        for file in &self.files {
            check_time(&file.path, file.age.as_deref(), file.modified)?;
            check_stated_times(&file.path, &[file.changed.as_deref(), file.accessed.as_deref()])?;
            if file.kind.is_some() && file.fill.is_some() {
                return Err(FixtureError::KindWithFill { path: file.path.clone() });
            }
            if file.fill == Some(Fill::CachedirTag) && file.size != 0 {
                return Err(FixtureError::TagWithSize { path: file.path.clone() });
            }
            if file.fill == Some(Fill::Data) {
                filled = filled.saturating_add(file.size);
                if file.size > MAX_FILLED_FILE || filled > MAX_FILLED_TREE {
                    return Err(FixtureError::FillOverCap { path: file.path.clone() });
                }
            }
        }
        Ok(())
    }
}

/// An age before "now", `"30d"`, `"12h"` or `"90s"`, or after it with a leading `-` (`"-2d"`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Age {
    Before(Duration),
    After(Duration),
}

fn parse_age(text: &str) -> Option<Age> {
    let (after, text) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (count, per_unit) = match (text.strip_suffix('d'), text.strip_suffix('h'), text.strip_suffix('s')) {
        (Some(count), ..) => (count, 24 * 3600),
        (None, Some(count), _) => (count, 3600),
        (None, None, Some(count)) => (count, 1),
        (None, None, None) => return None,
    };
    // Digits only: `parse` would take a second sign.
    if count.is_empty() || !count.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let duration = Duration::from_secs(count.parse::<u64>().ok()?.checked_mul(per_unit)?);
    Some(if after { Age::After(duration) } else { Age::Before(duration) })
}

/// Refuses an `age` together with `modified`, and an age not of the form `"<n>d"`, `"<n>h"` or `"<n>s"`.
fn check_time(path: &Path, age: Option<&str>, modified: Option<u64>) -> Result<(), FixtureError> {
    if age.is_some() && modified.is_some() {
        return Err(FixtureError::AgeWithModified { path: path.to_path_buf() });
    }
    check_stated_times(path, &[age])
}

/// Refuses a stated time that is not an age.
fn check_stated_times(path: &Path, ages: &[Option<&str>]) -> Result<(), FixtureError> {
    for age in ages.iter().flatten() {
        parse_age(age).ok_or_else(|| FixtureError::InvalidAge { path: path.to_path_buf() })?;
    }
    Ok(())
}

/// The time `age` (`"10d"`, `"3h"`, `"90s"`, or `"-2d"` after) is counted back from `now`; `None` when it is not an
/// age.
pub fn time_at_age(age: &str, now: SystemTime) -> Option<SystemTime> {
    match parse_age(age)? {
        Age::Before(duration) => now.checked_sub(duration),
        Age::After(duration) => now.checked_add(duration),
    }
}

/// The time an entry is given: its age counted back from `now`, or its `modified` seconds; `None` for neither.
fn time_of(
    path: &Path,
    age: Option<&str>,
    modified: Option<u64>,
    now: SystemTime,
) -> Result<Option<SystemTime>, FixtureError> {
    Ok(match (age, modified) {
        (Some(age), _) => {
            Some(time_at_age(age, now).ok_or_else(|| FixtureError::InvalidAge { path: path.to_path_buf() })?)
        }
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
    match spec.fill.unwrap_or_default() {
        Fill::Sparse => file.set_len(spec.size).map_err(create_error)?,
        // Sized first, so every write lands inside the file: a file grown by appends can keep blocks the file
        // system reserved past its end, under load even after the sync, and a walk would read them. Synced, so
        // the blocks are allocated before a walk reads the file's size on disk.
        Fill::Data => file
            .set_len(spec.size)
            .and_then(|()| write_pattern(&mut file, spec.size))
            .and_then(|()| file.sync_all())
            .map_err(create_error)?,
        Fill::CachedirTag => {
            file.write_all(CACHEDIR_TAG_CONTENT).and_then(|()| file.sync_all()).map_err(create_error)?;
        }
    }
    if let Some(time) = modified {
        file.set_modified(time).map_err(create_error)?;
    }
    let written = file.metadata().and_then(|metadata| metadata.modified()).map_err(create_error)?;
    file.set_times(FileTimes::new().set_accessed(written)).map_err(create_error)
}

/// Sets the link's own modification and access time: opened with `O_SYMLINK`, the link itself, never its target.
fn set_link_time(path: &Path, time: SystemTime) -> std::io::Result<()> {
    let link = OpenOptions::new().read(true).custom_flags(libc::O_SYMLINK).open(path)?;
    link.set_times(FileTimes::new().set_modified(time).set_accessed(time))
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
            for age in ["30", "d", "--1d", "-", "+1d", "3w", "", "5д", "d5", "1ds", "1 d"] {
                for kind in ["file", "dir"] {
                    let error = parse(&format!("[[{kind}]]\npath = \"a\"\nage = \"{age}\"\n")).unwrap_err();

                    assert!(matches!(error, FixtureError::InvalidAge { .. }), "{kind} {age}");
                }
            }
        }

        #[test]
        fn an_unknown_owner_or_flag_is_refused() {
            for fact in ["owner = \"root\"", "owner = \"me\"", "flags = [\"locked\"]", "flags = \"user-lock\""] {
                for kind in ["file", "dir"] {
                    let error = parse(&format!("[[{kind}]]\npath = \"a\"\n{fact}\n")).unwrap_err();

                    assert!(matches!(error, FixtureError::Parse { .. }), "{kind} {fact}");
                }
            }
        }

        #[test]
        fn a_stated_time_that_is_not_an_age_is_refused() {
            for fact in ["changed = \"30\"", "accessed = \"1w\""] {
                let error = parse(&format!("[[file]]\npath = \"a\"\n{fact}\n")).unwrap_err();

                assert!(matches!(error, FixtureError::InvalidAge { .. }), "{fact}");
            }
        }

        #[test]
        fn a_links_time_that_is_not_an_age_is_refused() {
            for fact in ["age = \"30\"", "changed = \"1w\""] {
                let error = parse(&format!("[[symlink]]\npath = \"a\"\ntarget = \"b\"\n{fact}\n")).unwrap_err();

                assert!(matches!(error, FixtureError::InvalidAge { .. }), "{fact}");
            }
        }

        #[test]
        fn a_link_states_no_owner_flag_kind_or_access() {
            for fact in ["owner = \"system\"", "flags = [\"user-lock\"]", "kind = \"socket\"", "accessed = \"1d\""] {
                let error = parse(&format!("[[symlink]]\npath = \"a\"\ntarget = \"b\"\n{fact}\n")).unwrap_err();

                assert!(matches!(error, FixtureError::Parse { .. }), "{fact}");
            }
        }

        #[test]
        fn a_kind_on_a_folder_or_another_kind_is_refused() {
            for text in [
                "[[dir]]\npath = \"a\"\nkind = \"socket\"\n",
                "[[file]]\npath = \"a\"\nkind = \"device\"\n",
                "[[file]]\npath = \"a\"\nkind = \"folder\"\n",
            ] {
                assert!(matches!(parse(text).unwrap_err(), FixtureError::Parse { .. }), "{text}");
            }
        }

        #[test]
        fn a_kind_with_a_fill_is_refused() {
            for fill in ["data", "sparse"] {
                let text = format!("[[file]]\npath = \"a\"\nkind = \"pipe\"\nfill = \"{fill}\"\n");

                assert!(matches!(parse(&text).unwrap_err(), FixtureError::KindWithFill { .. }), "{fill}");
            }
        }

        #[test]
        fn a_cache_directory_tag_with_a_size_or_a_kind_is_refused() {
            let sized = parse("[[file]]\npath = \"CACHEDIR.TAG\"\nsize = 10\nfill = \"cachedir-tag\"\n").unwrap_err();
            let kind = parse("[[file]]\npath = \"CACHEDIR.TAG\"\nkind = \"socket\"\nfill = \"cachedir-tag\"\n");

            assert!(matches!(sized, FixtureError::TagWithSize { .. }), "{sized}");
            assert!(matches!(kind.unwrap_err(), FixtureError::KindWithFill { .. }));
        }

        #[test]
        fn an_entry_that_states_nothing_is_not_stated() {
            let tree = parse("[[file]]\npath = \"a\"\nage = \"3d\"\n[[dir]]\npath = \"b\"\nflags = []\n").unwrap();

            assert_eq!(tree.stated(SystemTime::UNIX_EPOCH).unwrap(), []);
        }

        #[test]
        fn a_folders_age_and_modified_together_are_refused() {
            let error = parse("[[dir]]\npath = \"a\"\nage = \"1d\"\nmodified = 5\n").unwrap_err();

            assert!(matches!(error, FixtureError::AgeWithModified { .. }));
        }

        #[test]
        fn an_apps_changed_that_is_not_an_age_is_refused() {
            let app = "[[app]]\npath = \"Applications/Tool.app\"\nbundle-id = \"com.example.Tool\"\nname = \"Tool\"\n";
            for changed in ["soon", "10", "d"] {
                let error = TreeDescription::from_toml_str(&format!("{app}changed = \"{changed}\"\n")).unwrap_err();

                assert!(matches!(error, FixtureError::InvalidAge { .. }), "{changed}: {error}");
            }
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
        use std::path::PathBuf;
        use std::time::{Duration, SystemTime};

        use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE, Timestamp};

        use crate::{
            FIXTURE_NOW, FixtureTree, Stated, StatedFlag, StatedKind, StatedOwner, TreeDescription, TreeReport,
        };

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
        fn an_age_in_seconds_or_after_now_is_counted_from_now() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"s\"\nage = \"90s\"\n[[file]]\npath = \"later\"\nage = \"-2d\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            let modified = |name: &str| std::fs::metadata(folder.path().join(name)).unwrap().modified().unwrap();
            assert_eq!(modified("s"), now() - Duration::from_secs(90));
            assert_eq!(modified("later"), now() + Duration::from_hours(48));
        }

        #[test]
        fn stated_facts_are_listed_and_the_entry_is_written_plain() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[dir]]\npath = \"d\"\nowner = \"other-user\"\nchanged = \"2h\"\n\
                        [[file]]\npath = \"d/sock\"\nkind = \"socket\"\nflags = [\"no-unlink\", \"user-lock\"]\n\
                        [[file]]\npath = \"d/old\"\nchanged = \"30d\"\naccessed = \"-1h\"\nowner = \"system\"\n";
            let tree = TreeDescription::from_toml_str(text).unwrap();

            tree.build_at(folder.path(), now()).unwrap();
            let stated = tree.stated(now()).unwrap();

            assert!(std::fs::symlink_metadata(folder.path().join("d/sock")).unwrap().is_file());
            let at = |back: Duration| Some(Timestamp::from_system_time(now() - back));
            assert_eq!(
                stated,
                [
                    (
                        PathBuf::from("d"),
                        Stated {
                            changed: at(Duration::from_hours(2)),
                            owner: Some(StatedOwner::OtherUser),
                            ..Stated::default()
                        }
                    ),
                    (
                        PathBuf::from("d/sock"),
                        Stated {
                            flags: [StatedFlag::NoUnlink, StatedFlag::UserLock].into(),
                            kind: Some(StatedKind::Socket),
                            ..Stated::default()
                        }
                    ),
                    (
                        PathBuf::from("d/old"),
                        Stated {
                            changed: at(Duration::from_hours(30 * 24)),
                            accessed: Some(Timestamp::from_system_time(now() + Duration::from_hours(1))),
                            owner: Some(StatedOwner::System),
                            ..Stated::default()
                        }
                    ),
                ]
            );
        }

        #[test]
        fn an_apps_changed_is_stated_for_its_bundle() {
            let text = "[[app]]\npath = \"Applications/Tool.app\"\nbundle-id = \"com.example.Tool\"\nname = \"Tool\"\n\
                        changed = \"10d\"\n\
                        [[app]]\npath = \"Applications/Other.app\"\nbundle-id = \"com.example.Other\"\nname = \"Other\"\n";
            let tree = TreeDescription::from_toml_str(text).unwrap();

            let changed = Some(Timestamp::from_system_time(now() - Duration::from_hours(10 * 24)));
            assert_eq!(
                tree.stated(now()).unwrap(),
                [(PathBuf::from("Applications/Tool.app"), Stated { changed, ..Stated::default() })]
            );
        }

        #[test]
        fn a_links_age_is_its_own_time_and_its_change_is_stated() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"target\"\nage = \"1d\"\n\
                        [[symlink]]\npath = \"link\"\ntarget = \"target\"\nage = \"30d\"\nchanged = \"30d\"\n";
            let tree = TreeDescription::from_toml_str(text).unwrap();

            tree.build_at(folder.path(), now()).unwrap();
            tree.build_at(folder.path(), now()).unwrap();

            let link = std::fs::symlink_metadata(folder.path().join("link")).unwrap();
            assert!(link.is_symlink());
            assert_eq!(link.modified().unwrap(), now() - Duration::from_hours(30 * 24));
            assert_eq!(link.accessed().unwrap(), now() - Duration::from_hours(30 * 24));
            let target = std::fs::metadata(folder.path().join("target")).unwrap();
            assert_eq!(target.modified().unwrap(), now() - Duration::from_hours(24));
            let changed = Some(Timestamp::from_system_time(now() - Duration::from_hours(30 * 24)));
            assert_eq!(tree.stated(now()).unwrap(), [(PathBuf::from("link"), Stated { changed, ..Stated::default() })]);
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
        fn filled_files_take_exactly_their_blocks() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"a\"\nsize = 1000000\nfill = \"data\"\n\
                        [[file]]\npath = \"b\"\nsize = 52428800\nfill = \"data\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            // Whole 4 KiB blocks, none past the end: what the sandbox goldens hold.
            for (name, size, blocks) in [("a", 1_000_000, 1_003_520), ("b", 52_428_800, 52_428_800)] {
                let metadata = std::fs::metadata(folder.path().join(name)).unwrap();
                assert_eq!(metadata.len(), size, "{name}");
                assert_eq!(metadata.blocks() * 512, blocks, "{name}");
            }
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
        fn a_cache_directory_tag_holds_a_valid_tag() {
            let folder = tempfile::tempdir().unwrap();
            let text = "[[file]]\npath = \"a/CACHEDIR.TAG\"\nfill = \"cachedir-tag\"\n\
                        [[file]]\npath = \"b/CACHEDIR.TAG\"\nsize = 0\nfill = \"cachedir-tag\"\n";

            TreeDescription::from_toml_str(text).unwrap().build_at(folder.path(), now()).unwrap();

            for tag in ["a/CACHEDIR.TAG", "b/CACHEDIR.TAG"] {
                let content = std::fs::read(folder.path().join(tag)).unwrap();
                assert!(fresher_core::is_valid_cachedir_tag(&content), "{tag}");
                assert_eq!(content, crate::tree_description::CACHEDIR_TAG_CONTENT, "{tag}");
            }
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
