use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use fresher_core::{Protection, SANDBOX_MARKER, SANDBOX_SIGNATURE, SettingsStore as _, StoredSettings};
use fresher_store::{DataFolder, FileSettingsStore};
use fresher_testing::{FixtureError, TreeDescription};

use crate::expected::checklist;
use crate::layout::{CHECKLIST, DATA_FOLDER, PARTS, WORLD, canonical_folder, holds_marker};
use crate::{CaseFile, DemoAgreement, Sandbox, SandboxError, World, merged_settings};

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
/// the running apps the cases declare and the facts their files state). A demo world is generated at its own `now`
/// (`Demo::now`), which the caller passes.
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
    generate_leaving_out(files, &[], into, now, real_protection, user)
}

/// A case file `all` leaves out whole, because it writes a path an earlier file wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeftOutFile {
    /// The feature of the file that is left out.
    pub feature: String,
    /// The feature of the earlier file that wrote the path.
    pub first: String,
    /// The path both write, relative to the sandbox.
    pub path: PathBuf,
}

/// Splits `files` (in the order `all` writes them) into the files that fit one sandbox and the files left out whole,
/// each one writing a path an earlier kept file wrote. The kept files write no path twice.
///
/// # Errors
///
/// A [`SandboxError::Case`] when a file does not make one tree.
pub fn without_clashes(files: Vec<CaseFile>) -> Result<(Vec<CaseFile>, Vec<LeftOutFile>), SandboxError> {
    let mut written: HashMap<PathBuf, String> = HashMap::new();
    let (mut kept, mut left_out) = (Vec::new(), Vec::new());
    for file in files {
        let own = file.tree().map_err(|source| SandboxError::Case { source })?;
        let clash = own.paths().filter_map(|path| written.get(path).map(|first| (path, first))).min();
        if let Some((path, first)) = clash {
            left_out.push(LeftOutFile { feature: file.feature().into(), first: first.clone(), path: path.into() });
            continue;
        }
        written.extend(own.paths().map(|path| (path.to_path_buf(), file.feature().to_owned())));
        kept.push(file);
    }
    Ok((kept, left_out))
}

/// [`generate`] for case files [`without_clashes`] left some out of: `EXPECTED.md` names each left-out file.
///
/// # Errors
///
/// As [`generate`].
pub fn generate_leaving_out(
    files: &[CaseFile],
    left_out: &[LeftOutFile],
    into: &Path,
    now: SystemTime,
    real_protection: &Protection,
    user: u32,
) -> Result<Generated, SandboxError> {
    let tree = merged_tree(files)?;
    let mut settings = merged_settings(files).map_err(|source| SandboxError::Case { source })?;
    settings.extend(demo_acceptance(files));
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
    write_plain(&sandbox.checklist(), &checklist(files, left_out, sandbox.path(), now))?;
    write_plain(&sandbox.world(), &World::of(files, now)?.text()?)?;
    write_settings(&sandbox, settings)?;
    let strangers = strangers(sandbox.path(), &own)?;
    Ok(Generated { sandbox, strangers })
}

/// The agreement's acceptance a demo world with `agreement = "accepted"` starts with, at the world's clock; nothing for
/// any other file (case files cannot write these keys: `CaseError::AgreementSetting`).
fn demo_acceptance(files: &[CaseFile]) -> Vec<(String, toml::Value)> {
    let Some(demo) = files.iter().filter_map(CaseFile::demo).find(|demo| demo.agreement() == DemoAgreement::Accepted)
    else {
        return Vec::new();
    };
    let at = demo.now().duration_since(SystemTime::UNIX_EPOCH).map_or(0, |since| since.as_secs());
    vec![
        (
            fresher_core::AGREEMENT_VERSION_KEY.to_owned(),
            toml::Value::Integer(i64::from(fresher_core::AGREEMENT_VERSION)),
        ),
        (fresher_core::AGREEMENT_ACCEPTED_AT.to_owned(), toml::Value::Integer(i64::try_from(at).unwrap_or(i64::MAX))),
    ]
}

/// Writes the case files' `[settings]` into the settings file of the sandbox's data folder, in the app's own format,
/// before the app starts. Nothing is written when no file has any.
fn write_settings(sandbox: &Sandbox, settings: BTreeMap<String, toml::Value>) -> Result<(), SandboxError> {
    if settings.is_empty() {
        return Ok(());
    }
    let values = settings.into_iter().filter_map(|(key, value)| Some((key, stored(&value)?))).collect();
    let refused = |reason: String| SandboxError::Settings { reason };
    let folder = DataFolder::open(sandbox.data_folder()).map_err(|error| refused(error.to_string()))?;
    FileSettingsStore::new(folder).save(&StoredSettings { values }).map_err(|error| refused(error.to_string()))
}

/// A setting's value as the settings file keeps it: a plain JSON value; `None` for a kind no case file may write.
fn stored(value: &toml::Value) -> Option<serde_json::Value> {
    match value {
        toml::Value::Boolean(on) => Some((*on).into()),
        toml::Value::Integer(number) => Some((*number).into()),
        toml::Value::String(text) => Some(text.as_str().into()),
        _ => None,
    }
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
path = "home/bare/Sample app cache/blob.bin"
size = 1048576
age = "30d"

[[case]]
name = "a trial cache next to a marker goes to the Trash"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "home/work/Sample app cache/blob.bin"
size = 4096
fill = "data"
[case.expect]
item = "home/work/Sample app cache"
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
        use crate::{CaseFile, SandboxError, generate, generate_leaving_out, without_clashes};

        #[test]
        fn a_setting_two_files_set_differently_is_refused_before_anything_is_written() {
            let set = |feature: &str, value: u8| {
                let text = format!("feature = \"{feature}\"\n[settings]\n\"demo.count\" = {value}\n");
                CaseFile::parse(&text).unwrap()
            };
            let folder = tempfile::tempdir().unwrap();

            let error = run(&[set("one", 1), set("two", 2)], &folder.path().join("sandbox")).unwrap_err();

            assert!(matches!(error, SandboxError::Case { .. }), "{error}");
            assert!(!folder.path().join("sandbox").exists());
        }

        #[test]
        fn a_demo_world_without_agreement_writes_no_acceptance() {
            let folder = tempfile::tempdir().unwrap();
            let into = folder.path().join("sandbox");
            let header = "feature = \"world\"\n[demo]\nnow = 1795000000\nshows = [\"dev-caches.npm\"]\n";

            run(&[CaseFile::parse(header).unwrap()], &into).unwrap();

            assert!(!into.join(".fresher-data").join("settings.json").exists());
        }

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
            for own in ["home/work/Sample app cache/blob.bin", "EXPECTED.md", ".fresher-sandbox", "world.toml"] {
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
            let stranger = folder.path().join("home/bare/Sample app cache/stranger.txt");
            fs::write(&stranger, "mine").unwrap();
            fs::create_dir_all(folder.path().join("home/.Trash/Sample app cache")).unwrap();
            fs::create_dir_all(folder.path().join(".fresher-data/cache")).unwrap();
            let own = folder.path().join("home/work/Sample app cache/blob.bin");
            fs::write(&own, "changed").unwrap();

            let strangers = run(&[trial()], folder.path()).unwrap();

            let expected: Vec<PathBuf> =
                ["home/.Trash/Sample app cache", "home/bare/Sample app cache/stranger.txt"].map(PathBuf::from).into();
            assert_eq!(strangers, expected);
            assert_eq!(fs::read_to_string(&stranger).unwrap(), "mine");
            assert_eq!(fs::metadata(&own).unwrap().len(), 4096);
        }

        #[test]
        fn a_file_that_clashes_with_an_earlier_one_is_left_out_whole_and_named() {
            let other = CaseFile::parse(&TRIAL.replace("feature = \"trial\"", "feature = \"other\"")).unwrap();
            let folder = tempfile::tempdir().unwrap();

            let (kept, left_out) = without_clashes(vec![trial(), other]).unwrap();
            let strangers = generate_leaving_out(
                &kept,
                &left_out,
                folder.path(),
                now(),
                &Protection::default(),
                user(std::env::temp_dir().as_path()),
            )
            .unwrap()
            .strangers;

            let features: Vec<&str> = kept.iter().map(CaseFile::feature).collect();
            assert_eq!(features, ["trial"]);
            assert_eq!(left_out.len(), 1);
            assert_eq!((left_out[0].feature.as_str(), left_out[0].first.as_str()), ("other", "trial"));
            assert_eq!(strangers.len(), 0);
            let checklist = fs::read_to_string(folder.path().join("EXPECTED.md")).unwrap();
            assert!(checklist.contains("`other` is not in this sandbox"), "{checklist}");
            assert!(checklist.contains("npm run sandbox -- other"), "{checklist}");
            assert!(!checklist.contains("## other"), "{checklist}");
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

        use fresher_core::{
            KnownFolder, KnownFolders, Protection, SANDBOX_MARKER, SANDBOX_SIGNATURE, SettingsStore as _,
        };
        use fresher_store::{DataFolder, FileSettingsStore};
        use fresher_testing::{EntryKind, TreeReport};
        use serde_json::json;

        use super::{TRIAL, apps, run, trial, user};
        use crate::{Sandbox, World, generate, without_clashes};

        #[test]
        fn files_that_write_different_paths_are_all_kept() {
            let (kept, left_out) = without_clashes(vec![trial(), apps()]).unwrap();

            assert_eq!(kept.len(), 2);
            assert_eq!(left_out.len(), 0);
        }

        #[test]
        fn settings_are_written_in_the_format_the_stores_reads() {
            let folder = tempfile::tempdir().unwrap();
            let into = folder.path().join("sandbox");
            let text =
                format!("{TRIAL}[settings]\n\"demo.count\" = 3\n\"demo.deep\" = true\n\"demo.mode\" = \"few\"\n");
            let file = crate::CaseFile::parse(&text).unwrap();

            run(&[file], &into).unwrap();

            let data = DataFolder::open(fs::canonicalize(&into).unwrap().join(".fresher-data")).unwrap();
            let stored = FileSettingsStore::new(data).load().unwrap();
            let expected = json!({ "demo.count": 3, "demo.deep": true, "demo.mode": "few" });
            assert_eq!(serde_json::to_value(stored).unwrap(), expected);
        }

        #[test]
        fn a_demo_world_that_says_accepted_starts_with_the_agreement_accepted_at_its_clock() {
            let folder = tempfile::tempdir().unwrap();
            let into = folder.path().join("sandbox");
            let header = "feature = \"world\"\n[demo]\nnow = 1795000000\nshows = [\"dev-caches.npm\"]\nagreement = \"accepted\"\n";

            run(&[crate::CaseFile::parse(header).unwrap()], &into).unwrap();

            let data = DataFolder::open(fs::canonicalize(&into).unwrap().join(".fresher-data")).unwrap();
            let stored = FileSettingsStore::new(data).load().unwrap();
            let expected = json!({
                fresher_core::AGREEMENT_VERSION_KEY: fresher_core::AGREEMENT_VERSION,
                fresher_core::AGREEMENT_ACCEPTED_AT: 1_795_000_000,
            });
            assert_eq!(serde_json::to_value(stored).unwrap(), expected);
        }

        #[test]
        fn a_case_file_without_settings_writes_no_settings_file() {
            let folder = tempfile::tempdir().unwrap();
            let into = folder.path().join("sandbox");

            run(&[trial()], &into).unwrap();

            assert!(!into.join(".fresher-data").join("settings.json").exists());
        }

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
                    ("home/bare/Sample app cache", EntryKind::Dir),
                    ("home/bare/Sample app cache/blob.bin", EntryKind::File),
                    ("home/work", EntryKind::Dir),
                    ("home/work/.fresher-sandbox", EntryKind::File),
                    ("home/work/Sample app cache", EntryKind::Dir),
                    ("home/work/Sample app cache/blob.bin", EntryKind::File),
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
            let blob = folder.path().join("sandbox/trial/home/bare/Sample app cache/blob.bin");
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
        fn a_demo_world_is_generated_at_its_own_now() {
            let world = crate::CaseFile::parse(
                r#"
feature = "developer-mac"
[demo]
now = 1795000000
shows = ["trial.caches"]

[[case]]
name = "Xcode's build products"
kind = "scenery"
[[case.file]]
path = "home/Library/Developer/Xcode/DerivedData/Atlas-bxkqzr/Atlas.bin"
size = 4096
age = "40d"

[[case]]
name = "a browser's cache"
kind = "scenery"
[[case.file]]
path = "home/Library/Caches/com.example.Browser/data.bin"
size = 1024
age = "3d"
"#,
            )
            .unwrap();
            let folder = tempfile::tempdir().unwrap();
            let now = world.demo().unwrap().now();

            generate(&[world], folder.path(), now, &Protection::default(), user(folder.path())).unwrap();

            let day = std::time::Duration::from_hours(24);
            for (path, age) in [
                ("home/Library/Developer/Xcode/DerivedData/Atlas-bxkqzr/Atlas.bin", 40 * day),
                ("home/Library/Caches/com.example.Browser/data.bin", 3 * day),
            ] {
                assert_eq!(fs::metadata(folder.path().join(path)).unwrap().modified().unwrap(), now - age, "{path}");
            }
            let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();
            assert_eq!(World::read(&sandbox).unwrap().now(), Some(now));
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
