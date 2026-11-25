//! Demo mode of a development build: started with `FRESHER_SANDBOX` and `FRESHER_DEMO=<absolute path of a scenario>`,
//! the app sizes and places its window for the scenario and hands it to the UI's player (`demo_play`), which reports
//! how it ended (`demo_finished`). The shell reads only the scenario; it writes two files at the sandbox's root for the
//! recorder: `demo-window.json` (the window's content rectangle on its screen) and `demo-result.json` (the outcome).
//! Compiled in debug builds only (docs/plans/038-demo-recordings/00-design.md, section 2).
#![expect(clippy::needless_pass_by_value, reason = "Tauri hands a command its state and inputs by value")]

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use fresher_core::SANDBOX_MARKER;
use fresher_sandbox::{DemoOutcome, DemoPlay, Scenario, ScenarioError};
use serde::Serialize;
use tauri::{AppHandle, LogicalPosition, LogicalSize, PhysicalPosition, Runtime, State, WebviewWindow};

use crate::sandbox::SandboxTrash;

/// The variable naming the scenario, an absolute path.
const DEMO_VARIABLE: &str = "FRESHER_DEMO";
/// The window's rectangle, for the recorder, at the sandbox's root.
const WINDOW_FILE: &str = "demo-window.json";
/// How the scenario ended, for the recorder, at the sandbox's root.
const RESULT_FILE: &str = "demo-result.json";
/// Where the window is placed, in logical points of the main screen.
const DEMO_PLACE: (f64, f64) = (80.0, 80.0);

/// The exit code of a scenario played to its end.
const FINISHED_CODE: i32 = 0;
/// The exit code of a scenario that failed, or whose result could not be written.
const FAILED_CODE: i32 = 1;

/// The refusal of `demo_finished` outside demo mode.
const NOT_DEMO: &str = "the app is not in demo mode";

/// A development build's demo mode: the scenario, the sandbox it plays in, and the texts the page must never show.
#[derive(Debug)]
pub(crate) struct DemoMode {
    scenario: Scenario,
    sandbox: PathBuf,
    private: Vec<String>,
}

/// Why demo mode does not start: the app then stops instead of playing anywhere else.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DemoRefusal {
    #[error("{DEMO_VARIABLE} is set without FRESHER_SANDBOX")]
    WithoutSandbox,
    #[error("{DEMO_VARIABLE} must be an absolute path, not {path:?}")]
    NotAbsolute { path: PathBuf },
    #[error("a demo plays on the sandbox's own Trash, never the system's")]
    RealTrash,
    #[error("the sandbox was not generated from the demo world `{world}` of the scenario")]
    NotItsWorld { world: String },
    #[error("the scenario of {DEMO_VARIABLE}: {0}")]
    Scenario(#[from] ScenarioError),
}

/// The window's content rectangle in physical pixels, relative to its monitor's origin, and the monitor's scale.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub(crate) struct DemoRect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale: f64,
}

impl DemoMode {
    /// Whether the environment, read through `var`, asks for demo mode (`FRESHER_DEMO` is set), before it is checked.
    pub(crate) fn asked(var: impl Fn(&str) -> Option<OsString>) -> bool {
        var(DEMO_VARIABLE).is_some()
    }

    /// The mode the environment asks for, read through `var`: `None` when `FRESHER_DEMO` is unset; refused when it is
    /// set without a `sandbox` (the checked sandbox folder), names a relative path or no valid scenario, asks for the
    /// system's Trash, or names a scenario whose world the sandbox was not generated from.
    pub(crate) fn from_env(
        var: impl Fn(&str) -> Option<OsString>,
        sandbox: Option<&Path>,
        trash: SandboxTrash,
        real_home: &Path,
    ) -> Result<Option<Self>, DemoRefusal> {
        let Some(path) = var(DEMO_VARIABLE).map(PathBuf::from) else {
            return Ok(None);
        };
        let Some(sandbox) = sandbox else {
            return Err(DemoRefusal::WithoutSandbox);
        };
        if !path.is_absolute() {
            return Err(DemoRefusal::NotAbsolute { path });
        }
        if trash == SandboxTrash::Real {
            return Err(DemoRefusal::RealTrash);
        }
        let scenario = Scenario::read(&path)?;
        if !made_from(sandbox, &scenario.world) {
            return Err(DemoRefusal::NotItsWorld { world: scenario.world });
        }
        let private = [real_home, sandbox].map(|path| path.to_string_lossy().into_owned()).to_vec();
        Ok(Some(Self { scenario, sandbox: sandbox.to_path_buf(), private }))
    }

    /// Sizes `window` to the scenario and places it at [`DEMO_PLACE`]; a failure is logged, never fatal.
    pub(crate) fn place<R: Runtime>(&self, window: &WebviewWindow<R>) {
        let size = LogicalSize::new(f64::from(self.scenario.window.width), f64::from(self.scenario.window.height));
        if let Err(error) = window.set_size(size) {
            tracing::warn!(%error, "the demo window could not be sized");
        }
        if let Err(error) = window.set_position(LogicalPosition::new(DEMO_PLACE.0, DEMO_PLACE.1)) {
            tracing::warn!(%error, "the demo window could not be placed");
        }
    }

    fn play(&self) -> DemoPlay {
        DemoPlay { scenario: self.scenario.clone(), private: self.private.clone() }
    }
}

/// Whether the marker of `sandbox` names the demo world `world` as its only case file.
fn made_from(sandbox: &Path, world: &str) -> bool {
    let Ok(text) = fs::read_to_string(sandbox.join(SANDBOX_MARKER)) else {
        return false;
    };
    text.lines().any(|line| line.strip_prefix("case files: ") == Some(&format!("{world}.toml")))
}

/// The scenario a demo plays, or `None` outside demo mode. The UI is drawn when it asks: the window's rectangle is
/// written for the recorder first; a failed write is logged and the answer is still given.
#[tauri::command]
pub(crate) fn demo_play<R: Runtime>(window: WebviewWindow<R>, demo: State<'_, Option<DemoMode>>) -> Option<DemoPlay> {
    let mode = demo.inner().as_ref()?;
    match rect_of(&window) {
        Ok(rect) => {
            if let Err(error) = write_window(&mode.sandbox, rect) {
                tracing::warn!(%error, "the demo window's rectangle was not written");
            }
        }
        Err(error) => tracing::warn!(%error, "the demo window's rectangle could not be read"),
    }
    Some(mode.play())
}

/// The UI's player ended: the outcome is written for the recorder and the app exits, with 0 for a finished scenario.
/// Refused outside demo mode, where nothing is written and the app goes on.
#[tauri::command]
pub(crate) fn demo_finished<R: Runtime>(
    app: AppHandle<R>,
    demo: State<'_, Option<DemoMode>>,
    outcome: DemoOutcome,
) -> Result<(), String> {
    let code = finish(demo.inner().as_ref(), &outcome)?;
    app.exit(code);
    Ok(())
}

/// What `demo_finished` does before the exit: the exit code, after the result is written and a failure logged.
fn finish(demo: Option<&DemoMode>, outcome: &DemoOutcome) -> Result<i32, String> {
    let mode = demo.ok_or_else(|| NOT_DEMO.to_owned())?;
    let mut code = match outcome {
        DemoOutcome::Finished { .. } => FINISHED_CODE,
        DemoOutcome::Failed { step, reason } => {
            tracing::error!(step, reason, "the demo failed");
            FAILED_CODE
        }
    };
    if let Err(error) = write_result(&mode.sandbox, outcome) {
        tracing::error!(%error, "the demo's result was not written");
        code = FAILED_CODE;
    }
    Ok(code)
}

fn rect_of<R: Runtime>(window: &WebviewWindow<R>) -> tauri::Result<DemoRect> {
    let position = window.inner_position()?;
    let origin = window.current_monitor()?.map_or_else(
        || {
            tracing::warn!("the demo window has no monitor; its rectangle is relative to the origin");
            PhysicalPosition::default()
        },
        |monitor| *monitor.position(),
    );
    let size = window.inner_size()?;
    Ok(DemoRect {
        x: position.x - origin.x,
        y: position.y - origin.y,
        width: size.width,
        height: size.height,
        scale: window.scale_factor()?,
    })
}

/// Writes `rect` as `demo-window.json` in `folder`.
fn write_window(folder: &Path, rect: DemoRect) -> io::Result<()> {
    write_json(&folder.join(WINDOW_FILE), &rect)
}

/// Writes `outcome`, in the IPC's shape, as `demo-result.json` in `folder`.
fn write_result(folder: &Path, outcome: &DemoOutcome) -> io::Result<()> {
    write_json(&folder.join(RESULT_FILE), outcome)
}

/// Writes `value` as JSON at `path`, which must not be a link: a marked sandbox is not always a fresh demo folder.
fn write_json(path: &Path, value: &impl Serialize) -> io::Result<()> {
    if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_symlink()) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "the file is a link"));
    }
    fs::write(path, serde_json::to_vec(value)?)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    use fresher_core::{SANDBOX_MARKER, SANDBOX_SIGNATURE};

    use super::{DemoMode, DemoRefusal};
    use crate::sandbox::SandboxTrash;

    const SCENARIO: &str =
        "world = \"developer-mac\"\nwindow = { width = 1100, height = 720 }\n\n[[step]]\npause = 800\n";

    /// The environment with `FRESHER_DEMO` set to `value`, when given.
    fn env(value: Option<&Path>) -> impl Fn(&str) -> Option<OsString> {
        let value = value.map(|path| path.as_os_str().to_owned());
        move |name| (name == "FRESHER_DEMO").then(|| value.clone()).flatten()
    }

    /// A temp folder holding `overview.toml` with `text`.
    fn scenario(text: &str) -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("overview.toml");
        std::fs::write(&path, text).unwrap();
        (folder, path)
    }

    /// A folder marked as generated from the case files `files`.
    fn marked(files: &str) -> tempfile::TempDir {
        let folder = tempfile::tempdir().unwrap();
        std::fs::write(folder.path().join(SANDBOX_MARKER), format!("{SANDBOX_SIGNATURE}\ncase files: {files}\n"))
            .unwrap();
        folder
    }

    /// A demo mode playing in `sandbox`.
    fn mode(sandbox: &Path) -> DemoMode {
        let (_folder, path) = scenario(SCENARIO);
        let generated = marked("developer-mac.toml");
        let mut mode =
            DemoMode::from_env(env(Some(&path)), Some(generated.path()), SandboxTrash::Sandbox, Path::new("/Users/me"))
                .unwrap()
                .unwrap();
        mode.sandbox = sandbox.to_path_buf();
        mode
    }

    mod negative {
        use std::path::Path;

        use fresher_sandbox::{DemoOutcome, ScenarioError};

        use super::{DemoMode, DemoRefusal, env, marked, scenario};
        use crate::demo::{DemoRect, finish, write_result, write_window};
        use crate::sandbox::SandboxTrash;

        const HOME: &str = "/Users/me";
        const SANDBOX: &str = "/private/tmp/fresher-demo/overview-1";

        #[test]
        fn without_the_variable_no_demo_is_asked() {
            assert!(!DemoMode::asked(env(None)));
        }

        #[test]
        fn without_the_variable_there_is_no_demo_mode() {
            let mode = DemoMode::from_env(env(None), Some(Path::new(SANDBOX)), SandboxTrash::Sandbox, Path::new(HOME))
                .unwrap();

            assert!(mode.is_none());
        }

        #[test]
        fn the_variable_without_a_sandbox_is_refused() {
            let (_folder, path) = scenario(super::SCENARIO);

            let refused = DemoMode::from_env(env(Some(&path)), None, SandboxTrash::Sandbox, Path::new(HOME));

            assert!(matches!(refused, Err(DemoRefusal::WithoutSandbox)), "{refused:?}");
        }

        #[test]
        fn a_relative_scenario_path_is_refused() {
            let refused = DemoMode::from_env(
                env(Some(Path::new("overview.toml"))),
                Some(Path::new(SANDBOX)),
                SandboxTrash::Sandbox,
                Path::new(HOME),
            );

            assert!(matches!(refused, Err(DemoRefusal::NotAbsolute { .. })), "{refused:?}");
        }

        #[test]
        fn a_missing_scenario_is_refused() {
            let folder = tempfile::tempdir().unwrap();
            let path = folder.path().join("missing.toml");

            let refused =
                DemoMode::from_env(env(Some(&path)), Some(Path::new(SANDBOX)), SandboxTrash::Sandbox, Path::new(HOME));

            assert!(matches!(refused, Err(DemoRefusal::Scenario(ScenarioError::Io { .. }))), "{refused:?}");
        }

        #[test]
        fn an_invalid_scenario_is_refused() {
            let (_folder, path) = scenario("world = \"developer-mac\"\nwindow = { width = 100, height = 720 }\n");

            let refused =
                DemoMode::from_env(env(Some(&path)), Some(Path::new(SANDBOX)), SandboxTrash::Sandbox, Path::new(HOME));

            assert!(matches!(refused, Err(DemoRefusal::Scenario(ScenarioError::Window { .. }))), "{refused:?}");
            assert!(refused.unwrap_err().to_string().starts_with("the scenario of FRESHER_DEMO: "));
        }

        #[test]
        fn the_systems_trash_is_refused() {
            let (_folder, path) = scenario(super::SCENARIO);
            let generated = marked("developer-mac.toml");

            let refused =
                DemoMode::from_env(env(Some(&path)), Some(generated.path()), SandboxTrash::Real, Path::new(HOME));

            assert!(matches!(refused, Err(DemoRefusal::RealTrash)), "{refused:?}");
        }

        #[test]
        fn a_sandbox_of_another_world_is_refused() {
            let (_folder, path) = scenario(super::SCENARIO);
            let generated = marked("tiny-mac.toml");

            let refused =
                DemoMode::from_env(env(Some(&path)), Some(generated.path()), SandboxTrash::Sandbox, Path::new(HOME));

            assert!(matches!(refused, Err(DemoRefusal::NotItsWorld { .. })), "{refused:?}");
        }

        #[test]
        fn a_sandbox_without_a_marker_is_refused() {
            let (_folder, path) = scenario(super::SCENARIO);
            let bare = tempfile::tempdir().unwrap();

            let refused =
                DemoMode::from_env(env(Some(&path)), Some(bare.path()), SandboxTrash::Sandbox, Path::new(HOME));

            assert!(matches!(refused, Err(DemoRefusal::NotItsWorld { .. })), "{refused:?}");
        }

        #[test]
        fn a_link_in_place_of_a_result_file_is_not_written_through() {
            let folder = tempfile::tempdir().unwrap();
            let target = folder.path().join("target");
            std::fs::write(&target, "kept").unwrap();
            std::os::unix::fs::symlink(&target, folder.path().join("demo-result.json")).unwrap();
            std::os::unix::fs::symlink(&target, folder.path().join("demo-window.json")).unwrap();

            let result = write_result(folder.path(), &DemoOutcome::Finished { elapsed_ms: 1 });
            let window = write_window(folder.path(), DemoRect { x: 0, y: 0, width: 1, height: 1, scale: 1.0 });

            assert!(result.is_err());
            assert!(window.is_err());
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "kept");
        }

        #[test]
        fn finishing_outside_demo_mode_is_refused_and_writes_nothing() {
            let outcome = DemoOutcome::Finished { elapsed_ms: 1200 };

            let refused = finish(None, &outcome);

            assert_eq!(refused, Err("the app is not in demo mode".to_owned()));
        }

        #[test]
        fn a_result_that_cannot_be_written_fails_the_exit() {
            let folder = tempfile::tempdir().unwrap();
            let mode = super::mode(&folder.path().join("gone"));

            let code = finish(Some(&mode), &DemoOutcome::Finished { elapsed_ms: 1200 });

            assert_eq!(code, Ok(1));
        }
    }

    mod positive {
        use std::fs;
        use std::path::Path;

        use fresher_sandbox::DemoOutcome;

        use super::{DemoMode, env, marked, mode, scenario};
        use crate::demo::{DemoRect, finish, write_result, write_window};
        use crate::sandbox::SandboxTrash;

        #[test]
        fn the_variable_asks_for_a_demo_before_it_is_checked() {
            assert!(DemoMode::asked(env(Some(Path::new("not even absolute.toml")))));
        }

        #[test]
        fn a_valid_scenario_gives_a_mode_that_keeps_the_home_and_the_sandbox_private() {
            let (_folder, path) = scenario(super::SCENARIO);
            let generated = marked("developer-mac.toml");
            let sandbox = generated.path();

            let mode =
                DemoMode::from_env(env(Some(&path)), Some(sandbox), SandboxTrash::Sandbox, Path::new("/Users/me"))
                    .unwrap()
                    .unwrap();

            let play = mode.play();
            assert_eq!(play.private, ["/Users/me".to_owned(), sandbox.to_string_lossy().into_owned()]);
            assert_eq!(play.scenario.name, "overview");
            assert_eq!((play.scenario.window.width, play.scenario.window.height), (1100, 720));
            assert_eq!(mode.sandbox, sandbox);
        }

        #[test]
        fn the_window_file_holds_the_rectangle_in_physical_pixels() {
            let folder = tempfile::tempdir().unwrap();
            let rect = DemoRect { x: 160, y: 160, width: 2200, height: 1440, scale: 2.0 };

            write_window(folder.path(), rect).unwrap();

            let written = fs::read_to_string(folder.path().join("demo-window.json")).unwrap();
            assert_eq!(written, r#"{"x":160,"y":160,"width":2200,"height":1440,"scale":2.0}"#);
        }

        #[test]
        fn the_result_file_holds_the_outcome_in_the_ipc_shape() {
            let folder = tempfile::tempdir().unwrap();
            let file = folder.path().join("demo-result.json");

            write_result(folder.path(), &DemoOutcome::Finished { elapsed_ms: 1200 }).unwrap();
            assert_eq!(fs::read_to_string(&file).unwrap(), r#"{"kind":"finished","elapsedMs":1200}"#);

            write_result(folder.path(), &DemoOutcome::Failed { step: 3, reason: "no button Scan".into() }).unwrap();
            assert_eq!(fs::read_to_string(&file).unwrap(), r#"{"kind":"failed","step":3,"reason":"no button Scan"}"#);
        }

        #[test]
        fn finishing_writes_the_result_and_exits_by_the_outcome() {
            let folder = tempfile::tempdir().unwrap();
            let mode = mode(folder.path());
            let file = folder.path().join("demo-result.json");

            assert_eq!(finish(Some(&mode), &DemoOutcome::Finished { elapsed_ms: 900 }), Ok(0));
            assert_eq!(fs::read_to_string(&file).unwrap(), r#"{"kind":"finished","elapsedMs":900}"#);

            assert_eq!(finish(Some(&mode), &DemoOutcome::Failed { step: 1, reason: "gone".into() }), Ok(1));
            assert_eq!(fs::read_to_string(&file).unwrap(), r#"{"kind":"failed","step":1,"reason":"gone"}"#);
        }
    }
}
