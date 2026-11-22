//! What a row is, from its name's extension: two tables (rules as data), one for files and one for packages.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::TextKey;

/// What a row is. `File` and `Folder` are the row's real kind, not a catch-all category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ItemKind {
    DiskImage,
    Archive,
    Installer,
    Video,
    Audio,
    Image,
    VirtualMachine,
    MediaLibrary,
    Project,
    MemoryDump,
    Document,
    DeviceBackup,
    File,
    Folder,
}

impl ItemKind {
    /// Every kind, in the order of the enum.
    pub const ALL: [Self; 14] = [
        Self::DiskImage,
        Self::Archive,
        Self::Installer,
        Self::Video,
        Self::Audio,
        Self::Image,
        Self::VirtualMachine,
        Self::MediaLibrary,
        Self::Project,
        Self::MemoryDump,
        Self::Document,
        Self::DeviceBackup,
        Self::File,
        Self::Folder,
    ];

    /// The kind's name for the user: `core.largest.kind.<kebab>`.
    pub fn text_key(self) -> TextKey {
        let segment = match self {
            Self::DiskImage => "disk-image",
            Self::Archive => "archive",
            Self::Installer => "installer",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Image => "image",
            Self::VirtualMachine => "virtual-machine",
            Self::MediaLibrary => "media-library",
            Self::Project => "project",
            Self::MemoryDump => "memory-dump",
            Self::Document => "document",
            Self::DeviceBackup => "device-backup",
            Self::File => "file",
            Self::Folder => "folder",
        };
        TextKey::from_valid(format!("core.largest.kind.{segment}"))
    }
}

/// The extensions of files (lowercase, without the dot) and their kind.
pub(crate) const FILE_KINDS: &[(&str, ItemKind)] = &[
    ("dmg", ItemKind::DiskImage),
    ("iso", ItemKind::DiskImage),
    ("img", ItemKind::DiskImage),
    ("sparseimage", ItemKind::DiskImage),
    ("toast", ItemKind::DiskImage),
    ("zip", ItemKind::Archive),
    ("7z", ItemKind::Archive),
    ("rar", ItemKind::Archive),
    ("tar", ItemKind::Archive),
    ("gz", ItemKind::Archive),
    ("tgz", ItemKind::Archive),
    ("bz2", ItemKind::Archive),
    ("xz", ItemKind::Archive),
    ("zst", ItemKind::Archive),
    ("xip", ItemKind::Archive),
    ("pkg", ItemKind::Installer),
    ("mpkg", ItemKind::Installer),
    ("mov", ItemKind::Video),
    ("mp4", ItemKind::Video),
    ("m4v", ItemKind::Video),
    ("mkv", ItemKind::Video),
    ("avi", ItemKind::Video),
    ("wmv", ItemKind::Video),
    ("webm", ItemKind::Video),
    ("mts", ItemKind::Video),
    ("mp3", ItemKind::Audio),
    ("m4a", ItemKind::Audio),
    ("wav", ItemKind::Audio),
    ("aif", ItemKind::Audio),
    ("aiff", ItemKind::Audio),
    ("flac", ItemKind::Audio),
    ("aac", ItemKind::Audio),
    ("jpg", ItemKind::Image),
    ("jpeg", ItemKind::Image),
    ("png", ItemKind::Image),
    ("heic", ItemKind::Image),
    ("tif", ItemKind::Image),
    ("tiff", ItemKind::Image),
    ("psd", ItemKind::Image),
    ("raw", ItemKind::Image),
    ("dng", ItemKind::Image),
    ("cr2", ItemKind::Image),
    ("nef", ItemKind::Image),
    ("vdi", ItemKind::VirtualMachine),
    ("vmdk", ItemKind::VirtualMachine),
    ("qcow2", ItemKind::VirtualMachine),
    ("vhd", ItemKind::VirtualMachine),
    ("vhdx", ItemKind::VirtualMachine),
    ("hprof", ItemKind::MemoryDump),
    ("dmp", ItemKind::MemoryDump),
    ("core", ItemKind::MemoryDump),
    ("pdf", ItemKind::Document),
    ("key", ItemKind::Document),
    ("pages", ItemKind::Document),
    ("numbers", ItemKind::Document),
    ("docx", ItemKind::Document),
    ("pptx", ItemKind::Document),
    ("xlsx", ItemKind::Document),
    ("ipsw", ItemKind::DeviceBackup),
];

/// The extensions of packages, folders that are one document (lowercase, without the dot), and their kind.
pub(crate) const PACKAGE_KINDS: &[(&str, ItemKind)] = &[
    ("sparsebundle", ItemKind::DiskImage),
    ("pkg", ItemKind::Installer),
    ("mpkg", ItemKind::Installer),
    ("vmwarevm", ItemKind::VirtualMachine),
    ("pvm", ItemKind::VirtualMachine),
    ("utm", ItemKind::VirtualMachine),
    ("photoslibrary", ItemKind::MediaLibrary),
    ("musiclibrary", ItemKind::MediaLibrary),
    ("tvlibrary", ItemKind::MediaLibrary),
    ("imovielibrary", ItemKind::MediaLibrary),
    ("fcpbundle", ItemKind::MediaLibrary),
    ("aplibrary", ItemKind::MediaLibrary),
    ("logicx", ItemKind::Project),
    ("band", ItemKind::Project),
    ("xcarchive", ItemKind::Project),
];

fn in_table(table: &[(&str, ItemKind)], extension: &str) -> Option<ItemKind> {
    let extension = extension.to_ascii_lowercase();
    table.iter().find(|(known, _)| *known == extension).map(|(_, kind)| *kind)
}

fn look_up(table: &[(&str, ItemKind)], name: &str) -> Option<ItemKind> {
    in_table(table, name.rsplit_once('.')?.1)
}

/// The kind of the node called `name`: the last extension, case ignored, looked up in the table of its sort. A file
/// not in its table is `File`, a folder not in its table is `Folder`.
pub fn kind_of(name: &str, folder: bool) -> ItemKind {
    if folder {
        look_up(PACKAGE_KINDS, name).unwrap_or(ItemKind::Folder)
    } else {
        look_up(FILE_KINDS, name).unwrap_or(ItemKind::File)
    }
}

/// `true` for a folder name whose extension is a package's: the folder is one item and is never opened.
pub fn is_package(name: &str) -> bool {
    look_up(PACKAGE_KINDS, name).is_some()
}

/// `true` for an app bundle's name: the extension `app`, case ignored.
pub fn is_app_bundle(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("app"))
}

/// For a file that is an archive, its name without its archive extensions (`Foo.zip`, `Foo.tar.gz` and `Foo.tgz` are
/// `Foo`); `None` for anything else or when nothing is left.
pub fn archive_stem(name: &str) -> Option<&str> {
    let mut stem = name;
    let mut stripped = false;
    while let Some((rest, extension)) = stem.rsplit_once('.') {
        if in_table(FILE_KINDS, extension) != Some(ItemKind::Archive) {
            break;
        }
        stem = rest;
        stripped = true;
    }
    (stripped && !stem.is_empty()).then_some(stem)
}

#[cfg(test)]
mod tests {
    mod negative {
        use super::super::{ItemKind, archive_stem, is_app_bundle, is_package, kind_of};

        #[test]
        fn a_name_without_an_extension_is_a_plain_file() {
            assert_eq!(kind_of("README", false), ItemKind::File);
            assert_eq!(kind_of("notes.unknown", false), ItemKind::File);
        }

        #[test]
        fn a_folder_not_in_the_table_is_a_plain_folder() {
            assert_eq!(kind_of("Work", true), ItemKind::Folder);
            assert_eq!(kind_of("Work.d", true), ItemKind::Folder);
            assert!(!is_package("Work"));
        }

        #[test]
        fn a_file_extension_does_not_make_a_folder_a_package() {
            assert_eq!(kind_of("Foo.zip", true), ItemKind::Folder);
            assert!(!is_package("Foo.zip"));
        }

        #[test]
        fn an_app_bundle_has_no_kind_of_its_own() {
            assert_eq!(kind_of("Editor.app", true), ItemKind::Folder);
            assert!(!is_package("Editor.app"));
            assert!(is_app_bundle("Editor.app"));
            assert!(is_app_bundle("EDITOR.APP"));
            assert!(!is_app_bundle("Editor"));
        }

        #[test]
        fn a_non_archive_has_no_stem() {
            assert_eq!(archive_stem("Movie.mov"), None);
            assert_eq!(archive_stem("README"), None);
        }

        #[test]
        fn an_archive_named_only_by_its_extension_has_no_stem() {
            assert_eq!(archive_stem(".zip"), None);
            assert_eq!(archive_stem(".tar.gz"), None);
        }
    }

    mod positive {
        use super::super::{FILE_KINDS, ItemKind, PACKAGE_KINDS, archive_stem, is_package, kind_of};

        #[test]
        fn extensions_are_read_case_insensitively() {
            assert_eq!(kind_of("MOVIE.MOV", false), ItemKind::Video);
            assert_eq!(kind_of("Library.PhotosLibrary", true), ItemKind::MediaLibrary);
        }

        #[test]
        fn a_double_extension_archive_is_an_archive() {
            assert_eq!(kind_of("Foo.tar.gz", false), ItemKind::Archive);
        }

        #[test]
        fn a_virtual_machine_package_is_a_package() {
            assert!(is_package("Windows.pvm"));
            assert_eq!(kind_of("Windows.pvm", true), ItemKind::VirtualMachine);
            assert_eq!(kind_of("disk.vmdk", false), ItemKind::VirtualMachine);
        }

        #[test]
        fn every_table_entry_is_lowercase_and_unique() {
            for table in [FILE_KINDS, PACKAGE_KINDS] {
                let mut seen = std::collections::BTreeSet::new();
                for (extension, _) in table {
                    assert_eq!(*extension, extension.to_ascii_lowercase());
                    assert!(!extension.starts_with('.'), "{extension}");
                    assert!(seen.insert(*extension), "{extension} twice");
                }
            }
        }

        #[test]
        fn the_stem_drops_every_archive_extension() {
            assert_eq!(archive_stem("Foo.zip"), Some("Foo"));
            assert_eq!(archive_stem("Foo.tar.gz"), Some("Foo"));
            assert_eq!(archive_stem("Foo.tgz"), Some("Foo"));
            assert_eq!(archive_stem("Foo.bar.ZIP"), Some("Foo.bar"));
        }

        #[test]
        fn every_kind_has_its_text_key() {
            let keys: Vec<_> = ItemKind::ALL.iter().map(|kind| kind.text_key().to_string()).collect();

            assert_eq!(keys[0], "core.largest.kind.disk-image");
            assert_eq!(keys[13], "core.largest.kind.folder");
        }
    }
}
