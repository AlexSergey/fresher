//! What is known of an app's bundle: its sizes, its entry facts, its last use and who installed it.

use crate::{EntryFacts, Removability, Sizes, Timestamp};

/// The longest package-manager token accepted.
const MAX_TOKEN_LEN: usize = 128;

/// Why an app's facts could not be made.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum AppsError {
    /// A package manager's token that is not 1 to 128 of `[a-z0-9@._+-]`.
    #[error("`{token}` is not a package manager's token")]
    Token { token: String },
}

/// A package manager that installs apps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Manager {
    Homebrew,
}

impl Manager {
    /// The name the manager calls itself, a proper name the sentences pass as it is.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Homebrew => "Homebrew",
        }
    }
}

/// The package manager that installed an app, and the name it knows the app by.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct InstalledBy {
    manager: Manager,
    token: String,
}

impl InstalledBy {
    /// `token` must be 1 to 128 of `[a-z0-9@._+-]`: it is shown in a command the user may copy.
    pub fn new(manager: Manager, token: impl Into<String>) -> Result<Self, AppsError> {
        let token = token.into();
        let valid = !token.is_empty()
            && token.len() <= MAX_TOKEN_LEN
            && token.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"@._+-".contains(&byte));
        if valid { Ok(Self { manager, token }) } else { Err(AppsError::Token { token }) }
    }

    pub const fn manager(&self) -> Manager {
        self.manager
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    /// The command that uninstalls the app with its manager, for the user to copy; Fresher never runs it.
    pub fn command(&self) -> String {
        match self.manager {
            Manager::Homebrew => format!("brew uninstall --cask {}", self.token),
        }
    }
}

/// The facts of an app's bundle; read from metadata only, never from a file's contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleFacts {
    /// What the bundle takes: hard links counted once, no link followed.
    pub sizes: Sizes,
    /// The bundle's own entry: its owner, its folder's and its own write bits for the user, its flags.
    pub entry: EntryFacts,
    /// When the system last saw the app used, if it knows.
    pub last_used: Option<Timestamp>,
    /// The package manager that installed it, if one did.
    pub installed_by: Option<InstalledBy>,
}

impl BundleFacts {
    /// Whether the user may move the bundle, from its entry facts.
    pub const fn removability(&self) -> Removability {
        Removability::of(self.entry)
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::PathBuf;

        use crate::{
            AppsError, BundleId, CancelToken, InstalledApp, InstalledApps, InstalledBy, InstalledList, Manager,
            SystemError,
        };

        /// A port that knows only the three methods every port had before bundles.
        struct ListOnly;

        impl InstalledApps for ListOnly {
            fn list(&self) -> Result<InstalledList, SystemError> {
                Ok(InstalledList::default())
            }

            fn find(&self, _: &[BundleId], _: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
                Ok(Vec::new())
            }

            fn search_complete(&self, _: &CancelToken) -> Result<(), SystemError> {
                Ok(())
            }
        }

        #[test]
        fn a_token_with_a_slash_or_a_space_is_refused() {
            for token in ["editor/x", "editor x", "editor;rm", "Editor", "editor\n", "$(editor)"] {
                assert_eq!(
                    InstalledBy::new(Manager::Homebrew, token),
                    Err(AppsError::Token { token: token.into() }),
                    "{token:?}"
                );
            }
        }

        #[test]
        fn an_empty_or_long_token_is_refused() {
            assert!(InstalledBy::new(Manager::Homebrew, "").is_err());
            assert!(InstalledBy::new(Manager::Homebrew, "a".repeat(129)).is_err());
        }

        #[test]
        fn the_default_bundle_answer_is_an_error() {
            let app = InstalledApp {
                bundle_id: BundleId::new("com.example.Editor").unwrap(),
                team_id: None,
                name: "Editor".into(),
                version: None,
                build: None,
                path: "/fixture/Applications/Editor.app".into(),
                helpers: Vec::new(),
                changed: None,
            };

            assert!(ListOnly.bundle(&app, &CancelToken::new()).is_err());
        }
    }

    mod positive {
        use crate::{BundleFacts, EntryFacts, InstalledBy, Manager, Removability, Sizes};

        #[test]
        fn a_valid_token_is_kept_and_named_in_the_command() {
            let installed = InstalledBy::new(Manager::Homebrew, "editor@beta.2_x+y-z").unwrap();

            assert_eq!(installed.manager(), Manager::Homebrew);
            assert_eq!(installed.token(), "editor@beta.2_x+y-z");
            assert_eq!(installed.command(), "brew uninstall --cask editor@beta.2_x+y-z");
            assert!(InstalledBy::new(Manager::Homebrew, "a".repeat(128)).is_ok());
        }

        #[test]
        fn the_manager_has_its_name() {
            assert_eq!(Manager::Homebrew.name(), "Homebrew");
        }

        #[test]
        fn the_removability_is_the_entry_s() {
            let facts = |entry| BundleFacts { sizes: Sizes::ZERO, entry, last_used: None, installed_by: None };

            assert_eq!(facts(EntryFacts::REMOVABLE).removability(), Removability::Removable);
            assert_eq!(facts(EntryFacts::default()).removability(), Removability::NeedsAdministrator);
        }
    }
}
