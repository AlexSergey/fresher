//! Sandbox mode of a development build: started with `FRESHER_SANDBOX=<folder>`, the app lives entirely inside that
//! sandbox folder — its home and known folders, its data folder, its Trash — and its cleaner reaches nothing outside
//! it (docs/plans/007-safe-development/00-design.md, "Sandbox mode of the app"). Compiled in debug builds only.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach, SandboxFolder};
use fresher_core::{AccessProbe, Clock, CommandRunner, ModuleId, OpenFiles, RunningApps, Trash, Volumes};
use fresher_engine::{CleanupPorts, Engine};
use fresher_macos::{MacOpenFiles, MacTrash, MacVolumes, current_user, places_that_prompt, protection};
use fresher_sandbox::Sandbox;
use fresher_store::{DataFolder, FileCleanupJournal};
use fresher_system::{COMMAND_TIMEOUT, MacRunningApps, ProcessCommandRunner, SystemClock};
use fresher_testing::FakeTrash;

use crate::adapters::UnknownAccess;
use crate::compose::{ComposeError, Shared, assemble, real_protection};

/// The variable naming the sandbox folder, an absolute path.
const SANDBOX_VARIABLE: &str = "FRESHER_SANDBOX";
/// The variable choosing the Trash: absent for the sandbox's own, `real` for the system's.
const TRASH_VARIABLE: &str = "FRESHER_SANDBOX_TRASH";
/// The one value of [`TRASH_VARIABLE`].
const REAL_TRASH: &str = "real";

/// Where a cleanup in sandbox mode moves items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxTrash {
    /// The sandbox's own Trash, `home/.Trash`: nothing leaves the sandbox.
    Sandbox,
    /// The system's Trash, for the sandbox's own items only (`--real-trash`).
    Real,
}

/// A development build's sandbox mode: the sandbox folder, the Trash its cleanups use, and the modules it registers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxMode {
    path: PathBuf,
    trash: SandboxTrash,
    /// The one module registered, for an autotest of its case file; `None` (the app's) registers every module.
    only: Option<ModuleId>,
}

/// Why sandbox mode does not start: the app then stops instead of running anywhere else.
#[derive(Debug, thiserror::Error)]
pub enum SandboxRefusal {
    #[error("{SANDBOX_VARIABLE} must be an absolute path, not {path:?}")]
    NotAbsolute { path: PathBuf },
    #[error("{TRASH_VARIABLE} must be `{REAL_TRASH}` or unset, not {value:?}")]
    UnknownTrash { value: OsString },
    #[error("{TRASH_VARIABLE} is set without {SANDBOX_VARIABLE}")]
    TrashWithoutSandbox,
    /// A link, a protected place of this Mac, or a folder without a valid marker of the user.
    #[error(transparent)]
    Folder(#[from] fresher_cleaner::SandboxError),
    /// A part of the sandbox's layout, such as `home/`, is missing.
    #[error(transparent)]
    Layout(#[from] fresher_sandbox::SandboxError),
}

/// Why the sandbox's engine could not be made.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct SandboxEngineError(#[from] ComposeError);

impl SandboxEngineError {
    /// Why sandbox mode refused the folder or the environment, when that is the error.
    pub const fn refusal(&self) -> Option<&SandboxRefusal> {
        match &self.0 {
            ComposeError::Sandbox(refusal) => Some(refusal),
            _ => None,
        }
    }
}

impl SandboxMode {
    /// The sandbox at `path`, its cleanups going to `trash`.
    pub fn new(path: impl Into<PathBuf>, trash: SandboxTrash) -> Self {
        Self { path: path.into(), trash, only: None }
    }

    /// The same mode with only `module` registered, as the autotest of a case file naming it runs; no module when the
    /// app has none with that id.
    #[must_use]
    pub fn only(self, module: ModuleId) -> Self {
        Self { only: Some(module), ..self }
    }

    /// The mode the environment asks for, read through `var`: `FRESHER_SANDBOX` (an absolute path) and
    /// `FRESHER_SANDBOX_TRASH` (`real` or absent); `None` when neither is set. Anything else is refused.
    pub fn from_env(var: impl Fn(&str) -> Option<OsString>) -> Result<Option<Self>, SandboxRefusal> {
        let trash = match var(TRASH_VARIABLE) {
            None => SandboxTrash::Sandbox,
            Some(value) if value == REAL_TRASH => SandboxTrash::Real,
            Some(value) => return Err(SandboxRefusal::UnknownTrash { value }),
        };
        let Some(path) = var(SANDBOX_VARIABLE).map(PathBuf::from) else {
            return match trash {
                SandboxTrash::Sandbox => Ok(None),
                SandboxTrash::Real => Err(SandboxRefusal::TrashWithoutSandbox),
            };
        };
        if !path.is_absolute() {
            return Err(SandboxRefusal::NotAbsolute { path });
        }
        Ok(Some(Self { path, trash, only: None }))
    }

    /// The sandbox folder as it was named.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Where its cleanups move items.
    pub const fn trash(&self) -> SandboxTrash {
        self.trash
    }
}

/// What sandbox mode keeps real in the app, and the autotests replace by fakes: all of it is only read.
#[derive(Clone)]
pub struct SandboxPorts {
    pub clock: Arc<dyn Clock>,
    pub volumes: Arc<dyn Volumes>,
    pub open_files: Arc<dyn OpenFiles>,
    pub running: Arc<dyn RunningApps>,
    pub commands: Arc<dyn CommandRunner>,
    pub access: Arc<dyn AccessProbe>,
}

impl std::fmt::Debug for SandboxPorts {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("SandboxPorts").finish_non_exhaustive()
    }
}

impl SandboxPorts {
    /// The app's real ports, as a normal development build has them.
    pub(crate) fn real() -> Self {
        Self {
            clock: Arc::new(SystemClock),
            volumes: Arc::new(MacVolumes::new()),
            open_files: Arc::new(MacOpenFiles::new()),
            running: Arc::new(MacRunningApps::new()),
            commands: Arc::new(ProcessCommandRunner::new(COMMAND_TIMEOUT)),
            access: Arc::new(UnknownAccess),
        }
    }
}

/// The engine of sandbox mode over `ports`: the sandbox's home and known folders, its data folder
/// `<sandbox>/.fresher-data`, its Trash (`home/.Trash`, or the system's for [`SandboxTrash::Real`]), a cleaner that
/// reaches only inside the sandbox folder, and both protection lists — the real Mac's for `real_home` (with the
/// development build's own `real_data_folder`) and the sandbox home's. Refused when the folder is a link, is
/// protected on the real Mac, holds no valid marker of the user, or misses a part of its layout. It leaves the
/// process's IO policy alone: the app's start sets it.
pub fn sandbox_engine(
    sandbox: &SandboxMode,
    real_home: &Path,
    real_data_folder: &Path,
    ports: SandboxPorts,
) -> Result<Engine, SandboxEngineError> {
    Ok(compose(sandbox, real_home, real_data_folder, ports)?.0)
}

/// The sandbox's engine and its checked layout.
pub(crate) fn compose(
    mode: &SandboxMode,
    real_home: &Path,
    real_data_folder: &Path,
    ports: SandboxPorts,
) -> Result<(Engine, Sandbox), ComposeError> {
    let user = current_user();
    let real = real_protection(real_home, real_data_folder)?;
    let folder = SandboxFolder::new(mode.path(), user, &real).map_err(SandboxRefusal::from)?;
    let sandbox = Sandbox::open(folder.path(), user).map_err(SandboxRefusal::from)?;
    let home = sandbox.home();
    let data = DataFolder::open(sandbox.data_folder())?;
    // Each list keeps its own carve-outs: the real Mac stays safe whatever a bug does, and the sandbox behaves like a
    // Mac (its `Documents` whole, its `.Trash` never touched).
    let joined = real.joined(&protection(&home, data.path(), None)?)?;
    let trash: Arc<dyn Trash> = match mode.trash() {
        SandboxTrash::Sandbox => Arc::new(FakeTrash::new(sandbox.trash())),
        SandboxTrash::Real => Arc::new(MacTrash::new()),
    };
    let cleaner = CleanerPorts {
        trash,
        open_files: ports.open_files,
        running: Arc::clone(&ports.running),
        commands: Arc::clone(&ports.commands),
        clock: Arc::clone(&ports.clock),
    };
    let cleaner = DiskCleaner::new(Reach::SandboxFolder(folder), user, home.clone(), joined, cleaner);
    let cleanup = CleanupPorts {
        cleaner: Arc::new(cleaner),
        journal: Arc::new(FileCleanupJournal::open(&data)),
        volumes: Arc::clone(&ports.volumes),
    };
    // Neither home's places that prompt are opened: a scan root outside the sandbox, such as the real home, must
    // raise no system prompt either (docs/hacks/).
    let mut avoid = places_that_prompt(&home);
    avoid.extend(places_that_prompt(real_home));
    let engine = assemble(Shared {
        folder: data,
        avoid,
        known_folders: Arc::new(sandbox.known_folders()),
        volumes: ports.volumes,
        clock: ports.clock,
        running: ports.running,
        commands: ports.commands,
        access: ports.access,
        cleanup,
        only: mode.only,
    })?;
    Ok((engine, sandbox))
}

#[cfg(test)]
mod tests;
