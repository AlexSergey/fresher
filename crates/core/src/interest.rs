//! What a module wants to see during the one shared walk, and what the walk delivers.

use std::path::{Component, Path, PathBuf};

use crate::{EntryFacts, ModuleId, Sizes, Timestamp};

/// One thing a module wants the walk to report.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Interest {
    pattern: Pattern,
    claim: Claim,
    siblings: Vec<String>,
}

/// Where the walk looks. Names are single file or folder names; the home path is relative to the home folder.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// Any folder with this name (`node_modules`).
    FolderName(String),
    /// A path under the home folder, `*` matching one name (`Library/Caches/*`).
    UnderHome(String),
    /// A folder holding a file with this name (`CACHEDIR.TAG`).
    MarkerFile(String),
    /// Files with this extension, without the dot (`dmg`).
    Extension(String),
}

/// What the module claims of a matched entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Claim {
    /// The entry and everything inside it: summed, not indexed further.
    Whole,
    /// Just this entry; the walk goes on inside it.
    Item,
}

/// Why an interest was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum InterestError {
    #[error("an interest names nothing")]
    EmptyName,
    #[error("`{name}` is not a single file or folder name")]
    NotAName { name: String },
    #[error("`{path}` is not a relative path inside the home folder")]
    OutsideHome { path: String },
}

impl Interest {
    /// An interest in `pattern`; `siblings` are names whose presence next to a matched entry the walk reports
    /// (`package.json` next to `node_modules`).
    pub fn new(pattern: Pattern, claim: Claim, siblings: Vec<String>) -> Result<Self, InterestError> {
        match &pattern {
            Pattern::FolderName(name) | Pattern::MarkerFile(name) | Pattern::Extension(name) => check_name(name)?,
            Pattern::UnderHome(path) => check_home_path(path)?,
        }
        for sibling in &siblings {
            check_name(sibling)?;
        }
        Ok(Self { pattern, claim, siblings })
    }

    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub const fn claim(&self) -> Claim {
        self.claim
    }

    pub fn siblings(&self) -> &[String] {
        &self.siblings
    }
}

fn check_name(name: &str) -> Result<(), InterestError> {
    if name.is_empty() {
        return Err(InterestError::EmptyName);
    }
    if name == "." || name == ".." || name.contains(['/', '\0']) {
        return Err(InterestError::NotAName { name: name.into() });
    }
    Ok(())
}

fn check_home_path(path: &str) -> Result<(), InterestError> {
    if path.is_empty() {
        return Err(InterestError::EmptyName);
    }
    let inside = !path.contains('\0') && Path::new(path).components().all(|part| matches!(part, Component::Normal(_)));
    if inside { Ok(()) } else { Err(InterestError::OutsideHome { path: path.into() }) }
}

/// Every module's interests, each with its owner; the walk gets one set for all modules.
#[derive(Clone, Debug, Default)]
pub struct InterestSet {
    entries: Vec<(ModuleId, usize, Interest)>,
}

impl InterestSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the interests of `owner`, in order.
    pub fn add(&mut self, owner: ModuleId, interests: &[Interest]) {
        let added = interests.iter().cloned().enumerate().map(|(local, interest)| (owner, local, interest));
        self.entries.extend(added);
    }

    /// The interest at `index` in the set.
    pub fn get(&self, index: usize) -> Option<&Interest> {
        self.entries.get(index).map(|(_, _, interest)| interest)
    }

    /// The module that owns the interest at `index`, and the interest's index in that module's own `interests()`.
    pub fn owner(&self, index: usize) -> Option<(ModuleId, usize)> {
        self.entries.get(index).map(|&(owner, local, _)| (owner, local))
    }

    /// Every interest with its index in the set.
    pub fn iter(&self) -> impl Iterator<Item = (usize, &Interest)> {
        self.entries.iter().map(|(_, _, interest)| interest).enumerate()
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// What kind of entry the walk matched.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    File,
    Folder,
    /// A symbolic link: reported, never followed.
    Symlink,
}

/// An entry the walk matched against one interest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// The index of the interest: in the `InterestSet` as the walk reports it, in the module's own `interests()` as
    /// the engine passes it to the module.
    pub interest: usize,
    pub path: PathBuf,
    pub kind: EntryKind,
    /// The entry's size; for a `Claim::Whole` folder, everything inside it.
    pub sizes: Sizes,
    pub modified: Option<Timestamp>,
    /// Which of the interest's sibling names exist next to the entry.
    pub siblings: Vec<String>,
    /// For a `Pattern::MarkerFile` interest named `CACHEDIR.TAG`, whether the tag holds the signature; `None` for
    /// every other interest.
    pub marker: Option<MarkerCheck>,
    /// What the walk read about the entry, for its removability; the default when it could not tell.
    pub facts: EntryFacts,
}

/// What the walk found in a marker file whose content it checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MarkerCheck {
    /// A regular file with the expected content.
    Valid,
    /// Not a regular file, unreadable, or not the expected content.
    Invalid,
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{Claim, Interest, InterestError, Pattern};

        #[test]
        fn rejects_an_interest_with_an_empty_name() {
            for pattern in [
                Pattern::FolderName(String::new()),
                Pattern::MarkerFile(String::new()),
                Pattern::Extension(String::new()),
                Pattern::UnderHome(String::new()),
            ] {
                assert_eq!(Interest::new(pattern, Claim::Whole, vec![]), Err(InterestError::EmptyName));
            }
            let empty_sibling =
                Interest::new(Pattern::FolderName("node_modules".into()), Claim::Whole, vec![String::new()]);
            assert_eq!(empty_sibling, Err(InterestError::EmptyName));
        }

        #[test]
        fn rejects_a_name_that_is_a_path() {
            for name in ["a/b", ".", "..", "a\0b"] {
                let interest = Interest::new(Pattern::FolderName(name.into()), Claim::Item, vec![]);
                assert_eq!(interest, Err(InterestError::NotAName { name: name.into() }), "{name:?}");
            }
        }

        #[test]
        fn rejects_an_under_home_path_that_climbs_out() {
            for path in ["../Library", "Library/../../etc", "/Library/Caches", "./Library", "Library\0"] {
                let interest = Interest::new(Pattern::UnderHome(path.into()), Claim::Whole, vec![]);
                assert_eq!(interest, Err(InterestError::OutsideHome { path: path.into() }), "{path:?}");
            }
        }

        #[test]
        fn an_index_outside_the_set_has_no_owner() {
            let set = crate::InterestSet::new();

            assert!(set.is_empty());
            assert_eq!(set.owner(0), None);
            assert_eq!(set.get(0), None);
        }
    }

    mod positive {
        use crate::{Claim, Interest, InterestSet, ModuleId, Pattern};

        fn folder(name: &str) -> Interest {
            Interest::new(Pattern::FolderName(name.into()), Claim::Whole, vec!["package.json".into()]).unwrap()
        }

        #[test]
        fn accepts_a_valid_interest() {
            let interest = Interest::new(Pattern::UnderHome("Library/Caches/*".into()), Claim::Item, vec![]).unwrap();

            assert_eq!(interest.pattern(), &Pattern::UnderHome("Library/Caches/*".into()));
            assert_eq!(interest.claim(), Claim::Item);
            assert_eq!(interest.siblings(), Vec::<String>::new());
        }

        #[test]
        fn interest_set_remembers_the_owner_of_each_interest() {
            let demo = ModuleId::new("demo").unwrap();
            let node = ModuleId::new("node-modules").unwrap();
            let mut set = InterestSet::new();

            set.add(demo, &[folder("a"), folder("b")]);
            set.add(node, &[folder("node_modules")]);

            assert_eq!(set.len(), 3);
            assert_eq!(set.owner(0), Some((demo, 0)));
            assert_eq!(set.owner(1), Some((demo, 1)));
            assert_eq!(set.owner(2), Some((node, 0)));
            assert_eq!(set.get(2), Some(&folder("node_modules")));
            assert_eq!(set.iter().map(|(index, _)| index).collect::<Vec<_>>(), [0, 1, 2]);
        }
    }
}
