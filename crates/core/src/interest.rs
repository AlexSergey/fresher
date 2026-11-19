//! What a module wants to see during the one shared walk, and what the walk delivers.

use std::path::{Component, Path, PathBuf};

use crate::{EntryFacts, Inside, KnownFolder, ModuleId, Sizes, Timestamp};

/// One thing a module wants the walk to report.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Interest {
    pattern: Pattern,
    claim: Claim,
    siblings: Vec<String>,
    holds: Vec<String>,
    only_bundle_ids: bool,
    suffixes: Vec<String>,
}

/// Where the walk looks. Names are single file or folder names; the path of `Under` is relative to its known folder.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Pattern {
    /// Any folder with this name (`node_modules`).
    FolderName(String),
    /// A path under a known folder, `*` matching one name (`AppCaches` and `JetBrains/*`). The folder is resolved
    /// through the `KnownFolders` port, so a module never names an OS path.
    Under(KnownFolder, String),
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
    #[error("`{path}` is not a relative path inside its known folder")]
    OutsideFolder { path: String },
    #[error("only an interest anchored under a known folder may name what its folder holds")]
    HoldsNotAnchored,
    #[error("an interest names {count} entries its folder holds; at most {MAX_HOLDS}")]
    TooManyHolds { count: usize },
    #[error("only an interest anchored under a known folder whose path ends in `*` may match only some names")]
    NoStarToShape,
    #[error(
        "`{suffix}` is not a suffix an interest may match: a dot and a name, each part between dots not empty, \
             listed once"
    )]
    BadSuffix { suffix: String },
    #[error("an interest's last `*` matches either bundle ids or names with a suffix, not both")]
    TwoShapes,
}

/// The most names an interest may look for in its matched folder: a hit carries them as a 64-bit mask.
const MAX_HOLDS: usize = 64;

impl Interest {
    /// An interest in `pattern`; `siblings` are names whose presence next to a matched entry the walk reports
    /// (`package.json` next to `node_modules`).
    pub fn new(pattern: Pattern, claim: Claim, siblings: Vec<String>) -> Result<Self, InterestError> {
        match &pattern {
            Pattern::FolderName(name) | Pattern::MarkerFile(name) | Pattern::Extension(name) => check_name(name)?,
            Pattern::Under(_, path) => check_relative_path(path)?,
        }
        for sibling in &siblings {
            check_name(sibling)?;
        }
        Ok(Self { pattern, claim, siblings, holds: Vec::new(), only_bundle_ids: false, suffixes: Vec::new() })
    }

    /// This interest, with `names` the walk looks for in the matched folder's own listing (`index-v5`,
    /// `CACHEDIR.TAG`): at most 64 single names, only for a `Pattern::Under` interest. They are reported in
    /// [`Match::holds`]; a `CACHEDIR.TAG` among them is also checked for its signature ([`Match::marker`]).
    pub fn holding(self, names: Vec<String>) -> Result<Self, InterestError> {
        if !matches!(self.pattern, Pattern::Under(..)) {
            return Err(InterestError::HoldsNotAnchored);
        }
        if names.len() > MAX_HOLDS {
            return Err(InterestError::TooManyHolds { count: names.len() });
        }
        for name in &names {
            check_name(name)?;
        }
        Ok(Self { holds: names, ..self })
    }

    /// This interest, its pattern's last `*` matching only a name shaped as a bundle id
    /// ([`is_bundle_id_shaped`](crate::is_bundle_id_shaped)): only for a `Pattern::Under` whose last segment is `*`.
    /// A whole claim of `Caches/*` would otherwise swallow the places other modules claim (`Caches/JetBrains`).
    pub fn only_bundle_ids(self) -> Result<Self, InterestError> {
        self.check_star_to_shape()?;
        if !self.suffixes.is_empty() {
            return Err(InterestError::TwoShapes);
        }
        Ok(Self { only_bundle_ids: true, ..self })
    }

    /// This interest, its pattern's last `*` matching only a name that ends with one of `suffixes` and is longer than
    /// it, ASCII case ignored (`.download` takes `movie.download` and `Movie.DOWNLOAD`, not `.download` itself): only
    /// for a `Pattern::Under` whose last segment is `*`. A whole claim of `Downloads/*` would otherwise make every
    /// folder of the user's there opaque to the storage map and to every other module; a plain `*` of the same place
    /// takes the name too. Each suffix is a dot and a name whose parts between dots are not empty (`.app`,
    /// `.tar.gz`), listed once; they are kept lowercased. A second call replaces the list.
    pub fn only_suffixes(self, suffixes: Vec<String>) -> Result<Self, InterestError> {
        self.check_star_to_shape()?;
        if self.only_bundle_ids {
            return Err(InterestError::TwoShapes);
        }
        if suffixes.is_empty() {
            return Err(InterestError::BadSuffix { suffix: String::new() });
        }
        let mut lowered: Vec<String> = Vec::with_capacity(suffixes.len());
        for suffix in suffixes {
            let lower = suffix.to_ascii_lowercase();
            if !is_suffix(&suffix) || lowered.contains(&lower) {
                return Err(InterestError::BadSuffix { suffix });
            }
            lowered.push(lower);
        }
        Ok(Self { suffixes: lowered, ..self })
    }

    fn check_star_to_shape(&self) -> Result<(), InterestError> {
        let Pattern::Under(_, path) = &self.pattern else { return Err(InterestError::NoStarToShape) };
        if path.rsplit('/').next() != Some("*") {
            return Err(InterestError::NoStarToShape);
        }
        Ok(())
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

    /// The names the walk looks for in the matched folder's own listing; empty for most interests.
    pub fn holds(&self) -> &[String] {
        &self.holds
    }

    /// Whether the pattern's last `*` matches only a name shaped as a bundle id.
    pub const fn matches_only_bundle_ids(&self) -> bool {
        self.only_bundle_ids
    }

    /// The suffixes, lowercased, one of which a name must end with for the pattern's last `*` to match it; empty when
    /// the `*` is not shaped so.
    pub fn name_suffixes(&self) -> &[String] {
        &self.suffixes
    }
}

/// Whether `suffix` is a dot followed by parts that are not empty, joined by dots, none holding `/` or `\0`.
fn is_suffix(suffix: &str) -> bool {
    suffix
        .strip_prefix('.')
        .is_some_and(|rest| !rest.contains(['/', '\0']) && rest.split('.').all(|part| !part.is_empty()))
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

fn check_relative_path(path: &str) -> Result<(), InterestError> {
    if path.is_empty() {
        return Err(InterestError::EmptyName);
    }
    let inside = !path.contains('\0') && Path::new(path).components().all(|part| matches!(part, Component::Normal(_)));
    if inside { Ok(()) } else { Err(InterestError::OutsideFolder { path: path.into() }) }
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
    /// A socket, a named pipe or a device: reported as itself, never read, never a marker.
    Other,
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
    /// The entry's own last status change.
    pub changed: Option<Timestamp>,
    /// The entry's own last access; `None` for a folder, whose access time moves when the walk lists it.
    pub accessed: Option<Timestamp>,
    /// What a `Claim::Whole` folder holds, gathered while the walk summed it; `None` for anything else.
    ///
    /// `changed`, `accessed` and `inside` are read from the disk on every walk inside a walked place (`UserTemp`,
    /// `SharedTemp`), which is always read again in full. Elsewhere a rescan copies an unchanged folder's values from
    /// the cache, so they can be older than the disk's.
    pub inside: Option<Inside>,
    /// Which of the interest's sibling names exist next to the entry.
    pub siblings: Vec<String>,
    /// Which of the interest's `holds` names the matched folder itself holds, in the interest's order; empty for a
    /// file, a link, an `Other` entry, a folder not entered or not read (another volume, another user's, not
    /// scanned, unreadable) and an interest that names none. An entry that could not be read is absent; any kind
    /// counts, but a `CACHEDIR.TAG` only as a regular file.
    pub holds: Vec<String>,
    /// For a `Pattern::MarkerFile` interest named `CACHEDIR.TAG`, whether the tag holds the signature. For an
    /// interest whose `holds` names `CACHEDIR.TAG` and whose folder's listing was read, whether the folder holds a
    /// valid one: `Invalid` when it is absent, not a regular file, unreadable or without the signature. `None` for
    /// every other interest, and when the folder's listing was not read.
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
        use crate::{Claim, Interest, InterestError, KnownFolder, Pattern};

        #[test]
        fn rejects_an_interest_with_an_empty_name() {
            for pattern in [
                Pattern::FolderName(String::new()),
                Pattern::MarkerFile(String::new()),
                Pattern::Extension(String::new()),
                Pattern::Under(KnownFolder::AppCaches, String::new()),
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
        fn rejects_a_path_that_climbs_out_of_its_folder() {
            for path in ["../Library", "Library/../../etc", "/Library/Caches", "./Library", "Library\0"] {
                let interest = Interest::new(Pattern::Under(KnownFolder::Home, path.into()), Claim::Whole, vec![]);
                assert_eq!(interest, Err(InterestError::OutsideFolder { path: path.into() }), "{path:?}");
            }
        }

        fn anchored() -> Interest {
            Interest::new(Pattern::Under(KnownFolder::Home, ".npm/_cacache".into()), Claim::Whole, vec![]).unwrap()
        }

        #[test]
        fn holding_names_is_refused_for_an_unanchored_interest() {
            for pattern in [
                Pattern::FolderName("caches".into()),
                Pattern::MarkerFile("CACHEDIR.TAG".into()),
                Pattern::Extension("dmg".into()),
            ] {
                let interest = Interest::new(pattern.clone(), Claim::Whole, vec![]).unwrap();
                assert_eq!(interest.holding(vec!["a".into()]), Err(InterestError::HoldsNotAnchored), "{pattern:?}");
            }
        }

        #[test]
        fn a_held_name_that_is_not_a_single_name_is_refused() {
            assert_eq!(anchored().holding(vec![String::new()]), Err(InterestError::EmptyName));
            for name in ["a/b", ".", "..", "a\0b"] {
                let held = anchored().holding(vec!["index-v5".into(), name.into()]);
                assert_eq!(held, Err(InterestError::NotAName { name: name.into() }), "{name:?}");
            }
        }

        #[test]
        fn more_than_64_held_names_are_refused() {
            let names = (0..65).map(|index| format!("name{index}")).collect();
            assert_eq!(anchored().holding(names), Err(InterestError::TooManyHolds { count: 65 }));
        }

        #[test]
        fn only_bundle_ids_is_refused_without_a_last_star_to_shape() {
            for pattern in [
                Pattern::FolderName("Caches".into()),
                Pattern::MarkerFile("CACHEDIR.TAG".into()),
                Pattern::Extension("plist".into()),
                Pattern::Under(KnownFolder::AppCaches, "a".into()),
                Pattern::Under(KnownFolder::AppCaches, "*/a".into()),
                Pattern::Under(KnownFolder::AppCaches, "a*".into()),
            ] {
                let interest = Interest::new(pattern.clone(), Claim::Whole, vec![]).unwrap();
                assert_eq!(interest.only_bundle_ids(), Err(InterestError::NoStarToShape), "{pattern:?}");
            }
        }

        #[test]
        fn a_plain_interest_matches_more_than_bundle_ids() {
            let pattern = Pattern::Under(KnownFolder::AppCaches, "*".into());
            assert!(!Interest::new(pattern, Claim::Whole, vec![]).unwrap().matches_only_bundle_ids());
        }

        fn downloads(path: &str) -> Interest {
            Interest::new(Pattern::Under(KnownFolder::Downloads, path.into()), Claim::Whole, vec![]).unwrap()
        }

        fn suffixes(list: &[&str]) -> Vec<String> {
            list.iter().map(|&suffix| suffix.into()).collect()
        }

        #[test]
        fn only_suffixes_refuses_a_pattern_without_a_last_star() {
            let folder = Interest::new(Pattern::FolderName("Downloads".into()), Claim::Whole, vec![]).unwrap();
            for interest in [folder, downloads("a/b")] {
                assert_eq!(interest.only_suffixes(suffixes(&[".download"])), Err(InterestError::NoStarToShape));
            }
        }

        #[test]
        fn only_suffixes_refuses_an_empty_list_and_a_bad_suffix() {
            assert_eq!(downloads("*").only_suffixes(vec![]), Err(InterestError::BadSuffix { suffix: String::new() }));
            for bad in ["", "download", ".", ".a/b", ".a..b", ".a\0b", ".a.", ".app/"] {
                let shaped = downloads("*").only_suffixes(suffixes(&[".app", bad]));
                assert_eq!(shaped, Err(InterestError::BadSuffix { suffix: bad.into() }), "{bad:?}");
            }
        }

        #[test]
        fn only_suffixes_refuses_a_suffix_listed_twice() {
            let twice = downloads("*").only_suffixes(suffixes(&[".app", ".APP"]));
            assert_eq!(twice, Err(InterestError::BadSuffix { suffix: ".APP".into() }));
        }

        #[test]
        fn an_interest_takes_only_one_shape() {
            let suffixed = downloads("*").only_suffixes(suffixes(&[".app"])).unwrap();
            assert_eq!(suffixed.only_bundle_ids(), Err(InterestError::TwoShapes));
            let shaped = downloads("*").only_bundle_ids().unwrap();
            assert_eq!(shaped.only_suffixes(suffixes(&[".app"])), Err(InterestError::TwoShapes));
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
        use crate::{Claim, Interest, InterestSet, KnownFolder, ModuleId, Pattern};

        fn folder(name: &str) -> Interest {
            Interest::new(Pattern::FolderName(name.into()), Claim::Whole, vec!["package.json".into()]).unwrap()
        }

        #[test]
        fn accepts_a_valid_interest() {
            let pattern = Pattern::Under(KnownFolder::AppCaches, "JetBrains/*".into());
            let interest = Interest::new(pattern.clone(), Claim::Item, vec![]).unwrap();

            assert_eq!(interest.pattern(), &pattern);
            assert_eq!(interest.claim(), Claim::Item);
            assert_eq!(interest.siblings(), Vec::<String>::new());
            assert_eq!(interest.holds(), Vec::<String>::new());
        }

        #[test]
        fn an_anchored_interest_holds_up_to_64_names_in_order() {
            let pattern = Pattern::Under(KnownFolder::Home, ".npm/_cacache".into());
            let interest = Interest::new(pattern, Claim::Whole, vec![]).unwrap();
            let names: Vec<String> = (0..64).rev().map(|index| format!("name{index}")).collect();

            let held = interest.clone().holding(names.clone()).unwrap();

            assert_eq!(held.holds(), names);
            assert_eq!((held.pattern(), held.claim()), (interest.pattern(), interest.claim()));
            assert_ne!(held, interest, "the held names are part of the interest");
        }

        #[test]
        fn a_last_star_is_shaped_to_match_only_bundle_ids() {
            for path in ["*", "HTTPStorages/*"] {
                let plain = Interest::new(Pattern::Under(KnownFolder::Library, path.into()), Claim::Whole, vec![])
                    .unwrap()
                    .holding(vec!["a".into()])
                    .unwrap();

                let shaped = plain.clone().only_bundle_ids().unwrap();

                assert!(shaped.matches_only_bundle_ids(), "{path}");
                assert_eq!((shaped.pattern(), shaped.holds()), (plain.pattern(), plain.holds()), "{path}");
                assert_ne!(shaped, plain, "the shape is part of the interest");
            }
        }

        #[test]
        fn only_suffixes_keeps_its_suffixes_lowercased() {
            let plain = Interest::new(Pattern::Under(KnownFolder::Downloads, "*".into()), Claim::Whole, vec![])
                .unwrap()
                .holding(vec!["Contents".into()])
                .unwrap();

            let suffixed = plain.clone().only_suffixes(vec![".Download".into(), ".tar.gz".into()]).unwrap();

            assert_eq!(suffixed.name_suffixes(), [".download", ".tar.gz"]);
            assert!(!suffixed.matches_only_bundle_ids());
            assert_eq!((suffixed.pattern(), suffixed.holds()), (plain.pattern(), plain.holds()));
            assert_eq!(plain.name_suffixes(), &[] as &[String]);
            assert_ne!(suffixed, plain, "the suffixes are part of the interest");
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
