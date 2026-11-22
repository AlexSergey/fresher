//! What the cleaner never removes, decided by path components alone: no IO, no OS. Which places are protected on
//! one system is its adapter's to say; this only says what "protected" means for a path.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

/// The places a removal must keep away from.
///
/// - `never`: the place, anything inside it and every ancestor of it are refused;
/// - `except`: a place carved out of a `never` (`/usr/local` out of `/usr`): what is inside it may go, the place
///   itself never (as a whole folder);
/// - `whole`: the place and every ancestor of it are refused; what is inside may go one by one;
/// - the ignored paths ([`Protection::with_ignored`]): refused like `never`, with their own answer;
/// - the app folders ([`Protection::with_app_folders`]): `never` places that let through exactly a whole app bundle
///   at their top or one folder below, and nothing else;
/// - the refused places ([`Protection::with_refused`]): `never` places with no carve-out and no bundle opening.
///
/// Paths are compared by components, never by string prefix: `/Users/anna` is not inside `/Users/ann`. The caller
/// gives canonical paths, so a volume that ignores case is handled before this check. One exception: below an app
/// folder only the root of a plan item is made canonical, so a bundle is also refused when its names match, ignoring
/// case, a `never`, `whole` or ignored place inside that folder (`/Applications/fresher.app` against
/// `/Applications/Fresher.app`). Names are only lowered with `to_lowercase`: no Unicode case folding and no
/// normalisation is done, so a name the file system takes for the same but that differs by folding (`ſ` for `s`, a
/// ligature such as `ﬃ`) or by normal form (decomposed against composed) is not matched. The check stays lexical:
/// it never asks the disk what a name is (a link named `X.app` passes it). The walls are elsewhere: the uninstall
/// names bundles only from the system's listing of installed apps (canonical names, a real folder read with
/// `lstat`), never from what the interface sends, and checks each bundle's canonical path before it plans it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Protection {
    never: Vec<Never>,
    whole: Vec<PathBuf>,
    except: Vec<PathBuf>,
    ignored: Vec<PathBuf>,
    made: Made,
}

/// How a list was made: app folders open only the places of a list made alone, never of a joined one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Made {
    #[default]
    Alone,
    Joined,
}

/// A `never` place with the carve-outs of its own list and its kind: joined lists never let one list's carve-out or
/// bundle opening open another list's `never` place.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Never {
    place: PathBuf,
    except: Vec<PathBuf>,
    kind: NeverKind,
}

/// What a `never` place lets through besides its carve-outs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NeverKind {
    /// Nothing: a place of [`Protection::new`].
    Closed,
    /// Exactly a whole app bundle at its top or one folder below: an app folder.
    OpensBundles,
    /// Nothing, ever: a place of [`Protection::with_refused`], which no app folder opens.
    Refused,
}

/// Why a path is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Protected {
    /// A `never` place or inside one, outside its carve-outs; also any path that is not absolute and plain.
    Never,
    /// An ancestor of a `never` place.
    Ancestor,
    /// A `whole` place, an ancestor of one, or a carved-out place itself.
    WholeFolder,
    /// An ignored path, inside one, or an ancestor of one.
    Ignored,
}

/// Why a protection could not be made.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProtectionError {
    #[error("{path} is not an absolute path without `.` or `..`")]
    NotPlain { path: PathBuf },
    #[error("{path} is carved out of no protected place")]
    LooseExcept { path: PathBuf },
    #[error("{path} is an app folder but none of the list's own protected places")]
    LooseAppFolder { path: PathBuf },
    #[error("app folders are given to a list before it is joined with another")]
    AppFoldersOfAJoinedList,
}

impl Protection {
    /// Every path must be absolute and plain; every `except` strictly inside a `never`.
    pub fn new(never: Vec<PathBuf>, whole: Vec<PathBuf>, except: Vec<PathBuf>) -> Result<Self, ProtectionError> {
        check_plain(never.iter().chain(&whole).chain(&except))?;
        if let Some(loose) = except.iter().find(|place| !never.iter().any(|never| strictly_inside(place, never))) {
            return Err(ProtectionError::LooseExcept { path: loose.clone() });
        }
        let never = never
            .into_iter()
            .map(|place| {
                let carved = except.iter().filter(|cut| cut.starts_with(&place)).cloned().collect();
                Never { place, except: carved, kind: NeverKind::Closed }
            })
            .collect();
        Ok(Self { never, whole, except, ignored: Vec::new(), made: Made::Alone })
    }

    /// Both lists in one: every place either refuses stays refused, with the same answer as the graver of the two.
    /// A carve-out opens only the `never` places of its own list. Checked as [`Protection::new`] checks.
    pub fn joined(&self, other: &Self) -> Result<Self, ProtectionError> {
        let never: Vec<Never> = self.never.iter().chain(&other.never).cloned().collect();
        check_plain(never.iter().flat_map(|never| std::iter::once(&never.place).chain(&never.except)))?;
        check_plain(self.whole.iter().chain(&other.whole).chain(&self.ignored).chain(&other.ignored))?;
        let except: Vec<PathBuf> = self.except.iter().chain(&other.except).cloned().collect();
        let carved_from_its_own =
            |cut: &PathBuf| never.iter().any(|never| never.except.contains(cut) && strictly_inside(cut, &never.place));
        if let Some(loose) = except.iter().find(|cut| !carved_from_its_own(cut)) {
            return Err(ProtectionError::LooseExcept { path: loose.clone() });
        }
        let joined = |mine: &[PathBuf], theirs: &[PathBuf]| mine.iter().chain(theirs).cloned().collect();
        Ok(Self {
            never,
            whole: joined(&self.whole, &other.whole),
            except,
            ignored: joined(&self.ignored, &other.ignored),
            made: Made::Joined,
        })
    }

    /// The same protection, refusing also the user's `ignored` paths, what is inside them and their ancestors.
    pub fn with_ignored(&self, ignored: &[PathBuf]) -> Result<Self, ProtectionError> {
        check_plain(ignored)?;
        let mut protection = self.clone();
        protection.ignored.extend_from_slice(ignored);
        Ok(protection)
    }

    /// The same protection, where each of `folders` (each one of this list's own `never` places, given as plain)
    /// lets through exactly a whole app bundle at its top or one folder below: `<folder>/<name>.app` or
    /// `<folder>/<sub>/<name>.app`. The folder itself, any other folder in it and anything inside a bundle stay
    /// `never`; every other place keeps its answer. Given to a list made alone, before any
    /// [`Protection::joined`]: a joined list refuses it (`AppFoldersOfAJoinedList`), so one list never opens
    /// another's place.
    pub fn with_app_folders(&self, folders: &[PathBuf]) -> Result<Self, ProtectionError> {
        if self.made == Made::Joined {
            return Err(ProtectionError::AppFoldersOfAJoinedList);
        }
        check_plain(folders)?;
        let openable = |never: &Never, folder: &PathBuf| never.kind != NeverKind::Refused && &never.place == folder;
        if let Some(loose) = folders.iter().find(|folder| !self.never.iter().any(|never| openable(never, folder))) {
            return Err(ProtectionError::LooseAppFolder { path: loose.clone() });
        }
        let mut protection = self.clone();
        for never in &mut protection.never {
            if folders.iter().any(|folder| openable(never, folder)) {
                never.kind = NeverKind::OpensBundles;
            }
        }
        Ok(protection)
    }

    /// The places that open their app bundles ([`Protection::with_app_folders`]), in the list's order: an ordinary
    /// cleanup refuses them whole.
    pub fn app_folders(&self) -> Vec<PathBuf> {
        let opens = |never: &&Never| never.kind == NeverKind::OpensBundles;
        self.never.iter().filter(opens).map(|never| never.place.clone()).collect()
    }

    /// The same protection, refusing also each of `refused` (given as plain) as a `never` place with no carve-out and
    /// no bundle opening, now or after a later [`Protection::with_app_folders`]: the place and what is inside it are
    /// `never`, what holds it is an ancestor.
    pub fn with_refused(&self, refused: &[PathBuf]) -> Result<Self, ProtectionError> {
        check_plain(refused)?;
        let mut protection = self.clone();
        protection.never.extend(refused.iter().map(|place| Never {
            place: place.clone(),
            except: Vec::new(),
            kind: NeverKind::Refused,
        }));
        Ok(protection)
    }

    /// `Ok` when `path` may be removed as far as protection goes. The order: a `never` place (but a whole app bundle
    /// of a place that opens its bundles), an ancestor of one, a `whole` or carved-out place, an ignored path, and an
    /// opened bundle that matches a place inside its app folder only when case is ignored (`Never`).
    pub fn check(&self, path: &Path) -> Result<(), Protected> {
        if !is_plain(path) {
            return Err(Protected::Never);
        }
        let carved = |never: &Never| never.except.iter().any(|place| path.starts_with(place));
        let opened = |never: &Never| never.kind == NeverKind::OpensBundles && is_app_bundle_in(path, &never.place);
        if self.never.iter().any(|never| path.starts_with(&never.place) && !carved(never) && !opened(never)) {
            return Err(Protected::Never);
        }
        if self.never.iter().any(|never| strictly_inside(&never.place, path)) {
            return Err(Protected::Ancestor);
        }
        if self.whole.iter().chain(&self.except).any(|place| place.starts_with(path)) {
            return Err(Protected::WholeFolder);
        }
        if self.ignored.iter().any(|ignored| path.starts_with(ignored) || ignored.starts_with(path)) {
            return Err(Protected::Ignored);
        }
        // Last, so that a place named in the same case keeps its own answer.
        let mut opened_folders = self.never.iter().filter(|never| opened(never));
        if opened_folders.any(|never| self.matches_a_place_inside_ignoring_case(path, &never.place)) {
            return Err(Protected::Never);
        }
        Ok(())
    }

    /// `true` when `path`, ignoring case, is, holds or lies inside a `never`, `whole` or ignored place strictly
    /// inside `folder` (ignoring case too).
    fn matches_a_place_inside_ignoring_case(&self, path: &Path, folder: &Path) -> bool {
        let (path, folder) = (folded(path), folded(folder));
        let places = self.never.iter().map(|never| &never.place).chain(&self.whole).chain(&self.ignored);
        places.map(|place| folded(place)).any(|place| {
            place.len() > folder.len()
                && place.starts_with(&folder)
                && (path.starts_with(&place) || place.starts_with(&path))
        })
    }
}

/// The names of `path` in lower case, for comparing names that differ only by case.
fn folded(path: &Path) -> Vec<String> {
    path.components().map(|component| component.as_os_str().to_string_lossy().to_lowercase()).collect()
}

/// The extension of an app bundle, as macOS names its bundles.
const BUNDLE_EXTENSION: &str = "app";

/// `true` when the names after `folder` in `path` are exactly `[<name>.app]` or `[<sub>, <name>.app]`, every one
/// UTF-8: the last one's extension is `app` as written (an `.APP` is refused; a name that is only `.app` has no
/// extension), and `<sub>` does not end in `.app` in any case (a folder that may be a bundle, `.app` alone too,
/// opens nothing).
fn is_app_bundle_in(path: &Path, folder: &Path) -> bool {
    let Ok(below) = path.strip_prefix(folder) else { return false };
    let names: Option<Vec<&str>> = below
        .components()
        .map(|component| match component {
            Component::Normal(name) => name.to_str(),
            _ => None,
        })
        .collect();
    let is_bundle = |name: &str| Path::new(name).extension() == Some(OsStr::new(BUNDLE_EXTENSION));
    let may_be_bundle = |name: &str| name.to_lowercase().ends_with(&format!(".{BUNDLE_EXTENSION}"));
    match names.as_deref() {
        Some([bundle]) => is_bundle(bundle),
        Some([sub, bundle]) => !may_be_bundle(sub) && is_bundle(bundle),
        _ => false,
    }
}

/// `inner` is below `outer`, not `outer` itself, by components.
fn strictly_inside(inner: &Path, outer: &Path) -> bool {
    inner.starts_with(outer) && inner.components().count() > outer.components().count()
}

fn check_plain<'a>(paths: impl IntoIterator<Item = &'a PathBuf>) -> Result<(), ProtectionError> {
    match paths.into_iter().find(|path| !is_plain(path)) {
        Some(path) => Err(ProtectionError::NotPlain { path: path.clone() }),
        None => Ok(()),
    }
}

/// Absolute, with no `.` or `..` anywhere: `components()` drops a `.` inside a path, so the raw names are looked at.
pub(crate) fn is_plain(path: &Path) -> bool {
    let bytes = path.as_os_str().as_encoded_bytes();
    let dotted =
        bytes.split(|byte| std::path::is_separator(char::from(*byte))).any(|name| name == b"." || name == b"..");
    path.is_absolute()
        && !dotted
        && path
            .components()
            .all(|component| matches!(component, Component::RootDir | Component::Prefix(_) | Component::Normal(_)))
}

#[cfg(test)]
#[path = "protection_tests.rs"]
mod tests;
