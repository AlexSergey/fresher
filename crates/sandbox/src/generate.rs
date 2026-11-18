use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use fresher_core::{Protection, SANDBOX_MARKER, SANDBOX_SIGNATURE};
use fresher_testing::{FixtureError, TreeDescription};

use crate::expected::checklist;
use crate::layout::{CHECKLIST, DATA_FOLDER, PARTS, WORLD, canonical_folder, holds_marker};
use crate::{CaseFile, Sandbox, SandboxError, World};

/// What a run of the generator made: the sandbox and the entries in it the generator did not make.
#[derive(Debug)]
pub struct Generated {
    /// The sandbox written to, at its canonical path.
    pub sandbox: Sandbox,
    /// The entries the generator did not make, relative to the sandbox, sorted; left untouched.
    pub strangers: Vec<PathBuf>,
}

/// Writes the case files `files` into the sandbox folder `into`: the marker (the signature, then a line naming the
/// case files), the layout, every case's tree (its ages counted back from `now`), `EXPECTED.md` and `world.toml` (`now`,
/// the running apps the cases declare and the facts their files state).
///
/// Refuses a folder that `real_protection` (the real Mac's list) refuses, that is a link or reached through one inside
/// the sandbox, or that is not empty and holds no valid marker of `user`. On a sandbox it made before it tops up:
/// writes what is missing and resets its own files. It removes nothing: it returns the entries it did not make,
/// relative to the sandbox, untouched (the app's data folder aside). It never rewrites a file of its own that has a
/// name it did not make (a hard link from elsewhere): such a sandbox is refused before anything is written.
pub fn generate(
    files: &[CaseFile],
    into: &Path,
    now: SystemTime,
    real_protection: &Protection,
    user: u32,
) -> Result<Generated, SandboxError> {
    let tree = merged_tree(files)?;
    let sandbox = Sandbox::at(target(into, real_protection, user)?);
    let own = own_paths(&tree);
    refuse_shared_files(sandbox.path(), &own)?;
    write_marker(&sandbox, files)?;
    for part in PARTS {
        make_folder(sandbox.path(), Path::new(part))?;
    }
    tree.build_at(sandbox.path(), now).map_err(|error| match error {
        FixtureError::PathThroughLink { path } => SandboxError::ThroughLink { path },
        source => SandboxError::Tree { source },
    })?;
    write_plain(&sandbox.checklist(), &checklist(files, sandbox.path(), now))?;
    write_plain(&sandbox.world(), &World::of(files, now)?.text()?)?;
    let strangers = strangers(sandbox.path(), &own)?;
    Ok(Generated { sandbox, strangers })
}

/// Refuses a regular file among the generator's own paths that has more names than the generator gives it: rewriting
/// it would change the file behind a hard link it did not make, which may lie outside the sandbox.
fn refuse_shared_files(root: &Path, own: &HashSet<PathBuf>) -> Result<(), SandboxError> {
    let mut names: HashMap<(u64, u64), (u64, u64, PathBuf)> = HashMap::new();
    for relative in own {
        let path = root.join(relative);
        let Ok(metadata) = fs::symlink_metadata(&path) else { continue };
        if metadata.is_file() {
            let entry = names.entry((metadata.dev(), metadata.ino())).or_insert((metadata.nlink(), 0, path));
            entry.1 += 1;
        }
    }
    match names.into_values().find(|(links, own, _)| links > own) {
        Some((_, _, path)) => Err(SandboxError::SharedFile { path }),
        None => Ok(()),
    }
}

/// Every case file's tree as one; a path two files write is refused.
fn merged_tree(files: &[CaseFile]) -> Result<TreeDescription, SandboxError> {
    let mut written: HashMap<PathBuf, &str> = HashMap::new();
    let mut tree = TreeDescription::default();
    for file in files {
        let own = file.tree().map_err(|source| SandboxError::Case { source })?;
        for path in own.paths() {
            if let Some(first) = written.insert(path.to_path_buf(), file.feature()) {
                return Err(SandboxError::PathWrittenTwice {
                    path: path.to_path_buf(),
                    first: first.into(),
                    second: file.feature().into(),
                });
            }
        }
        tree = tree.merge(own).map_err(|source| SandboxError::Tree { source })?;
    }
    Ok(tree)
}

/// The canonical sandbox folder, made if missing: no link itself, not protected on the real Mac, and empty or holding
/// a valid marker.
fn target(into: &Path, real_protection: &Protection, user: u32) -> Result<PathBuf, SandboxError> {
    let (existing, missing) = split_existing(into)?;
    // The folder itself must be no link; a link above it is resolved, and the real place is what is checked.
    let above = if missing.is_empty() {
        canonical_folder(existing)?
    } else {
        fs::canonicalize(existing).map_err(|source| SandboxError::Io { path: existing.into(), source })?
    };
    let folder = missing.iter().fold(above, |path, name| path.join(name));
    if real_protection.check(&folder).is_err() {
        return Err(SandboxError::Protected { folder });
    }
    if !missing.is_empty() {
        fs::create_dir_all(&folder).map_err(|source| SandboxError::Io { path: folder.clone(), source })?;
        return Ok(folder);
    }
    let mut entries = fs::read_dir(&folder).map_err(|source| SandboxError::Io { path: folder.clone(), source })?;
    if entries.next().is_some() && !holds_marker(&folder, user) {
        return Err(SandboxError::NotASandbox { folder });
    }
    Ok(folder)
}

/// The nearest existing ancestor of `path` (`path` itself when it exists) and the names below it that do not exist.
fn split_existing(path: &Path) -> Result<(&Path, Vec<&std::ffi::OsStr>), SandboxError> {
    let mut missing = Vec::new();
    let mut existing = path;
    while fs::symlink_metadata(existing).is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        let (Some(parent), Some(Component::Normal(name))) = (existing.parent(), existing.components().next_back())
        else {
            return Err(SandboxError::NotPlain { path: path.into() });
        };
        missing.push(name);
        existing = if parent.as_os_str().is_empty() { Path::new(".") } else { parent };
    }
    missing.reverse();
    Ok((existing, missing))
}

/// The marker: the signature, then a line naming the case files.
fn write_marker(sandbox: &Sandbox, files: &[CaseFile]) -> Result<(), SandboxError> {
    let names: Vec<String> = files.iter().map(|file| format!("{}.toml", file.feature())).collect();
    write_plain(&sandbox.marker(), &format!("{SANDBOX_SIGNATURE}\ncase files: {}\n", names.join(", ")))
}

/// Writes `text` into `path`, which must not be a link.
fn write_plain(path: &Path, text: &str) -> Result<(), SandboxError> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_symlink()) {
        return Err(SandboxError::ThroughLink { path: path.into() });
    }
    fs::write(path, text).map_err(|source| SandboxError::Io { path: path.into(), source })
}

/// Makes the folder `relative` below `root`, through no link.
fn make_folder(root: &Path, relative: &Path) -> Result<(), SandboxError> {
    let mut path = root.to_path_buf();
    for name in relative {
        path.push(name);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_symlink() => return Err(SandboxError::ThroughLink { path }),
            Ok(metadata) if !metadata.is_dir() => return Err(SandboxError::NotAFolder { path }),
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {
                fs::create_dir(&path).map_err(|source| SandboxError::Io { path: path.clone(), source })?;
            }
            Err(source) => return Err(SandboxError::Io { path, source }),
        }
    }
    Ok(())
}

/// What the generator writes, relative to the sandbox, with every folder above it.
fn own_paths(tree: &TreeDescription) -> HashSet<PathBuf> {
    let markers = tree.markers().map(|folder| folder.join(SANDBOX_MARKER));
    let bundles = tree.apps().iter().flat_map(fresher_testing::AppBundle::written);
    let written = PARTS.iter().map(PathBuf::from).chain([SANDBOX_MARKER, CHECKLIST, WORLD].map(PathBuf::from));
    let mut own = HashSet::new();
    for path in written.chain(tree.paths().map(Path::to_path_buf)).chain(markers).chain(bundles) {
        own.extend(path.ancestors().filter(|path| !path.as_os_str().is_empty()).map(Path::to_path_buf));
    }
    own
}

/// The entries below `root` the generator did not make, sorted; a stranger folder is listed, not entered.
fn strangers(root: &Path, own: &HashSet<PathBuf>) -> Result<Vec<PathBuf>, SandboxError> {
    let mut found = Vec::new();
    let mut folders = vec![PathBuf::new()];
    while let Some(relative) = folders.pop() {
        let folder = root.join(&relative);
        let entries = fs::read_dir(&folder).map_err(|source| SandboxError::Io { path: folder.clone(), source })?;
        for entry in entries {
            let entry = entry.map_err(|source| SandboxError::Io { path: folder.clone(), source })?;
            let path = relative.join(entry.file_name());
            let is_folder = entry.file_type().is_ok_and(|kind| kind.is_dir());
            if !own.contains(&path) {
                found.push(path);
            } else if is_folder && path != Path::new(DATA_FOLDER) {
                folders.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, SystemTime};

    use fresher_core::Protection;
    use fresher_testing::FIXTURE_NOW;

    use crate::{CaseFile, SandboxError, generate};

    const TRIAL: &str = r#"
feature = "trial"
module = "trial"

[[case]]
name = "a trial folder with no marker next to it is never offered"
kind = "negative"
[[case.file]]
path = "home/bare/fresher-trial-cache/blob.bin"
size = 1048576
age = "30d"

[[case]]
name = "a trial cache next to a marker goes to the Trash"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "home/work/fresher-trial-cache/blob.bin"
size = 4096
fill = "data"
[case.expect]
item = "home/work/fresher-trial-cache"
category = "trial.caches"
danger = "low"
after-cleanup = "in-trash"
"#;

    fn now() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW)
    }

    fn trial() -> CaseFile {
        CaseFile::parse(TRIAL).unwrap()
    }

    /// A case file that installs two apps.
    fn apps() -> CaseFile {
        CaseFile::parse(
            r#"
feature = "apps"

[[case]]
name = "installed apps"
kind = "negative"
[[case.app]]
path = "Applications/Tool.app"
bundle-id = "com.example.Tool"
name = "Tool"
version = "1.0"
[[case.app]]
path = "home/Applications/Mine.app"
bundle-id = "com.example.Mine"
name = "Mine"
"#,
        )
        .unwrap()
    }

    fn user(folder: &Path) -> u32 {
        fs::metadata(folder).unwrap().uid()
    }

    /// Generates `files` into `into` with no protection.
    fn run(files: &[CaseFile], into: &Path) -> Result<Vec<PathBuf>, SandboxError> {
        generate(files, into, now(), &Protection::default(), user(std::env::temp_dir().as_path()))
            .map(|generated| generated.strangers)
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::PathBuf;

        use fresher_core::{Protection, SANDBOX_MARKER};

        use super::{TRIAL, now, run, trial, user};
        use crate::{CaseFile, SandboxError, generate};

        #[test]
        fn a_non_empty_folder_without_a_marker_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            fs::write(folder.path().join("notes.txt"), "mine").unwrap();

            let error = run(&[trial()], folder.path()).unwrap_err();

            assert!(matches!(error, SandboxError::NotASandbox { .. }));
            let names: Vec<_> = fs::read_dir(folder.path()).unwrap().map(|entry| entry.unwrap().file_name()).collect();
            assert_eq!(names, ["notes.txt"]);
        }

        #[test]
        fn a_folder_with_an_unsigned_marker_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            fs::write(folder.path().join(SANDBOX_MARKER), "not a signature\n").unwrap();

            let error = run(&[trial()], folder.path()).unwrap_err();

            assert!(matches!(error, SandboxError::NotASandbox { .. }));
            assert!(!folder.path().join("home").exists());
        }

        #[test]
        fn a_protected_folder_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            let target = fs::canonicalize(folder.path()).unwrap();
            let whole = Protection::new(vec![], vec![target.clone()], vec![]).unwrap();

            for into in [target.clone(), target.parent().unwrap().to_path_buf()] {
                let error = generate(&[trial()], &into, now(), &whole, user(&target)).unwrap_err();

                assert!(matches!(error, SandboxError::Protected { .. }), "{}", into.display());
            }
            assert!(!target.join("home").exists());
        }

        #[test]
        fn a_target_inside_a_never_place_is_refused_before_anything_is_made() {
            let home = tempfile::tempdir().unwrap();
            let documents = fs::canonicalize(home.path()).unwrap().join("Documents");
            fs::create_dir(&documents).unwrap();
            let never = Protection::new(vec![documents.clone()], vec![], vec![]).unwrap();

            let into = documents.join("x");
            let error = generate(&[trial()], &into, now(), &never, user(home.path())).unwrap_err();

            assert!(matches!(error, SandboxError::Protected { .. }));
            assert!(!into.exists());
        }

        #[test]
        fn a_dot_dot_below_the_existing_folders_is_refused() {
            let folder = tempfile::tempdir().unwrap();

            let error = run(&[trial()], &folder.path().join("missing/../sandbox")).unwrap_err();

            assert!(matches!(error, SandboxError::NotPlain { .. }));
            assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
        }

        #[test]
        fn a_file_with_a_hard_link_from_outside_is_never_rewritten() {
            for own in ["home/work/fresher-trial-cache/blob.bin", "EXPECTED.md", ".fresher-sandbox", "world.toml"] {
                let folder = tempfile::tempdir().unwrap();
                let outside = tempfile::tempdir().unwrap();
                run(&[trial()], folder.path()).unwrap();
                let linked = outside.path().join("linked");
                fs::hard_link(folder.path().join(own), &linked).unwrap();
                let before = fs::read(&linked).unwrap();

                let error = run(&[trial()], folder.path()).unwrap_err();

                assert!(matches!(&error, SandboxError::SharedFile { path } if path.ends_with(own)), "{own}");
                assert_eq!(fs::read(&linked).unwrap(), before, "{own}");
            }
        }

        #[test]
        fn a_target_through_a_link_is_refused() {
            let real = tempfile::tempdir().unwrap();
            let outer = tempfile::tempdir().unwrap();
            let link = outer.path().join("sandbox");
            symlink(real.path(), &link).unwrap();

            let error = run(&[trial()], &link).unwrap_err();
            assert!(matches!(error, SandboxError::ThroughLink { .. }));

            let elsewhere = tempfile::tempdir().unwrap();
            let folder = tempfile::tempdir().unwrap();
            let marked = folder.path().join("sandbox");
            run(&[trial()], &marked).unwrap();
            let trash = marked.join("home/.Trash");
            let renamed = marked.join("trash-was-here");
            fs::rename(&trash, &renamed).unwrap();
            symlink(elsewhere.path(), &trash).unwrap();

            let error = run(&[trial()], &marked).unwrap_err();

            assert!(matches!(error, SandboxError::ThroughLink { path } if path.ends_with("home/.Trash")));
            assert_eq!(fs::read_dir(elsewhere.path()).unwrap().count(), 0);
        }

        #[test]
        fn a_second_run_removes_nothing() {
            let folder = tempfile::tempdir().unwrap();
            run(&[trial()], folder.path()).unwrap();
            let stranger = folder.path().join("home/bare/fresher-trial-cache/stranger.txt");
            fs::write(&stranger, "mine").unwrap();
            fs::create_dir_all(folder.path().join("home/.Trash/fresher-trial-cache")).unwrap();
            fs::create_dir_all(folder.path().join(".fresher-data/cache")).unwrap();
            let own = folder.path().join("home/work/fresher-trial-cache/blob.bin");
            fs::write(&own, "changed").unwrap();

            let strangers = run(&[trial()], folder.path()).unwrap();

            let expected: Vec<PathBuf> =
                ["home/.Trash/fresher-trial-cache", "home/bare/fresher-trial-cache/stranger.txt"]
                    .map(PathBuf::from)
                    .into();
            assert_eq!(strangers, expected);
            assert_eq!(fs::read_to_string(&stranger).unwrap(), "mine");
            assert_eq!(fs::metadata(&own).unwrap().len(), 4096);
        }

        #[test]
        fn all_refuses_two_features_writing_one_path() {
            let other = CaseFile::parse(&TRIAL.replace("feature = \"trial\"", "feature = \"other\"")).unwrap();
            let folder = tempfile::tempdir().unwrap();

            let error = run(&[trial(), other], folder.path()).unwrap_err();

            assert!(matches!(error, SandboxError::PathWrittenTwice { first, second, .. }
                if first == "trial" && second == "other"));
            assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 0);
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::MetadataExt as _;

        use fresher_core::{KnownFolder, KnownFolders, SANDBOX_MARKER, SANDBOX_SIGNATURE};
        use fresher_testing::{EntryKind, TreeReport};

        use super::{apps, run, trial, user};
        use crate::Sandbox;

        #[test]
        fn a_case_file_becomes_its_sandbox() {
            let folder = tempfile::tempdir().unwrap();

            let strangers = run(&[trial()], &folder.path().join("sandbox/trial")).unwrap();

            assert_eq!(strangers, Vec::<std::path::PathBuf>::new());
            let report = TreeReport::of(&folder.path().join("sandbox/trial")).unwrap();
            let entries: Vec<_> = report.entries.iter().map(|entry| (entry.path.as_str(), entry.kind)).collect();
            assert_eq!(
                entries,
                [
                    (".fresher-data", EntryKind::Dir),
                    (".fresher-sandbox", EntryKind::File),
                    ("Applications", EntryKind::Dir),
                    ("EXPECTED.md", EntryKind::File),
                    ("System", EntryKind::Dir),
                    ("System/Applications", EntryKind::Dir),
                    ("home", EntryKind::Dir),
                    ("home/.Trash", EntryKind::Dir),
                    ("home/bare", EntryKind::Dir),
                    ("home/bare/fresher-trial-cache", EntryKind::Dir),
                    ("home/bare/fresher-trial-cache/blob.bin", EntryKind::File),
                    ("home/work", EntryKind::Dir),
                    ("home/work/.fresher-sandbox", EntryKind::File),
                    ("home/work/fresher-trial-cache", EntryKind::Dir),
                    ("home/work/fresher-trial-cache/blob.bin", EntryKind::File),
                    ("private", EntryKind::Dir),
                    ("private/tmp", EntryKind::Dir),
                    ("private/var", EntryKind::Dir),
                    ("private/var/folders", EntryKind::Dir),
                    ("private/var/folders/xx", EntryKind::Dir),
                    ("private/var/folders/xx/sandbox", EntryKind::Dir),
                    ("private/var/folders/xx/sandbox/0", EntryKind::Dir),
                    ("private/var/folders/xx/sandbox/C", EntryKind::Dir),
                    ("private/var/folders/xx/sandbox/T", EntryKind::Dir),
                    ("private/var/folders/xx/sandbox/X", EntryKind::Dir),
                    ("world.toml", EntryKind::File),
                ]
            );
            let marker = fs::read_to_string(folder.path().join("sandbox/trial").join(SANDBOX_MARKER)).unwrap();
            assert_eq!(marker, format!("{SANDBOX_SIGNATURE}\ncase files: trial.toml\n"));
            let blob = folder.path().join("sandbox/trial/home/bare/fresher-trial-cache/blob.bin");
            assert_eq!(fs::metadata(&blob).unwrap().len(), 1_048_576);
            assert_eq!(
                fs::metadata(&blob).unwrap().modified().unwrap(),
                super::now() - std::time::Duration::from_hours(30 * 24)
            );
        }

        #[test]
        fn apps_are_written_as_bundles_and_rewritten_on_a_second_run() {
            let folder = tempfile::tempdir().unwrap();
            run(&[apps()], folder.path()).unwrap();
            let plist = folder.path().join("Applications/Tool.app/Contents/Info.plist");
            let first = fs::read_to_string(&plist).unwrap();
            fs::write(&plist, "changed").unwrap();

            let strangers = run(&[apps()], folder.path()).unwrap();

            assert_eq!(strangers, Vec::<std::path::PathBuf>::new());
            assert_eq!(fs::read_to_string(&plist).unwrap(), first);
            assert!(first.contains("<string>com.example.Tool</string>"));
            assert!(folder.path().join("home/Applications/Mine.app/Contents/MacOS").is_dir());
        }

        #[test]
        fn a_missing_folder_below_a_link_is_made_at_the_real_place() {
            let real = tempfile::tempdir().unwrap();
            let outer = tempfile::tempdir().unwrap();
            std::os::unix::fs::symlink(real.path(), outer.path().join("link")).unwrap();

            run(&[trial()], &outer.path().join("link/sandbox")).unwrap();

            assert!(real.path().join("sandbox").join(SANDBOX_MARKER).is_file());
        }

        #[test]
        fn a_generated_sandbox_opens() {
            let folder = tempfile::tempdir().unwrap();
            run(&[trial()], folder.path()).unwrap();

            let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();

            let home = fs::canonicalize(folder.path()).unwrap().join("home");
            assert_eq!(sandbox.known_folders().locate(KnownFolder::Home), Some(home));
            assert!(fs::metadata(sandbox.checklist()).unwrap().size() > 0);
        }
    }
}
