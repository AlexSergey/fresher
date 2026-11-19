//! The module's categories and their danger levels (design, section 3.2): code, since a category id is a
//! `&'static str`.

use fresher_core::DangerLevel;

/// A category of the module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Category {
    Unfinished,
    SafariResumable,
    Installers,
    MacosInstallers,
    LargeUntouched,
}

impl Category {
    /// Every category, in the order the module lists them.
    pub(crate) const ALL: [Self; 5] =
        [Self::Unfinished, Self::SafariResumable, Self::Installers, Self::MacosInstallers, Self::LargeUntouched];

    /// The category's name: the start of each of its findings' identities (`unfinished/movie.mp4.crdownload`).
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Unfinished => "unfinished",
            Self::SafariResumable => "safari-resumable",
            Self::Installers => "installers",
            Self::MacosInstallers => "macos-installers",
            Self::LargeUntouched => "large-untouched",
        }
    }

    /// The category's id in the module (`downloads.unfinished`).
    pub(crate) const fn id(self) -> &'static str {
        match self {
            Self::Unfinished => "downloads.unfinished",
            Self::SafariResumable => "downloads.safari-resumable",
            Self::Installers => "downloads.installers",
            Self::MacosInstallers => "downloads.macos-installers",
            Self::LargeUntouched => "downloads.large-untouched",
        }
    }

    pub(crate) const fn danger(self) -> DangerLevel {
        match self {
            Self::SafariResumable => DangerLevel::Medium,
            Self::LargeUntouched => DangerLevel::High,
            Self::Unfinished | Self::Installers | Self::MacosInstallers => DangerLevel::Low,
        }
    }
}
