//! What the cleaner never removes, decided by path components alone: no IO, no OS. Which places are protected on
//! one system is its adapter's to say; this only says what "protected" means for a path.

use std::path::{Component, Path, PathBuf};

/// The places a removal must keep away from.
///
/// - `never`: the place, anything inside it and every ancestor of it are refused;
/// - `except`: a place carved out of a `never` (`/usr/local` out of `/usr`): what is inside it may go, the place
///   itself never (as a whole folder);
/// - `whole`: the place and every ancestor of it are refused; what is inside may go one by one;
/// - the ignored paths ([`Protection::with_ignored`]): refused like `never`, with their own answer.
///
/// Paths are compared by components, never by string prefix: `/Users/anna` is not inside `/Users/ann`. The caller
/// gives canonical paths, so a volume that ignores case is handled before this check.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Protection {
    never: Vec<Never>,
    whole: Vec<PathBuf>,
    except: Vec<PathBuf>,
    ignored: Vec<PathBuf>,
}

/// A `never` place with the carve-outs of its own list: joined lists never let one list's carve-out open another
/// list's `never` place.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Never {
    place: PathBuf,
    except: Vec<PathBuf>,
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
                Never { place, except: carved }
            })
            .collect();
        Ok(Self { never, whole, except, ignored: Vec::new() })
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
        })
    }

    /// The same protection, refusing also the user's `ignored` paths, what is inside them and their ancestors.
    pub fn with_ignored(&self, ignored: &[PathBuf]) -> Result<Self, ProtectionError> {
        check_plain(ignored)?;
        let mut protection = self.clone();
        protection.ignored.extend_from_slice(ignored);
        Ok(protection)
    }

    /// `Ok` when `path` may be removed as far as protection goes. The order: a `never` place, an ancestor of one, a
    /// `whole` or carved-out place, an ignored path.
    pub fn check(&self, path: &Path) -> Result<(), Protected> {
        if !is_plain(path) {
            return Err(Protected::Never);
        }
        let carved = |never: &Never| never.except.iter().any(|place| path.starts_with(place));
        if self.never.iter().any(|never| path.starts_with(&never.place) && !carved(never)) {
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
        Ok(())
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
fn is_plain(path: &Path) -> bool {
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
mod tests {
    use std::path::PathBuf;

    use crate::cleanup::Protection;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    /// A small version of the macOS list: system places, the synced folders, the home and its own folders.
    fn mac_like() -> Protection {
        Protection::new(
            paths(&["/System", "/usr", "/Users/ann/Library/Mobile Documents", "/Users/bob"]),
            paths(&["/Users", "/Users/ann", "/Users/ann/Library", "/Users/ann/Library/Caches"]),
            paths(&["/usr/local"]),
        )
        .unwrap()
    }

    mod negative {
        use std::path::Path;

        use super::{mac_like, paths};
        use crate::cleanup::{Protected, Protection, ProtectionError};

        #[test]
        fn a_protected_path_is_refused() {
            assert_eq!(mac_like().check(Path::new("/System")), Err(Protected::Never));
            assert_eq!(mac_like().check(Path::new("/Users/bob")), Err(Protected::Never));
        }

        #[test]
        fn a_path_inside_a_protected_one_is_refused() {
            assert_eq!(mac_like().check(Path::new("/usr/bin/ls")), Err(Protected::Never));
            assert_eq!(
                mac_like().check(Path::new("/Users/ann/Library/Mobile Documents/com~apple~Notes")),
                Err(Protected::Never)
            );
        }

        #[test]
        fn an_ancestor_of_a_protected_path_is_refused() {
            for path in ["/", "/Users/ann/Library"] {
                assert!(mac_like().check(Path::new(path)).is_err(), "{path}");
            }
            let only_never = Protection::new(paths(&["/Users/ann/Library/Mobile Documents"]), vec![], vec![]).unwrap();
            for path in ["/", "/Users", "/Users/ann", "/Users/ann/Library"] {
                assert_eq!(only_never.check(Path::new(path)), Err(Protected::Ancestor), "{path}");
            }
        }

        #[test]
        fn a_whole_folder_is_refused_as_a_whole() {
            assert_eq!(mac_like().check(Path::new("/Users/ann/Library/Caches")), Err(Protected::WholeFolder));
            // The home is also an ancestor of a `never` place, which is checked first.
            assert_eq!(mac_like().check(Path::new("/Users/ann")), Err(Protected::Ancestor));
        }

        #[test]
        fn an_ancestor_of_a_whole_folder_is_refused() {
            let whole = Protection::new(vec![], paths(&["/Users/ann/Downloads"]), vec![]).unwrap();

            for path in ["/", "/Users", "/Users/ann"] {
                assert_eq!(whole.check(Path::new(path)), Err(Protected::WholeFolder), "{path}");
            }
        }

        #[test]
        fn a_carved_out_place_itself_is_refused() {
            assert_eq!(mac_like().check(Path::new("/usr/local")), Err(Protected::WholeFolder));
        }

        #[test]
        fn a_relative_or_dotted_path_is_not_accepted() {
            for path in ["usr", "/usr/./local", "/usr/local/..", "/a/../b", "./a"] {
                let error = Protection::new(paths(&[path]), vec![], vec![]);
                assert_eq!(error, Err(ProtectionError::NotPlain { path: path.into() }), "{path}");
                assert_eq!(Protection::default().check(Path::new(path)), Err(Protected::Never), "{path}");
                assert!(Protection::default().with_ignored(&paths(&[path])).is_err(), "{path}");
            }
        }

        #[test]
        fn an_except_outside_every_protected_place_is_refused() {
            let error = Protection::new(paths(&["/usr"]), vec![], paths(&["/opt/local"]));
            let itself = Protection::new(paths(&["/usr"]), vec![], paths(&["/usr"]));

            assert_eq!(error, Err(ProtectionError::LooseExcept { path: "/opt/local".into() }));
            assert_eq!(itself, Err(ProtectionError::LooseExcept { path: "/usr".into() }));
        }

        #[test]
        fn a_shared_prefix_is_not_inside() {
            let ann = Protection::new(paths(&["/Users/ann"]), vec![], vec![]).unwrap();

            assert_eq!(ann.check(Path::new("/Users/anna")), Ok(()));
            assert_eq!(ann.check(Path::new("/Users/anna/x")), Ok(()));
            assert_eq!(ann.check(Path::new("/Users/ann/x")), Err(Protected::Never));
        }

        #[test]
        fn an_ignored_path_and_its_ancestors_are_refused() {
            let ignoring = mac_like().with_ignored(&paths(&["/Users/ann/Library/Caches/keep"])).unwrap();

            assert_eq!(ignoring.check(Path::new("/Users/ann/Library/Caches/keep")), Err(Protected::Ignored));
            assert_eq!(ignoring.check(Path::new("/Users/ann/Library/Caches/keep/a")), Err(Protected::Ignored));
            assert_eq!(ignoring.check(Path::new("/Users/ann/Library/Caches/kept")), Ok(()));
            assert_eq!(mac_like().check(Path::new("/Users/ann/Library/Caches/keep")), Ok(()));
        }

        #[test]
        fn a_never_place_inside_a_carve_out_is_still_refused() {
            let nested = Protection::new(paths(&["/usr", "/usr/local/keep"]), vec![], paths(&["/usr/local"])).unwrap();

            assert_eq!(nested.check(Path::new("/usr/local/keep/x")), Err(Protected::Never));
            assert_eq!(nested.check(Path::new("/usr/local/other")), Ok(()));
        }

        #[test]
        fn a_carve_out_of_one_list_does_not_open_the_other() {
            let strict = Protection::new(paths(&["/usr"]), vec![], vec![]).unwrap();

            let joined = mac_like().joined(&strict).unwrap();

            assert_eq!(joined.check(Path::new("/usr/local/x")), Err(Protected::Never));
            assert_eq!(strict.joined(&mac_like()).unwrap().check(Path::new("/usr/local/x")), Err(Protected::Never));
            assert_eq!(joined.check(Path::new("/usr/local")), Err(Protected::Never));
        }
    }

    mod positive {
        use std::path::Path;

        use super::mac_like;

        #[test]
        fn an_item_inside_a_whole_folder_may_go() {
            assert_eq!(mac_like().check(Path::new("/Users/ann/Library/Caches/x")), Ok(()));
            assert_eq!(mac_like().check(Path::new("/Users/ann/Downloads")), Ok(()));
        }

        #[test]
        fn an_item_inside_a_carve_out_may_go() {
            assert_eq!(mac_like().check(Path::new("/usr/local/x")), Ok(()));
            assert_eq!(mac_like().check(Path::new("/usr/local/x/y")), Ok(()));
        }

        #[test]
        fn an_unrelated_path_passes() {
            assert_eq!(mac_like().check(Path::new("/opt/homebrew/cache")), Ok(()));
        }
    }

    mod property {
        use std::path::{Path, PathBuf};

        use proptest::prelude::*;

        use crate::cleanup::{Protected, Protection};

        /// A path of up to four names out of a few like names, so that equal, nested and shared-prefix paths come up.
        fn names() -> impl Strategy<Value = Vec<&'static str>> {
            prop::collection::vec(prop_oneof![Just("a"), Just("ab"), Just("b"), Just("a b")], 0..4)
        }

        fn path_of(names: &[&str]) -> PathBuf {
            std::iter::once("/").chain(names.iter().copied()).collect()
        }

        /// `inner` is `outer` or below it: the plain loop the model is written with.
        fn below(inner: &[&str], outer: &[&str]) -> bool {
            if inner.len() < outer.len() {
                return false;
            }
            for index in 0..outer.len() {
                if inner[index] != outer[index] {
                    return false;
                }
            }
            true
        }

        /// The rules of `Protection::check`, written over lists of names.
        fn model(
            path: &[&str],
            never: &[Vec<&str>],
            whole: &[Vec<&str>],
            except: &[Vec<&str>],
            ignored: &[Vec<&str>],
        ) -> Result<(), Protected> {
            for place in never {
                let mut carved = false;
                for cut in except {
                    if below(cut, place) && below(path, cut) {
                        carved = true;
                    }
                }
                if below(path, place) && !carved {
                    return Err(Protected::Never);
                }
            }
            for place in never {
                if below(place, path) && place.len() > path.len() {
                    return Err(Protected::Ancestor);
                }
            }
            for place in whole.iter().chain(except) {
                if below(place, path) {
                    return Err(Protected::WholeFolder);
                }
            }
            for place in ignored {
                if below(path, place) || below(place, path) {
                    return Err(Protected::Ignored);
                }
            }
            Ok(())
        }

        fn lists() -> impl Strategy<Value = Vec<Vec<&'static str>>> {
            prop::collection::vec(names(), 0..3)
        }

        /// A random list: `never`, `whole`, carve-outs strictly inside the first `never` place, and ignored paths.
        fn protection() -> impl Strategy<Value = Protection> {
            (lists(), lists(), lists(), lists()).prop_map(|(never, whole, cuts, ignored)| {
                let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
                let never: Vec<Vec<&str>> = never.into_iter().filter(|place| !place.is_empty()).collect();
                let except: Vec<PathBuf> = match never.first() {
                    Some(place) => {
                        cuts.iter().filter_map(|cut| cut.first().map(|name| path_of(place).join(name))).collect()
                    }
                    None => Vec::new(),
                };
                Protection::new(to_paths(&never), to_paths(&whole), except)
                    .unwrap()
                    .with_ignored(&to_paths(&ignored))
                    .unwrap()
            })
        }

        /// The order `check` answers in: the first answer is the gravest.
        fn gravity(answer: Result<(), Protected>) -> u8 {
            match answer {
                Err(Protected::Never) => 4,
                Err(Protected::Ancestor) => 3,
                Err(Protected::WholeFolder) => 2,
                Err(Protected::Ignored) => 1,
                Ok(()) => 0,
            }
        }

        proptest! {
            #[test]
            fn check_agrees_with_a_naive_component_model(
                path in names(), never in lists(), whole in lists(), cuts in lists(), ignored in lists(),
            ) {
                let never: Vec<Vec<&str>> = never.into_iter().filter(|place| !place.is_empty()).collect();
                // Only carve-outs strictly inside a `never` are accepted: each is the first one plus a name.
                let except: Vec<Vec<&str>> = match never.first() {
                    Some(place) => cuts
                        .into_iter()
                        .filter_map(|cut| cut.first().map(|name| place.iter().copied().chain([*name]).collect()))
                        .collect(),
                    None => Vec::new(),
                };
                let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
                let protection = Protection::new(to_paths(&never), to_paths(&whole), to_paths(&except))
                    .unwrap()
                    .with_ignored(&to_paths(&ignored))
                    .unwrap();

                let answer = protection.check(&path_of(&path));

                prop_assert_eq!(answer, model(&path, &never, &whole, &except, &ignored));
            }

            #[test]
            fn a_joined_list_refuses_what_either_refuses(
                path in names(), first in protection(), second in protection(),
            ) {
                let path = path_of(&path);

                let answer = first.joined(&second).unwrap().check(&path);

                let graver = [first.check(&path), second.check(&path)].into_iter().max_by_key(|answer| gravity(*answer));
                prop_assert_eq!(answer, graver.unwrap());
                if first.check(&path).is_err() || second.check(&path).is_err() {
                    prop_assert!(answer.is_err());
                }
            }

            #[test]
            fn an_ancestor_of_anything_never_passes(never in lists(), whole in lists(), ignored in lists()) {
                let to_paths = |list: &[Vec<&str>]| list.iter().map(|names| path_of(names)).collect::<Vec<_>>();
                let protection = Protection::new(to_paths(&never), to_paths(&whole), vec![])
                    .unwrap()
                    .with_ignored(&to_paths(&ignored))
                    .unwrap();

                for place in never.iter().chain(&whole).chain(&ignored) {
                    for depth in 0..=place.len() {
                        let ancestor = path_of(&place[..depth]);
                        prop_assert!(protection.check(&ancestor).is_err(), "{}", ancestor.display());
                    }
                }
                prop_assert!(protection.check(Path::new("relative")).is_err());
            }
        }
    }
}
