use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::{Access, AccessStatus, CancelToken, Capability, GuardedPlace};

/// The wall clock, injected so that tests never read the real time.
pub trait Clock: Send + Sync {
    fn now(&self) -> SystemTime;
}

/// The system's Trash. Only the cleaner is given one: it moves an item there by default, and puts it back.
pub trait Trash: Send + Sync {
    /// Moves the item at `path`, whose open handle the cleaner holds as `held`, to the Trash of its volume, and says
    /// where it landed. Nothing is ever deleted instead: a volume without a Trash answers `NoTrash`.
    fn move_to_trash(&self, path: &Path, held: &File) -> Result<Landed, TrashError>;
    /// Moves `landed` back to `original`; never replaces anything at `original` and follows no link in either path.
    fn put_back(&self, landed: &Path, original: &Path) -> Result<(), TrashError>;
}

/// Where a moved item landed. `at` is where the held handle's item is after the move, read from the handle itself;
/// `reported` is the landed path the system's call returned, which it may leave out. The cleaner trusts `reported`
/// only when that path holds the verified identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Landed {
    pub at: PathBuf,
    pub reported: Option<PathBuf>,
}

/// Why the Trash did not move an item; the item stays where it was. No variant holds a path.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum TrashError {
    /// Nothing is at the path.
    #[error("the item does not exist")]
    NotFound,
    /// Its volume has no Trash (a read-only or foreign volume).
    #[error("the volume has no Trash")]
    NoTrash,
    /// No permission: the item is locked, its folder is read-only, or the volume refuses a Trash folder.
    #[error("the Trash call was not allowed")]
    NotAllowed,
    /// Put back: something is at the original path.
    #[error("the original place is taken")]
    Occupied,
    /// Put back: the folder of the original path is gone.
    #[error("the original folder is gone")]
    NoParent,
    /// Put back: a component of either path is a link.
    #[error("a path goes through a link")]
    ThroughLink,
    /// Any other refusal, with the system's words for the log.
    #[error("the Trash refused: {reason}")]
    Refused { reason: String },
}

/// The files and folders the user's own processes hold open, read from the system's process table.
pub trait OpenFiles: Send + Sync {
    /// One snapshot of what is held open at or under `within`, with the name of each holder.
    fn snapshot(&self, within: &[PathBuf]) -> Result<OpenSet, OpenFilesError>;
}

/// Why the open files could not be listed: the cleanup is refused, fail closed.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
#[error("the open files could not be listed: {reason}")]
pub struct OpenFilesError {
    pub reason: String,
}

/// Paths held open, each with the name of a process holding it, sorted by path components.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenSet {
    held: Vec<(PathBuf, String)>,
}

impl OpenSet {
    pub fn new(mut held: Vec<(PathBuf, String)>) -> Self {
        held.sort();
        Self { held }
    }

    /// The name of a process holding `path` or anything inside it, by components: `/a/b` is not inside `/a/bc`.
    pub fn holder_under(&self, path: &Path) -> Option<&str> {
        // `Path` orders by components, so everything inside `path` follows it with nothing else in between.
        let first = self.held.partition_point(|(held, _)| held.as_path() < path);
        self.held.get(first).filter(|(held, _)| held.starts_with(path)).map(|(_, app)| app.as_str())
    }

    pub fn len(&self) -> usize {
        self.held.len()
    }

    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }
}

/// The result of an external program (the Docker CLI, `tmutil`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runs external programs, injected so that tests use a fake instead of the real Docker CLI.
pub trait CommandRunner: Send + Sync {
    /// Runs `program` with exactly `args`, never through a shell. Once `cancel` is set the program is stopped and the
    /// run answers `Cancelled`; a token set before the call runs nothing.
    fn run(&self, program: &str, args: &[&str], cancel: &CancelToken) -> Result<CommandOutput, CommandRunError>;
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum CommandRunError {
    #[error("{program} is not installed")]
    NotInstalled { program: String },
    #[error("{program} could not be started: {reason}")]
    Failed { program: String, reason: String },
    /// The caller's cancel token was set: the program was stopped, or never started.
    #[error("{program} was stopped: the operation was cancelled")]
    Cancelled { program: String },
}

/// What the system lets the app see. Only reads: a probe never opens anything on the system's interface.
pub trait AccessProbe: Send + Sync {
    /// The state of every permission this system has. Never causes a system prompt.
    fn status(&self) -> AccessStatus;
    /// The places the system guards (under the home folder the adapter was built with, and the mounted volumes of
    /// the scan roots), each with the capability that opens it.
    fn guarded_places(&self, roots: &[PathBuf]) -> Vec<GuardedPlace>;
    /// Tries one guarded place. MAY cause a system prompt: called only inside a scan the user started, for a place
    /// the user opted into.
    fn try_place(&self, place: &GuardedPlace) -> Access;
}

/// Acts on the system's interface, on the user's click only.
pub trait SystemUi: Send + Sync {
    /// Opens the system's settings at the pane that grants `capability`.
    fn open_access_settings(&self, capability: &Capability) -> Result<(), SystemUiError>;
    /// Shows a file or folder in the system's file manager, selected.
    fn reveal(&self, path: &Path) -> Result<(), SystemUiError>;
    /// Opens a folder in the system's file manager, as the user would by double-clicking it.
    fn open_folder(&self, path: &Path) -> Result<(), SystemUiError>;
}

/// Why the system's interface did not do what it was asked.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum SystemUiError {
    /// This system has no such pane or action.
    #[error("the system does not support this")]
    Unsupported,
    #[error("the system's interface failed: {reason}")]
    Failed { reason: String },
}

/// Whether Finder removes items from the Trash 30 days after they were put there (Finder → Settings → Advanced).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoEmpty {
    /// The setting is on: an item leaves the Trash by itself 30 days after it was put there.
    After30Days,
    /// The setting is off: items stay in the Trash until the user empties it.
    Never,
    /// The setting could not be read.
    Unknown,
}

/// Finder's settings the app reads, never writes.
pub trait FinderSettings: Send + Sync {
    /// Whether Finder empties old items from the Trash by itself.
    fn trash_auto_empty(&self) -> AutoEmpty;
}

#[cfg(test)]
mod tests {
    use crate::OpenSet;

    fn held() -> OpenSet {
        OpenSet::new(vec![
            ("/home/b/db.sqlite".into(), "Notes".into()),
            ("/home/a b/x".into(), "Mail".into()),
            ("/home/ab/log".into(), "Safari".into()),
            ("/home/a/cache/0".into(), "Music".into()),
        ])
    }

    mod negative {
        use std::path::Path;

        use super::held;

        #[test]
        fn an_open_set_does_not_take_a_shared_prefix_for_inside() {
            let set = held();

            assert_eq!(set.holder_under(Path::new("/home/a/cach")), None);
            assert_eq!(set.holder_under(Path::new("/home/a/cache/0/deeper")), None);
            assert_eq!(set.holder_under(Path::new("/home/c")), None);
            assert_eq!(crate::OpenSet::default().holder_under(Path::new("/")), None);
            assert!(crate::OpenSet::default().is_empty());
        }
    }

    mod positive {
        use std::path::Path;

        use super::held;

        #[test]
        fn an_open_set_names_the_holder_of_an_item_inside() {
            let set = held();

            assert_eq!(set.holder_under(Path::new("/home/a")), Some("Music"));
            assert_eq!(set.holder_under(Path::new("/home/a/cache/0")), Some("Music"));
            assert_eq!(set.holder_under(Path::new("/home/ab")), Some("Safari"));
            assert_eq!(set.holder_under(Path::new("/home/a b")), Some("Mail"));
            assert_eq!(set.holder_under(Path::new("/")), Some("Music"));
            assert_eq!(set.len(), 4);
        }
    }

    mod property {
        use std::path::PathBuf;

        use proptest::prelude::*;

        use crate::OpenSet;

        fn path() -> impl Strategy<Value = PathBuf> {
            prop::collection::vec(prop_oneof![Just("a"), Just("ab"), Just("a b"), Just("b")], 0..4)
                .prop_map(|names| std::iter::once("/").chain(names).collect())
        }

        proptest! {
            #[test]
            fn the_holder_is_found_exactly_when_a_held_path_is_inside(
                held in prop::collection::vec(path(), 0..6),
                probe in path(),
            ) {
                let set = OpenSet::new(held.iter().map(|path| (path.clone(), "app".to_owned())).collect());

                prop_assert_eq!(set.holder_under(&probe).is_some(), held.iter().any(|path| path.starts_with(&probe)));
            }
        }
    }
}
