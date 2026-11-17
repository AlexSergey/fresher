use fresher_core::{BundleId, RunningApp};
use fresher_testing::FakeRunningApps;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::ErrorKind;
use std::time::{Duration, SystemTime};

use crate::{CaseFile, Sandbox, SandboxError};

/// The largest `world.toml` read: far more than any set of case files declares.
const MAX_WORLD_BYTES: u64 = 1024 * 1024;

/// What the case files declare about the Mac beyond their files: the apps that are open, and the moment the sandbox
/// was generated. Written as `world.toml` at the sandbox's top; sandbox mode and the autotest take the running apps
/// and the clock's start from it, never from the real Mac.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct World {
    /// The generator's "now", in seconds since the Unix epoch: every age of the case files counts back from it.
    now: Option<u64>,
    running: Vec<RunningApp>,
}

/// `world.toml`, as written and read back.
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawWorld {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    now: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    running: Vec<RawRunning>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawRunning {
    name: String,
    bundle_ids: Vec<String>,
}

impl World {
    /// The world of `files` generated at `now`, joined: every case's installed app declared `running = true` (its
    /// name and bundle id), then its `[[case.running]]`, in the order of the files and their cases.
    pub fn of(files: &[CaseFile], now: SystemTime) -> Self {
        let mut running = Vec::new();
        for case in files.iter().flat_map(CaseFile::cases) {
            // An app's bundle id was checked when its case was read.
            running.extend(case.running_apps().filter_map(|app| {
                let id = BundleId::new(app.bundle_id.as_str()).ok()?;
                Some(RunningApp { name: app.name.clone(), bundle_ids: vec![id] })
            }));
            running.extend(case.running_elsewhere().iter().cloned());
        }
        let now = now.duration_since(SystemTime::UNIX_EPOCH).ok().map(|since| since.as_secs());
        Self { now, running }
    }

    /// The world of `sandbox`, read from its `world.toml`, through no link; a missing file is an empty world. Anything
    /// but a file, a file over `MAX_WORLD_BYTES`, one that is not of the world's format, or one that names an app with
    /// no or an invalid bundle id, is refused.
    pub fn read(sandbox: &Sandbox) -> Result<Self, SandboxError> {
        let path = sandbox.world();
        let io = |source| SandboxError::Io { path: path.clone(), source };
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(io(error)),
            Ok(metadata) if metadata.is_symlink() => return Err(SandboxError::ThroughLink { path }),
            Ok(metadata) if !metadata.is_file() => {
                return Err(SandboxError::World { path, reason: "it is not a file".into() });
            }
            Ok(metadata) if metadata.len() > MAX_WORLD_BYTES => {
                let reason = format!("it is over {MAX_WORLD_BYTES} bytes");
                return Err(SandboxError::World { path, reason });
            }
            Ok(_) => {}
        }
        let text = fs::read_to_string(&path).map_err(io)?;
        Self::parse(&text).map_err(|reason| SandboxError::World { path, reason })
    }

    /// The running apps, in their declared order.
    pub fn running(&self) -> &[RunningApp] {
        &self.running
    }

    /// The moment the sandbox was generated, which its ages count back from; `None` for a world written before the
    /// world held it.
    pub fn now(&self) -> Option<SystemTime> {
        self.now.map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
    }

    /// The running apps as the port the app asks: the same answer at every call.
    pub fn running_apps(&self) -> FakeRunningApps {
        FakeRunningApps::new(self.running.clone())
    }

    /// The `world.toml` text.
    pub(crate) fn text(&self) -> Result<String, SandboxError> {
        let running = self
            .running
            .iter()
            .map(|app| RawRunning {
                name: app.name.clone(),
                bundle_ids: app.bundle_ids.iter().map(|id| id.as_str().to_owned()).collect(),
            })
            .collect();
        let body = toml::to_string(&RawWorld { now: self.now, running })
            .map_err(|error| SandboxError::World { path: "world.toml".into(), reason: error.to_string() })?;
        Ok(format!(
            "# The moment the sandbox was generated and the running apps the app sees in sandbox mode, written by the\n\
             # generator from the case files.\n{body}"
        ))
    }

    fn parse(text: &str) -> Result<Self, String> {
        let raw: RawWorld = toml::from_str(text).map_err(|error| error.to_string())?;
        let running = raw
            .running
            .into_iter()
            .map(|app| {
                if app.bundle_ids.is_empty() {
                    return Err(format!("the running app \"{}\" has no bundle id", app.name));
                }
                let bundle_ids = app
                    .bundle_ids
                    .iter()
                    .map(|id| BundleId::new(id.as_str()).map_err(|_| format!("`{id}` is not a bundle id")))
                    .collect::<Result<_, _>>()?;
                Ok(RunningApp { name: app.name, bundle_ids })
            })
            .collect::<Result<_, String>>()?;
        Ok(Self { now: raw.now, running })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    use fresher_core::Protection;
    use fresher_testing::FIXTURE_NOW;

    use crate::{CaseFile, Sandbox, World, generate};

    /// A case file whose cases declare `running` lines: a negative case with no app.
    fn case_file(feature: &str, running: &str) -> CaseFile {
        CaseFile::parse(&format!(
            "feature = \"{feature}\"\n[[case]]\nname = \"{feature} case\"\nkind = \"negative\"\n\
             [[case.file]]\npath = \"home/{feature}.bin\"\n{running}"
        ))
        .unwrap()
    }

    fn installed(path: &str, id: &str, name: &str, running: bool) -> String {
        format!("[[case.app]]\npath = \"{path}\"\nbundle-id = \"{id}\"\nname = \"{name}\"\nrunning = {running}\n")
    }

    fn elsewhere(name: &str, ids: &str) -> String {
        format!("[[case.running]]\nname = \"{name}\"\nbundle-ids = [{ids}]\n")
    }

    fn user(folder: &Path) -> u32 {
        use std::os::unix::fs::MetadataExt as _;
        fs::metadata(folder).unwrap().uid()
    }

    fn now() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW)
    }

    /// `files` generated into a temporary folder, opened.
    fn generated(files: &[CaseFile]) -> (tempfile::TempDir, Sandbox) {
        let folder = tempfile::tempdir().unwrap();
        generate(files, folder.path(), now(), &Protection::default(), user(folder.path())).unwrap();
        let sandbox = Sandbox::open(folder.path(), user(folder.path())).unwrap();
        (folder, sandbox)
    }

    fn names(world: &World) -> Vec<&str> {
        world.running().iter().map(|app| app.name.as_str()).collect()
    }

    /// Generates `files` again into the sandbox at `folder`.
    fn generate_into(folder: &Path, files: &[CaseFile]) {
        generate(files, folder, now(), &Protection::default(), user(folder)).unwrap();
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::symlink;

        use fresher_core::RunningApps;

        use super::{case_file, generated, installed};
        use crate::{SandboxError, World};

        #[test]
        fn a_sandbox_with_no_world_file_has_no_running_app_and_no_now() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            fs::rename(sandbox.world(), folder.path().join("home/moved.toml")).unwrap();

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world, World::default());
            assert_eq!(world.now(), None);
            assert_eq!(world.running_apps().list(), Ok(Vec::new()));
        }

        #[test]
        fn an_app_that_is_not_running_is_not_in_the_world() {
            let file = case_file("apps", &installed("Applications/Tool.app", "a.tool", "Tool", false));

            assert_eq!(World::of(&[file], super::now()).running(), []);
        }

        #[test]
        fn a_world_file_with_a_now_that_is_not_a_number_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for now in ["now = \"today\"\n", "now = -1\n"] {
                fs::write(sandbox.world(), now).unwrap();

                assert!(matches!(World::read(&sandbox), Err(SandboxError::World { .. })), "{now}");
            }
        }

        #[test]
        fn a_world_file_with_no_or_an_invalid_bundle_id_is_refused() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);
            for running in [
                "[[running]]\nname = \"CLion\"\nbundle-ids = []\n",
                "[[running]]\nname = \"CLion\"\n",
                "[[running]]\nname = \"CLion\"\nbundle-ids = [\"not an id\"]\n",
                "[[running]]\nname = \"CLion\"\nbundle-ids = [\"a.b\"]\ncolour = \"red\"\n",
            ] {
                fs::write(sandbox.world(), running).unwrap();

                let error = World::read(&sandbox).unwrap_err();

                assert!(matches!(error, SandboxError::World { .. }), "{running}: {error}");
            }
        }

        #[test]
        fn a_world_file_that_is_a_folder_or_too_large_is_refused() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            fs::rename(sandbox.world(), folder.path().join("home/moved.toml")).unwrap();
            fs::create_dir(sandbox.world()).unwrap();

            assert!(matches!(World::read(&sandbox), Err(SandboxError::World { .. })));

            let (_large_folder, large) = generated(&[case_file("plain", "")]);
            fs::OpenOptions::new().write(true).open(large.world()).unwrap().set_len(1024 * 1024 + 1).unwrap();

            assert!(matches!(World::read(&large), Err(SandboxError::World { .. })));
        }

        #[test]
        fn a_world_file_that_is_a_link_is_refused() {
            let (folder, sandbox) = generated(&[case_file("plain", "")]);
            let elsewhere = folder.path().join("home/elsewhere.toml");
            fs::write(&elsewhere, "").unwrap();
            fs::rename(sandbox.world(), folder.path().join("home/moved.toml")).unwrap();
            symlink(&elsewhere, sandbox.world()).unwrap();

            assert!(matches!(World::read(&sandbox), Err(SandboxError::ThroughLink { .. })));
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::RunningApps;

        use super::{case_file, elsewhere, generated, installed, names};
        use crate::World;

        #[test]
        fn an_installed_app_declared_running_is_in_the_world_with_its_bundle_id() {
            let file =
                case_file("apps", &installed("Applications/WebStorm.app", "com.jetbrains.WebStorm", "WebStorm", true));
            let (_folder, sandbox) = generated(&[file]);

            let world = World::read(&sandbox).unwrap();

            let listed = world.running_apps().list().unwrap();
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0].name, "WebStorm");
            assert_eq!(listed[0].bundle_ids[0].as_str(), "com.jetbrains.WebStorm");
        }

        #[test]
        fn a_running_entry_is_in_the_world_with_its_bundle_ids() {
            let file = case_file("open", &elsewhere("CLion", "\"com.jetbrains.CLion\", \"a.helper\""));
            let (_folder, sandbox) = generated(&[file]);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(names(&world), ["CLion"]);
            let ids: Vec<&str> = world.running()[0].bundle_ids.iter().map(fresher_core::BundleId::as_str).collect();
            assert_eq!(ids, ["com.jetbrains.CLion", "a.helper"]);
        }

        #[test]
        fn two_case_files_join_their_worlds() {
            let first = case_file("first", &installed("Applications/Tool.app", "a.tool", "Tool", true));
            let second = case_file("second", &elsewhere("CLion", "\"com.jetbrains.CLion\""));
            let (_folder, sandbox) = generated(&[first, second]);

            assert_eq!(names(&World::read(&sandbox).unwrap()), ["Tool", "CLion"]);
        }

        #[test]
        fn a_world_file_left_by_an_earlier_run_is_reset() {
            let open = case_file("box", &elsewhere("CLion", "\"com.jetbrains.CLion\""));
            let (folder, sandbox) = generated(&[open]);

            super::generate_into(folder.path(), &[case_file("box", "")]);

            let world = World::read(&sandbox).unwrap();
            assert_eq!(world.running(), []);
            assert_eq!(world.now(), Some(super::now()));
            assert!(fs::read_to_string(sandbox.world()).unwrap().starts_with('#'));
        }

        #[test]
        fn the_world_holds_the_moment_the_sandbox_was_generated() {
            let (_folder, sandbox) = generated(&[case_file("plain", "")]);

            let world = World::read(&sandbox).unwrap();

            assert_eq!(world.now(), Some(super::now()));
            let text = fs::read_to_string(sandbox.world()).unwrap();
            assert!(text.contains(&format!("now = {}", fresher_testing::FIXTURE_NOW)), "{text}");
        }

        #[test]
        fn the_world_file_reads_back_as_written() {
            let apps = format!(
                "{}{}",
                installed("Applications/A.app", "a.a", "A \\\"quoted\\\"", true),
                elsewhere("B", "\"b.b\"")
            );
            let world = World::of(&[case_file("box", &apps)], super::now());

            let parsed = World::parse(&world.text().unwrap()).unwrap();

            assert_eq!(parsed, world);
            assert_eq!(parsed.running_apps().list().unwrap().len(), 2);
        }
    }
}
