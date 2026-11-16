//! One bundle (`.app`, `.appex`, `.xpc`) as its own `Info.plist` and code signature describe it.

use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use fresher_core::{BundleId, TeamId};

use crate::plist::{MAX_PLIST, read_strings};
use crate::signature::team_id;

/// The bundle extensions an app's processes and helpers run from.
pub(crate) const BUNDLE_EXTENSIONS: [&str; 3] = ["app", "appex", "xpc"];

/// The folders of an app bundle that hold its helpers, one level deep.
const HELPER_FOLDERS: [&[&str]; 5] = [
    &["Contents", "Library", "LoginItems"],
    &["Contents", "Helpers"],
    &["Contents", "XPCServices"],
    &["Contents", "PlugIns"],
    &["Contents", "Frameworks"],
];

/// What a bundle's `Info.plist` says of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Bundle {
    pub(crate) id: BundleId,
    /// The name it shows: its display name, else its name, else its folder's name.
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) build: Option<String>,
    /// The main executable's file name.
    pub(crate) executable: Option<String>,
}

/// Whether `path` names a bundle by its extension.
pub(crate) fn is_bundle(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| BUNDLE_EXTENSIONS.iter().any(|known| known.eq_ignore_ascii_case(extension)))
}

/// The bundle at `path`; `None` when it is not a folder (a link is not followed), has no readable `Info.plist`, or
/// no valid `CFBundleIdentifier`.
pub(crate) fn read_bundle(path: &Path) -> Option<Bundle> {
    if !fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir()) {
        return None;
    }
    let plist = path.join("Contents").join("Info.plist");
    let bytes = read_capped(&plist)?;
    let mut strings = match read_strings(&bytes) {
        Ok(strings) => strings,
        Err(error) => {
            tracing::debug!(%error, "a bundle's Info.plist cannot be read");
            return None;
        }
    };
    let id = BundleId::new(strings.remove("CFBundleIdentifier")?).ok()?;
    let folder_name = path.file_stem().map(|stem| stem.to_string_lossy().into_owned());
    let name = strings
        .remove("CFBundleDisplayName")
        .or_else(|| strings.remove("CFBundleName"))
        .filter(|name| !name.trim().is_empty())
        .or(folder_name)
        .unwrap_or_else(|| id.as_str().to_owned());
    Some(Bundle {
        id,
        name,
        version: strings.remove("CFBundleShortVersionString"),
        build: strings.remove("CFBundleVersion"),
        executable: strings.remove("CFBundleExecutable"),
    })
}

/// The team id in the signature of the bundle's main executable `Contents/MacOS/<executable>`.
pub(crate) fn bundle_team(path: &Path, executable: &str) -> Option<TeamId> {
    if executable.contains('/') || executable == ".." {
        return None;
    }
    let file = path.join("Contents").join("MacOS").join(executable);
    if !fs::symlink_metadata(&file).is_ok_and(|metadata| metadata.is_file()) {
        return None;
    }
    let mut opened = File::open(&file).ok()?;
    TeamId::new(team_id(&mut opened)?).ok()
}

/// The bundle ids of the helpers inside the app bundle at `path`, sorted, each once.
pub(crate) fn helpers(path: &Path) -> Vec<BundleId> {
    let mut ids: Vec<BundleId> = HELPER_FOLDERS
        .iter()
        .map(|names| names.iter().fold(path.to_path_buf(), |folder, name| folder.join(name)))
        .flat_map(|folder| bundles_in(&folder))
        .filter_map(|bundle| read_bundle(&bundle).map(|bundle| bundle.id))
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

/// The bundles directly in `folder`, by path; nothing when it cannot be read.
pub(crate) fn bundles_in(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else { return Vec::new() };
    let mut found: Vec<PathBuf> =
        entries.filter_map(Result::ok).map(|entry| entry.path()).filter(|path| is_bundle(path)).collect();
    found.sort();
    found
}

/// The file at `path` when it is a regular file of at most `MAX_PLIST` bytes; a link is not followed.
fn read_capped(path: &Path) -> Option<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_PLIST as u64 {
        return None;
    }
    let mut bytes = Vec::new();
    File::open(path).ok()?.take(MAX_PLIST as u64 + 1).read_to_end(&mut bytes).ok()?;
    (bytes.len() <= MAX_PLIST).then_some(bytes)
}

#[cfg(test)]
pub(crate) mod fixture {
    use std::fs;
    use std::path::{Path, PathBuf};

    /// Writes a bundle at `path` with an XML `Info.plist` of `entries`.
    pub(crate) fn bundle(path: &Path, entries: &[(&str, &str)]) -> PathBuf {
        let contents = path.join("Contents");
        fs::create_dir_all(&contents).unwrap();
        let body: String =
            entries.iter().map(|(key, value)| ["<key>", key, "</key><string>", value, "</string>"].concat()).collect();
        let plist = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict>{body}<key>LSUIElement</key><true/></dict></plist>\n"
        );
        fs::write(contents.join("Info.plist"), plist).unwrap();
        path.to_path_buf()
    }

    /// An app bundle `<folder>/<name>.app` with the id `id`.
    pub(crate) fn app(folder: &Path, name: &str, id: &str) -> PathBuf {
        bundle(&folder.join(format!("{name}.app")), &[("CFBundleIdentifier", id), ("CFBundleName", name)])
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use crate::bundle::fixture::bundle;
        use crate::bundle::{bundle_team, helpers, read_bundle};

        #[test]
        fn a_folder_without_a_valid_id_is_not_a_bundle() {
            let root = tempfile::tempdir().unwrap();
            let missing = root.path().join("Missing.app");
            fs::create_dir(&missing).unwrap();
            let no_id = bundle(&root.path().join("NoId.app"), &[("CFBundleName", "No id")]);
            let bad_id = bundle(&root.path().join("Bad.app"), &[("CFBundleIdentifier", "not an id")]);
            let broken = bundle(&root.path().join("Broken.app"), &[]);
            fs::write(broken.join("Contents/Info.plist"), "<plist><dict><key>x</key>").unwrap();
            let file = root.path().join("File.app");
            fs::write(&file, "").unwrap();

            for path in [&missing, &no_id, &bad_id, &broken, &file] {
                assert_eq!(read_bundle(path), None, "{}", path.display());
            }
        }

        #[test]
        fn a_link_to_a_bundle_is_not_followed() {
            let root = tempfile::tempdir().unwrap();
            let real = bundle(&root.path().join("Real.app"), &[("CFBundleIdentifier", "com.example.Real")]);
            let link = root.path().join("Link.app");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            let linked_plist = root.path().join("Plist.app");
            fs::create_dir_all(linked_plist.join("Contents")).unwrap();
            std::os::unix::fs::symlink(real.join("Contents/Info.plist"), linked_plist.join("Contents/Info.plist"))
                .unwrap();

            assert_eq!(read_bundle(&link), None);
            assert_eq!(read_bundle(&linked_plist), None);
        }

        #[test]
        fn an_executable_that_is_missing_unsigned_or_outside_has_no_team() {
            let root = tempfile::tempdir().unwrap();
            let app = bundle(&root.path().join("App.app"), &[("CFBundleIdentifier", "com.example.App")]);
            fs::create_dir_all(app.join("Contents/MacOS")).unwrap();
            fs::write(app.join("Contents/MacOS/App"), b"#!/bin/sh\n").unwrap();

            assert_eq!(bundle_team(&app, "App"), None);
            assert_eq!(bundle_team(&app, "Missing"), None);
            assert_eq!(bundle_team(&app, "../../Contents/Info.plist"), None);
            assert_eq!(bundle_team(&app, ".."), None);
            assert_eq!(helpers(&app), []);
        }

        #[test]
        fn an_oversized_plist_is_not_read() {
            let root = tempfile::tempdir().unwrap();
            let app = bundle(&root.path().join("Big.app"), &[]);
            let padding = " ".repeat(crate::plist::MAX_PLIST);
            let plist = format!(
                "<plist><dict><key>CFBundleIdentifier</key><string>com.example.Big</string></dict></plist>{padding}"
            );
            fs::write(app.join("Contents/Info.plist"), plist).unwrap();

            assert_eq!(read_bundle(&app), None);
        }
    }

    mod positive {
        use fresher_core::BundleId;

        use crate::bundle::fixture::bundle;
        use crate::bundle::{helpers, is_bundle, read_bundle};

        #[test]
        fn a_bundle_gives_its_id_name_version_and_executable() {
            let root = tempfile::tempdir().unwrap();
            let app = bundle(
                &root.path().join("Editor.app"),
                &[
                    ("CFBundleIdentifier", "com.example.Editor"),
                    ("CFBundleName", "Editor"),
                    ("CFBundleDisplayName", "Example Editor"),
                    ("CFBundleShortVersionString", "2026.3"),
                    ("CFBundleVersion", "263.1"),
                    ("CFBundleExecutable", "editor"),
                ],
            );

            let read = read_bundle(&app).unwrap();

            assert_eq!(read.id, BundleId::new("com.example.Editor").unwrap());
            assert_eq!(read.name, "Example Editor");
            assert_eq!((read.version.as_deref(), read.build.as_deref()), (Some("2026.3"), Some("263.1")));
            assert_eq!(read.executable.as_deref(), Some("editor"));
            assert!(
                is_bundle(&app)
                    && is_bundle(std::path::Path::new("a.XPC"))
                    && !is_bundle(std::path::Path::new("a.txt"))
            );
        }

        #[test]
        fn a_bundle_without_a_name_is_named_after_its_folder() {
            let root = tempfile::tempdir().unwrap();
            let app = bundle(&root.path().join("Plain.app"), &[("CFBundleIdentifier", "com.example.Plain")]);

            assert_eq!(read_bundle(&app).unwrap().name, "Plain");
        }

        #[test]
        fn the_helpers_inside_an_app_are_found_once_each() {
            let root = tempfile::tempdir().unwrap();
            let app = bundle(&root.path().join("Editor.app"), &[("CFBundleIdentifier", "com.example.Editor")]);
            let id = |id: &'static str| [("CFBundleIdentifier", id)];
            bundle(&app.join("Contents/Library/LoginItems/Launcher.app"), &id("com.example.Editor.Launcher"));
            bundle(&app.join("Contents/XPCServices/Fetch.xpc"), &id("com.example.Editor.Fetch"));
            bundle(&app.join("Contents/PlugIns/Share.appex"), &id("com.example.Editor.Share"));
            bundle(&app.join("Contents/Frameworks/Helper.app"), &id("com.example.Editor.Fetch"));

            let found: Vec<String> = helpers(&app).iter().map(|id| id.as_str().to_owned()).collect();

            assert_eq!(found, ["com.example.Editor.Fetch", "com.example.Editor.Launcher", "com.example.Editor.Share"]);
        }
    }
}
