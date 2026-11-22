//! `fresher-sandbox demo <scenario> [--into <dir>]`: the world of a demo scenario, written into a new folder with no
//! user name in its path (docs/plans/038-demo-recordings/00-design.md, "Where it is generated"). It never removes or
//! overwrites anything: every run takes a folder that does not exist yet; old demo folders stay until the system clears
//! `/private/tmp`.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use fresher_core::Protection;
use fresher_sandbox::{CaseFile, Demo, Scenario, generate, is_plain_name};

/// The folder of the demo folders: `<scenario>-<n>` in it for each run.
pub(crate) const DEMO_FOLDER: &str = "/private/tmp/fresher-demo";
/// The most demo folders of one scenario the root may hold before `demo` asks for it to be cleared.
const MOST_RUNS: u32 = 9_999;

/// What `demo` wrote: the scenario's world and the folder it is in, canonical.
#[derive(Debug)]
pub(crate) struct DemoWritten {
    pub(crate) world: String,
    pub(crate) folder: PathBuf,
}

/// Writes the world of the scenario `scenario` of `demos` (`fixtures/demos/`) into a new folder at the world's own
/// clock, claimed by creating it ([`claim`]): `into` when given, else `<root>/<scenario>-<n>` with the first `n` from 1
/// that is free. An existing folder is refused, never removed, topped up or overwritten.
pub(crate) fn demo(
    demos: &Path,
    scenario: &str,
    into: Option<PathBuf>,
    root: &Path,
    protection: &Protection,
    user: u32,
) -> Result<DemoWritten, String> {
    if !is_plain_name(scenario) {
        return Err(format!("`{scenario}` is not a scenario name: letters, digits and `-` only"));
    }
    let path = demos.join("scenarios").join(format!("{scenario}.toml"));
    let played = Scenario::read(&path).map_err(|error| format!("scenario {scenario}: {error}"))?;
    let world_path = demos.join(format!("{}.toml", played.world));
    let text = fs::read_to_string(&world_path).map_err(|error| format!("{}: {error}", world_path.display()))?;
    let world = CaseFile::parse(&text).map_err(|error| format!("{}: {error}", world_path.display()))?;
    let now = world
        .demo()
        .map(Demo::now)
        .ok_or_else(|| format!("{}: not a demo world: it has no [demo] header", world_path.display()))?;
    let into = claim(into, root, scenario, user)?;
    let generated = generate(&[world], &into, now, protection, user).map_err(|error| error.to_string())?;
    Ok(DemoWritten { world: played.world, folder: generated.sandbox.path().to_path_buf() })
}

/// Claims a new, empty folder for one run by creating it, so that no two runs, and nothing made in between, share one:
/// `into` (its parents made as needed), refused when anything is there; else the first `<root>/<scenario>-<n>` that
/// can be created, `root` made as needed and refused when it is a link, not a folder or not owned by `user`.
fn claim(into: Option<PathBuf>, root: &Path, scenario: &str, user: u32) -> Result<PathBuf, String> {
    let failed = |path: &Path, error: std::io::Error| format!("{}: {error}", path.display());
    if let Some(into) = into {
        if let Some(parent) = into.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|error| failed(parent, error))?;
        }
        return match fs::create_dir(&into) {
            Ok(()) => Ok(into),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                Err(format!("{} exists: a demo is written into a new folder only", into.display()))
            }
            Err(error) => Err(failed(&into, error)),
        };
    }
    fs::create_dir_all(root).map_err(|error| failed(root, error))?;
    let metadata = fs::symlink_metadata(root).map_err(|error| failed(root, error))?;
    if metadata.is_symlink() || !metadata.is_dir() || metadata.uid() != user {
        return Err(format!("{} must be a folder of this user and no link", root.display()));
    }
    for run in 1..=MOST_RUNS {
        let folder = root.join(format!("{scenario}-{run}"));
        match fs::create_dir(&folder) {
            Ok(()) => return Ok(folder),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(failed(&folder, error)),
        }
    }
    Err(format!("{} holds {MOST_RUNS} demo folders of {scenario}: remove them by hand", root.display()))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::Path;

    use fresher_core::Protection;

    use super::{DemoWritten, demo};

    const SCENARIO: &str = "world = \"tiny-mac\"\nwindow = { width = 1100, height = 720 }\n[[step]]\npause = 800\n";
    const WORLD: &str = r#"feature = "tiny-mac"
[demo]
now = 1795000000
shows = ["trial.caches"]

[[case]]
name = "a browser's cache"
kind = "scenery"
[[case.file]]
path = "home/Library/Caches/com.example.Browser/data.bin"
size = 1024
age = "3d"
"#;
    const CACHE: &str = "home/Library/Caches/com.example.Browser/data.bin";

    /// A `fixtures/demos/` with the scenario `tiny` of the world `world`.
    fn demos(world: &str) -> tempfile::TempDir {
        let demos = tempfile::tempdir().unwrap();
        fs::create_dir(demos.path().join("scenarios")).unwrap();
        fs::write(demos.path().join("scenarios").join("tiny.toml"), SCENARIO).unwrap();
        fs::write(demos.path().join("tiny-mac.toml"), world).unwrap();
        demos
    }

    fn user(folder: &Path) -> u32 {
        fs::metadata(folder).unwrap().uid()
    }

    /// `demo tiny --into <into>`.
    fn run_into(demos: &Path, into: &Path) -> Result<DemoWritten, String> {
        demo(demos, "tiny", Some(into.to_path_buf()), Path::new("/nowhere"), &Protection::default(), user(demos))
    }

    /// `demo tiny` with the demo root `root`.
    fn run_in(demos: &Path, root: &Path) -> Result<DemoWritten, String> {
        demo(demos, "tiny", None, root, &Protection::default(), user(demos))
    }

    mod negative {
        use std::fs;

        use fresher_core::Protection;

        use super::{CACHE, WORLD, demos, run_in, run_into, user};
        use crate::demo_command::{claim, demo};

        #[test]
        fn a_scenario_that_is_not_a_plain_name_is_refused() {
            let demos = demos(WORLD);
            let root = tempfile::tempdir().unwrap();

            for scenario in ["", "..", "../tiny", "scenarios/tiny", "tiny.toml", "/tmp/tiny"] {
                let error = demo(demos.path(), scenario, None, root.path(), &Protection::default(), 0).unwrap_err();

                assert!(error.contains("not a scenario name"), "{scenario:?}: {error}");
            }
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
        }

        #[test]
        fn a_missing_scenario_is_refused_naming_its_path() {
            let demos = demos(WORLD);
            let root = tempfile::tempdir().unwrap();

            let error = demo(demos.path(), "gone", None, root.path(), &Protection::default(), 0).unwrap_err();

            assert!(error.contains("gone.toml"), "{error}");
        }

        #[test]
        fn an_existing_folder_is_refused_and_left_untouched() {
            let demos = demos(WORLD);
            let place = tempfile::tempdir().unwrap();
            let folder = place.path().join("folder");
            fs::create_dir(&folder).unwrap();
            fs::write(folder.join("notes.txt"), "mine").unwrap();
            let empty = place.path().join("empty");
            fs::create_dir(&empty).unwrap();
            let link = place.path().join("link");
            std::os::unix::fs::symlink(&folder, &link).unwrap();
            let file = place.path().join("file");
            fs::write(&file, "mine").unwrap();

            for into in [&folder, &empty, &link, &file] {
                let error = run_into(demos.path(), into).unwrap_err();

                assert!(error.contains("a demo is written into a new folder only"), "{}: {error}", into.display());
            }
            assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
            assert_eq!(fs::read_to_string(folder.join("notes.txt")).unwrap(), "mine");
            assert_eq!(fs::read_dir(&empty).unwrap().count(), 0);
            assert_eq!(fs::read_to_string(&file).unwrap(), "mine");
        }

        #[test]
        fn a_world_it_wrote_before_is_refused_and_left_as_it_is() {
            let demos = demos(WORLD);
            let place = tempfile::tempdir().unwrap();
            let folder = place.path().join("tiny");
            run_into(demos.path(), &folder).unwrap();
            fs::write(folder.join("home").join("extra.txt"), "added").unwrap();
            fs::write(folder.join(CACHE), "changed").unwrap();

            let error = run_into(demos.path(), &folder).unwrap_err();

            assert!(error.contains("a demo is written into a new folder only"), "{error}");

            assert_eq!(fs::read_to_string(folder.join("home").join("extra.txt")).unwrap(), "added");
            assert_eq!(fs::read_to_string(folder.join(CACHE)).unwrap(), "changed");
        }

        #[test]
        fn a_folder_made_before_the_claim_is_refused() {
            let place = tempfile::tempdir().unwrap();
            let folder = place.path().join("tiny");
            fs::create_dir(&folder).unwrap();

            let error = claim(Some(folder.clone()), place.path(), "tiny", user(place.path())).unwrap_err();

            assert!(error.contains("a demo is written into a new folder only"), "{error}");
        }

        #[test]
        fn a_root_that_is_a_link_a_file_or_of_another_user_is_refused() {
            let place = tempfile::tempdir().unwrap();
            let owner = user(place.path());
            let real = place.path().join("real");
            fs::create_dir(&real).unwrap();
            let link = place.path().join("link");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let file = place.path().join("file");
            fs::write(&file, "mine").unwrap();

            for (root, as_user) in [(&link, owner), (&file, owner), (&real, owner + 1)] {
                let error = claim(None, root, "tiny", as_user).unwrap_err();

                assert!(error.contains(&root.display().to_string()), "{}: {error}", root.display());
            }
            assert_eq!(fs::read_dir(&real).unwrap().count(), 0);
        }

        #[test]
        fn a_world_without_a_demo_header_is_refused() {
            let demos = demos("feature = \"tiny-mac\"\n[[case]]\nname = \"n\"\nkind = \"negative\"\n");
            let root = tempfile::tempdir().unwrap();

            let error = run_in(demos.path(), root.path()).unwrap_err();

            assert!(error.contains("no [demo] header"), "{error}");
            assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
        }
    }

    mod positive {
        use std::fs;
        use std::time::{Duration, SystemTime};

        use fresher_core::SANDBOX_MARKER;

        use super::{CACHE, WORLD, demos, run_in, run_into, user};
        use crate::demo_command::claim;

        #[test]
        fn writes_the_world_into_a_missing_folder_at_its_clock() {
            let demos = demos(WORLD);
            let place = tempfile::tempdir().unwrap();
            let folder = place.path().join("tiny");

            let written = run_into(demos.path(), &folder).unwrap();

            assert_eq!(written.world, "tiny-mac");
            assert_eq!(written.folder, fs::canonicalize(&folder).unwrap());
            assert!(folder.join(SANDBOX_MARKER).is_file());
            let modified = fs::metadata(folder.join(CACHE)).unwrap().modified().unwrap();
            let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_795_000_000);
            assert_eq!(modified, now - Duration::from_hours(72));
        }

        #[test]
        fn two_runs_make_two_folders_and_keep_the_first() {
            let demos = demos(WORLD);
            let root = tempfile::tempdir().unwrap();
            let root = fs::canonicalize(root.path()).unwrap();

            let first = run_in(demos.path(), &root.join("demos")).unwrap();
            fs::write(first.folder.join("home").join("extra.txt"), "added").unwrap();
            let second = run_in(demos.path(), &root.join("demos")).unwrap();

            assert_eq!(first.folder, root.join("demos").join("tiny-1"));
            assert_eq!(second.folder, root.join("demos").join("tiny-2"));
            assert_eq!(fs::read_to_string(first.folder.join("home").join("extra.txt")).unwrap(), "added");
            assert!(first.folder.join(CACHE).is_file());
            assert!(second.folder.join(CACHE).is_file());
            assert!(!second.folder.join("home").join("extra.txt").exists());
        }

        #[test]
        fn a_claim_takes_the_next_number_when_a_folder_was_made_first() {
            let place = tempfile::tempdir().unwrap();
            let root = place.path().join("demos");
            let owner = user(place.path());

            let first = claim(None, &root, "tiny", owner).unwrap();
            let second = claim(None, &root, "tiny", owner).unwrap();

            assert_eq!(first, root.join("tiny-1"));
            assert_eq!(second, root.join("tiny-2"));
            assert!(first.is_dir() && second.is_dir());
            assert_eq!(fs::read_dir(&first).unwrap().count(), 0);
        }

        #[test]
        fn takes_the_first_number_that_is_free() {
            let demos = demos(WORLD);
            let root = tempfile::tempdir().unwrap();
            let root = fs::canonicalize(root.path()).unwrap();
            fs::create_dir(root.join("tiny-1")).unwrap();
            fs::write(root.join("tiny-2"), "a file").unwrap();

            let written = run_in(demos.path(), &root).unwrap();

            assert_eq!(written.folder, root.join("tiny-3"));
            assert_eq!(fs::read_dir(root.join("tiny-1")).unwrap().count(), 0);
            assert_eq!(fs::read_to_string(root.join("tiny-2")).unwrap(), "a file");
        }
    }
}
