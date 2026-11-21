//! The development run record: what a run of a development build did, written beside the repository for a later
//! check. Debug builds only; nothing is written unless the launcher set `FRESHER_DEV_RUNS`.
//! Design: `docs/plans/026-dev-run-record/00-design.md`.

pub(crate) mod clock;
pub(crate) mod commands;
pub(crate) mod english;
pub(crate) mod files;
pub(crate) mod record;
pub(crate) mod run;
#[cfg(test)]
mod shell_tests;

use std::ffi::{OsStr, OsString};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;

use fresher_core::{Build, Event};
use fresher_engine::Engine;
use serde::{Serialize, Serializer};
use tauri::ipc::InvokeBody;
use tauri::{AppHandle, Manager, Runtime};

use self::commands::is_action;
use self::run::{Run, SnapshotWriter};

/// The folder the records go to; set by `scripts/dev.ts`.
pub(crate) const DEV_RUNS_VARIABLE: &str = "FRESHER_DEV_RUNS";
/// The short commit of the build; set by `scripts/dev.ts`.
pub(crate) const COMMIT_VARIABLE: &str = "FRESHER_BUILD_COMMIT";
/// `1` when the working tree had uncommitted changes at launch; set by `scripts/dev.ts`.
pub(crate) const DIRTY_VARIABLE: &str = "FRESHER_BUILD_DIRTY";

const UNKNOWN_COMMIT: &str = "unknown";

/// What the environment asks for: a folder to record into and the build that records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Launch {
    folder: PathBuf,
    build: BuildName,
}

impl Launch {
    /// `None` without `FRESHER_DEV_RUNS` (nothing is written); refused when it is not an absolute path.
    pub(crate) fn from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Option<Self>, DevRunsRefusal> {
        let Some(folder) = var(DEV_RUNS_VARIABLE) else {
            return Ok(None);
        };
        let folder = PathBuf::from(folder);
        if !folder.is_absolute() {
            return Err(DevRunsRefusal::NotAbsolute { path: folder });
        }
        let commit = var(COMMIT_VARIABLE);
        let dirty = var(DIRTY_VARIABLE);
        let build = BuildName::new(env!("CARGO_PKG_VERSION"), commit.as_deref(), dirty.as_deref());
        Ok(Some(Self { folder, build }))
    }
}

/// The build that records: its version, its commit and whether the working tree was dirty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BuildName {
    version: String,
    commit: String,
    dirty: bool,
}

impl BuildName {
    /// A commit of 4 to 40 characters of `0-9a-f` is kept, anything else is `unknown`; dirty only for exactly `1`.
    pub(crate) fn new(version: &str, commit: Option<&OsStr>, dirty: Option<&OsStr>) -> Self {
        let commit = commit
            .and_then(OsStr::to_str)
            .filter(|commit| {
                (4..=40).contains(&commit.len()) && commit.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
            })
            .unwrap_or(UNKNOWN_COMMIT);
        Self { version: version.to_owned(), commit: commit.to_owned(), dirty: dirty == Some(OsStr::new("1")) }
    }

    /// `<version>+<commit>[-dirty]`.
    pub(crate) fn name(&self) -> String {
        let dirty = if self.dirty { "-dirty" } else { "" };
        format!("{}+{}{dirty}", self.version, self.commit)
    }
}

/// How a build is written in the record.
#[derive(Serialize)]
struct BuildRecord<'a> {
    version: &'a str,
    commit: &'a str,
    dirty: bool,
    name: String,
}

impl Serialize for BuildName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        BuildRecord { version: &self.version, commit: &self.commit, dirty: self.dirty, name: self.name() }
            .serialize(serializer)
    }
}

/// Where the run happens: the real Mac or a sandbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    Mac,
    /// A sandbox, by its folder's name made safe for a file name.
    Sandbox(String),
}

impl Mode {
    /// `None` for a release build, which records nothing.
    pub(crate) fn of(build: &Build) -> Option<Self> {
        match build {
            Build::Release => None,
            Build::Development => Some(Self::Mac),
            Build::Sandbox { path, .. } => {
                let last = path.file_name().map(|name| name.to_string_lossy()).unwrap_or_default();
                let safe: String = last
                    .chars()
                    .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
                    .collect();
                Some(Self::Sandbox(if safe.is_empty() { "sandbox".to_owned() } else { safe }))
            }
        }
    }

    /// `mac` or `sandbox-<name>`.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Mac => "mac".to_owned(),
            Self::Sandbox(name) => format!("sandbox-{name}"),
        }
    }
}

/// The run being recorded, managed by the app: the thread of [`follow`] folds the engine's events into it, and
/// [`commands::observe`] the actions.
/// The snapshot is made under the run's lock and written under the writer's alone, so an action never waits on the
/// disk.
#[derive(Debug)]
pub(crate) struct DevRuns {
    run: Mutex<Run>,
    writer: Mutex<SnapshotWriter>,
}

impl DevRuns {
    pub(crate) fn new(run: Run) -> Self {
        let writer = Mutex::new(run.writer());
        Self { run: Mutex::new(run), writer }
    }

    /// Records `command` with its JSON arguments (a raw body as `null`) when it is an action.
    pub(crate) fn call(&self, command: &str, payload: &InvokeBody) {
        if !is_action(command) {
            return;
        }
        let args = match payload {
            InvokeBody::Json(value) => value.clone(),
            InvokeBody::Raw(_) => serde_json::Value::Null,
        };
        lock(&self.run).action(command, args);
    }

    pub(crate) fn event(&self, event: &Event, engine: &Engine) {
        let snapshot = lock(&self.run).event(event, engine);
        if let Some(snapshot) = snapshot {
            lock(&self.writer).write(&snapshot);
        }
    }
}

/// Folds every event of `events` into the app's managed run, on the thread `fresher-dev-runs`, until the engine's
/// events end. A thread that cannot start is logged; the app runs on unrecorded.
pub(crate) fn follow<R: Runtime>(app: AppHandle<R>, events: Receiver<Event>) {
    let spawned = thread::Builder::new().name("fresher-dev-runs".into()).spawn(move || {
        let (Some(runs), Some(engine)) = (app.try_state::<DevRuns>(), app.try_state::<Engine>()) else {
            tracing::warn!("the development run record has no run or no engine to follow");
            return;
        };
        for event in events {
            runs.event(&event, &engine);
        }
    });
    if let Err(error) = spawned {
        tracing::warn!(%error, "the development run record cannot follow the engine");
    }
}

/// The lock stays usable whatever a panicking thread was doing with it.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Why the record cannot be kept; a run the user thinks is recorded is never silently lost.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DevRunsRefusal {
    #[error("{DEV_RUNS_VARIABLE} must be an absolute folder, not `{}`", path.display())]
    NotAbsolute { path: PathBuf },
    #[error("the folder named by {DEV_RUNS_VARIABLE} cannot be created: `{}`", path.display())]
    Folder { path: PathBuf, source: io::Error },
    #[error("{DEV_RUNS_VARIABLE} names `{}`, which is not a folder (or is a link)", path.display())]
    NotAFolder { path: PathBuf },
    #[error("every record name `{stem}` up to -99 is taken in the {DEV_RUNS_VARIABLE} folder")]
    Taken { stem: String },
    #[error("a record file cannot be written: `{}`", path.display())]
    Write { path: PathBuf, source: io::Error },
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::ffi::OsString;
        use std::path::Path;

        use fresher_core::Build;

        use crate::dev_runs::{BuildName, DevRunsRefusal, Launch, Mode};

        #[test]
        fn no_variable_starts_no_run() {
            assert_eq!(Launch::from_env(|_| None).unwrap(), None);
        }

        #[test]
        fn a_relative_folder_is_refused() {
            let result = Launch::from_env(|name| (name == "FRESHER_DEV_RUNS").then(|| OsString::from("dev-runs")));

            assert!(
                matches!(result, Err(DevRunsRefusal::NotAbsolute { path }) if path.as_path() == Path::new("dev-runs"))
            );
        }

        #[test]
        fn a_commit_that_is_not_hex_is_unknown() {
            let long = "1".repeat(41);
            for commit in ["26E4CB0", "abc", "26e4cb0; rm", long.as_str()] {
                let build = BuildName::new("0.1.0", Some(commit.as_ref()), None);

                assert_eq!(build.name(), "0.1.0+unknown", "{commit:?}");
            }
        }

        #[test]
        fn dirty_is_only_one() {
            for dirty in [Some("true"), Some("yes"), Some(""), Some("0"), None] {
                let build = BuildName::new("0.1.0", Some("26e4cb0".as_ref()), dirty.map(AsRef::as_ref));

                assert_eq!(build.name(), "0.1.0+26e4cb0", "{dirty:?}");
            }
        }

        #[test]
        fn a_release_build_has_no_mode() {
            assert_eq!(Mode::of(&Build::Release), None);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::Build;

        use crate::dev_runs::{BuildName, Launch, Mode};

        #[test]
        fn the_build_name_is_version_plus_commit_and_dirty() {
            let dirty = BuildName::new("0.1.0", Some("26e4cb0".as_ref()), Some("1".as_ref()));
            let unknown = BuildName::new("0.1.0", None, None);

            assert_eq!(dirty.name(), "0.1.0+26e4cb0-dirty");
            assert_eq!(unknown.name(), "0.1.0+unknown");
            assert_eq!(
                serde_json::to_value(&dirty).unwrap(),
                serde_json::json!({
                    "version": "0.1.0", "commit": "26e4cb0", "dirty": true, "name": "0.1.0+26e4cb0-dirty"
                })
            );
        }

        #[test]
        fn a_launch_reads_the_folder_and_the_build() {
            let launch = Launch::from_env(|name| match name {
                "FRESHER_DEV_RUNS" => Some("/x/dev-runs".into()),
                "FRESHER_BUILD_COMMIT" => Some("26e4cb0".into()),
                "FRESHER_BUILD_DIRTY" => Some("1".into()),
                _ => None,
            })
            .unwrap()
            .unwrap();

            assert_eq!(launch.folder, PathBuf::from("/x/dev-runs"));
            assert_eq!(launch.build.name(), format!("{}+26e4cb0-dirty", env!("CARGO_PKG_VERSION")));
        }

        #[test]
        fn a_sandbox_mode_is_named_by_its_folder() {
            for (path, label) in [("/x/sandbox/trial", "sandbox-trial"), ("/x/my box!", "sandbox-my-box-")] {
                let build = Build::Sandbox {
                    path: PathBuf::from(path),
                    simulated: Vec::new(),
                    trash: fresher_core::SandboxTrashAt::System.text(),
                };

                assert_eq!(Mode::of(&build).unwrap().label(), label, "{path}");
            }
        }

        #[test]
        fn a_development_build_is_mac() {
            assert_eq!(Mode::of(&Build::Development).unwrap().label(), "mac");
        }
    }
}
