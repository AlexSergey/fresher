//! The three places an IDE keeps a folder per version in, each with its category (design, section 1).

use fresher_core::{Claim, DangerLevel, Interest, InterestError, KnownFolder, Pattern};

/// The path, under each place's known folder, of an IDE version's folder.
const VERSION_FOLDERS: &str = "JetBrains/*";

/// One of the three places.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Place {
    Settings,
    Caches,
    Logs,
}

/// Every place, in the order of the module's interests: a match's `interest` indexes it.
pub(crate) const PLACES: [Place; 3] = [Place::Settings, Place::Caches, Place::Logs];

impl Place {
    const fn folder(self) -> KnownFolder {
        match self {
            Self::Settings => KnownFolder::AppSupport,
            Self::Caches => KnownFolder::AppCaches,
            Self::Logs => KnownFolder::Logs,
        }
    }

    /// The place's category.
    pub(crate) const fn category(self) -> &'static str {
        match self {
            Self::Settings => "jetbrains.settings",
            Self::Caches => "jetbrains.caches",
            Self::Logs => "jetbrains.logs",
        }
    }

    /// The category's danger level (design, section 5).
    pub(crate) const fn danger(self) -> DangerLevel {
        match self {
            Self::Settings => DangerLevel::Medium,
            Self::Caches | Self::Logs => DangerLevel::Safe,
        }
    }

    /// The title of a finding of the place's folder when its IDE is no longer installed: `DataGrip 2024.3 caches`.
    pub(crate) const fn uninstalled_title(self) -> &'static str {
        match self {
            Self::Settings => "jetbrains.finding.uninstalled-settings",
            Self::Caches => "jetbrains.finding.uninstalled-caches",
            Self::Logs => "jetbrains.finding.uninstalled-logs",
        }
    }

    /// What a finding's identity starts with: `caches/WebStorm2024.3`.
    pub(crate) const fn identity(self) -> &'static str {
        match self {
            Self::Settings => "settings",
            Self::Caches => "caches",
            Self::Logs => "logs",
        }
    }

    /// Every version folder of the place, each claimed whole.
    pub(crate) fn interest(self) -> Result<Interest, InterestError> {
        Interest::new(Pattern::Under(self.folder(), VERSION_FOLDERS.into()), Claim::Whole, vec![])
    }
}
