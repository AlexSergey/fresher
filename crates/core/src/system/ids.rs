//! Who an app is: its bundle id and its developer's team id. Never a loose name.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::{Serialize, Serializer};

/// The longest bundle id taken.
const MAX_BUNDLE_ID: usize = 255;
/// The length of a team id.
const TEAM_ID: usize = 10;

/// Why an id was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppIdError {
    #[error("`{id}` is not a bundle id")]
    NotABundleId { id: String },
    #[error("`{id}` is not a team id")]
    NotATeamId { id: String },
}

/// An app's bundle id (`com.example.App`): ASCII letters, digits, `.`, `-` and `_`, not starting or ending with a dot.
/// Two ids that differ only in case are the same id, as the system treats them.
#[derive(Clone, Debug)]
pub struct BundleId {
    id: String,
    folded: String,
}

impl BundleId {
    pub fn new(id: impl Into<String>) -> Result<Self, AppIdError> {
        let id = id.into();
        let allowed = |char: char| char.is_ascii_alphanumeric() || matches!(char, '.' | '-' | '_');
        let valid = !id.is_empty()
            && id.len() <= MAX_BUNDLE_ID
            && id.chars().all(allowed)
            && !id.starts_with('.')
            && !id.ends_with('.')
            && !id.contains("..");
        if !valid {
            return Err(AppIdError::NotABundleId { id });
        }
        let folded = id.to_ascii_lowercase();
        Ok(Self { id, folded })
    }

    /// The id as the app writes it.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    /// The id in lower case: what two ids are compared by.
    pub fn folded(&self) -> &str {
        &self.folded
    }

    /// Whether `name` (a folder's or a file's) is named after this id: the id as the whole name, or followed by a dot
    /// and more (`com.example.App.savedState`), case ignored. `com.example.Appx` is not named after `com.example.App`.
    pub fn names(&self, name: &str) -> bool {
        let Some((head, rest)) = name.split_at_checked(self.folded.len()) else { return false };
        head.eq_ignore_ascii_case(&self.folded) && (rest.is_empty() || rest.starts_with('.'))
    }
}

impl PartialEq for BundleId {
    fn eq(&self, other: &Self) -> bool {
        self.folded == other.folded
    }
}

impl Eq for BundleId {}

impl Hash for BundleId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.folded.hash(state);
    }
}

impl PartialOrd for BundleId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BundleId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.folded.cmp(&other.folded)
    }
}

/// Serialized as the id the app writes.
impl Serialize for BundleId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.id)
    }
}

impl fmt::Display for BundleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.id)
    }
}

/// The team id of the developer who signed an app: ten ASCII capitals and digits.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TeamId(String);

impl TeamId {
    pub fn new(id: impl Into<String>) -> Result<Self, AppIdError> {
        let id = id.into();
        let valid = id.len() == TEAM_ID && id.chars().all(|char| char.is_ascii_uppercase() || char.is_ascii_digit());
        if valid { Ok(Self(id)) } else { Err(AppIdError::NotATeamId { id }) }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TeamId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use crate::{AppIdError, BundleId, TeamId};

        #[test]
        fn a_bundle_id_that_is_not_reverse_dns_is_refused() {
            let long = "a".repeat(256);
            for id in ["", "com example", "com/example", ".com.example", "com.example.", "com..example", "ünï", &long]
            {
                assert_eq!(BundleId::new(id), Err(AppIdError::NotABundleId { id: id.into() }), "{id}");
            }
        }

        #[test]
        fn a_team_id_that_is_not_ten_capitals_and_digits_is_refused() {
            for id in ["", "ABCDE1234", "ABCDE123456", "abcde12345", "ABCDE-1234"] {
                assert_eq!(TeamId::new(id), Err(AppIdError::NotATeamId { id: id.into() }), "{id}");
            }
        }

        #[test]
        fn a_name_that_only_starts_like_the_id_is_not_named_after_it() {
            let id = BundleId::new("com.example.App").unwrap();

            for name in
                ["com.example.Appx", "com.example.Ap", "x.com.example.App", "com.example", "", "com.exämple.App"]
            {
                assert!(!id.names(name), "{name}");
            }
        }

        #[test]
        fn different_bundle_ids_differ() {
            assert_ne!(BundleId::new("com.example.App").unwrap(), BundleId::new("com.example.Appx").unwrap());
        }
    }

    mod positive {
        use std::collections::HashSet;

        use crate::{BundleId, TeamId};

        #[test]
        fn bundle_ids_compare_ignoring_case_and_keep_their_spelling() {
            let written = BundleId::new("com.Example.App-2_b").unwrap();
            let lower = BundleId::new("com.example.app-2_b").unwrap();

            assert_eq!(written, lower);
            assert_eq!(written.cmp(&lower), std::cmp::Ordering::Equal);
            assert_eq!(written.partial_cmp(&lower), Some(std::cmp::Ordering::Equal));
            assert_eq!(HashSet::from([written.clone(), lower]).len(), 1);
            assert_eq!(written.as_str(), "com.Example.App-2_b");
            assert_eq!(written.folded(), "com.example.app-2_b");
            assert_eq!(written.to_string(), "com.Example.App-2_b");
        }

        #[test]
        fn a_name_is_named_after_the_id_whole_or_followed_by_a_dot() {
            let id = BundleId::new("com.example.App").unwrap();

            for name in ["com.example.App", "COM.EXAMPLE.APP", "com.example.App.plist", "com.example.app.savedState"] {
                assert!(id.names(name), "{name}");
            }
        }

        #[test]
        fn a_team_id_keeps_its_ten_characters() {
            let team = TeamId::new("ABCDE12345").unwrap();

            assert_eq!(team.as_str(), "ABCDE12345");
            assert_eq!(team.to_string(), "ABCDE12345");
        }
    }
}
