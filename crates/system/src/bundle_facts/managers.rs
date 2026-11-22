//! The package managers that install apps, known by the records they keep: data only.

use std::fs;
use std::path::{Component, Path, PathBuf};

use fresher_core::{InstalledBy, Manager};

/// Homebrew's records of the apps it installed: `<token>/<version>/<name>.app` is a link to the installed bundle.
pub(crate) const CASKROOMS: [&str; 2] = ["/opt/homebrew/Caskroom", "/usr/local/Caskroom"];

/// Every cask link of the Caskrooms, read once: the bundle each one points to and its cask's token.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct CaskIndex {
    links: Vec<(PathBuf, String)>,
}

impl CaskIndex {
    /// The links of `casks`, in order: in each, two levels of folders (`<token>/<version>`), then `read_link` on each
    /// `*.app` entry. Nothing is followed: a Caskroom, a token or a version that is a link is not read, and neither is
    /// a link's target. A missing or unreadable folder holds no link; a target that is not UTF-8 is still compared.
    pub(crate) fn read(casks: &[PathBuf]) -> Self {
        let mut links = Vec::new();
        for caskroom in casks {
            for token_folder in folders_in(caskroom) {
                let Some(token) = token_folder.file_name().and_then(|name| name.to_str()).map(str::to_owned) else {
                    continue;
                };
                for version in folders_in(&token_folder) {
                    for entry in entries(&version).into_iter().filter(|entry| is_app(entry)) {
                        let Ok(target) = fs::read_link(&entry) else { continue };
                        links.push((lexical(&version.join(target)), token.clone()));
                    }
                }
            }
        }
        if !links.is_empty() {
            tracing::debug!(links = links.len(), "Homebrew's cask links");
        }
        Self { links }
    }

    /// Homebrew, with the cask's token, when the first cask link to `bundle` (both made absolute, `..` resolved by
    /// name only) names it; `None` when no link does, when `bundle` is not absolute, or when the token is not one a
    /// command may show.
    pub(crate) fn installed_by(&self, bundle: &Path) -> Option<InstalledBy> {
        if !bundle.is_absolute() {
            return None;
        }
        let bundle = lexical(bundle);
        let (_, token) = self.links.iter().find(|(target, _)| *target == bundle)?;
        InstalledBy::new(Manager::Homebrew, token.as_str()).ok()
    }
}

/// The folders directly in `folder` that are real folders (a link to one is not), by name.
fn folders_in(folder: &Path) -> Vec<PathBuf> {
    if !is_real_folder(folder) {
        return Vec::new();
    }
    entries(folder).into_iter().filter(|entry| is_real_folder(entry)).collect()
}

/// The entries of `folder`, by name; none when it cannot be read.
fn entries(folder: &Path) -> Vec<PathBuf> {
    let Ok(listing) = fs::read_dir(folder) else { return Vec::new() };
    let mut entries: Vec<PathBuf> = listing.filter_map(Result::ok).map(|entry| entry.path()).collect();
    entries.sort();
    entries
}

fn is_real_folder(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_dir())
}

fn is_app(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
}

/// `path` with `.` dropped and `..` taking off the name before it, by name only: nothing is read on the disk.
fn lexical(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => resolved.push(other),
        }
    }
    resolved
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use crate::bundle_facts::managers::CaskIndex;

    /// A Caskroom in `root` with `<token>/<version>/` made, and the cask link `<name>.app` there to `target`.
    fn cask(root: &Path, token: &str, name: &str, target: &Path) -> PathBuf {
        let caskroom = root.join("Caskroom");
        let version = caskroom.join(token).join("1.0");
        fs::create_dir_all(&version).unwrap();
        std::os::unix::fs::symlink(target, version.join(format!("{name}.app"))).unwrap();
        caskroom
    }

    /// The bundle `<root>/Applications/<name>.app`, made.
    fn bundle(root: &Path, name: &str) -> PathBuf {
        let bundle = root.join("Applications").join(format!("{name}.app"));
        fs::create_dir_all(bundle.join("Contents")).unwrap();
        bundle
    }

    fn manager_of(casks: &[PathBuf], bundle: &Path) -> Option<fresher_core::InstalledBy> {
        CaskIndex::read(casks).installed_by(bundle)
    }

    mod negative {
        use std::fs;

        use super::{bundle, cask, manager_of};

        #[test]
        fn no_caskroom_is_no_manager() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            let file = root.path().join("file");
            fs::write(&file, b"").unwrap();

            assert_eq!(manager_of(&[], &tool), None);
            assert_eq!(manager_of(&[root.path().join("Missing")], &tool), None);
            assert_eq!(manager_of(&[file], &tool), None);
        }

        #[test]
        fn a_link_to_another_bundle_is_no_manager() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            let other = bundle(root.path(), "Other");
            let caskroom = cask(root.path(), "other", "Other", &other);
            // A file named like a bundle is no record either.
            fs::write(caskroom.join("other/1.0/Tool.app"), b"").unwrap();

            assert_eq!(manager_of(&[caskroom], &tool), None);
        }

        #[test]
        fn a_broken_link_is_no_manager() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            let caskroom = cask(root.path(), "tool", "Tool", &root.path().join("Applications/Gone.app"));

            assert_eq!(manager_of(&[caskroom], &tool), None);
        }

        #[test]
        fn a_token_off_its_shape_is_no_manager() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            let caskroom = cask(root.path(), "Tool Pro", "Tool", &tool);

            assert_eq!(manager_of(&[caskroom], &tool), None);
        }

        #[test]
        fn a_relative_bundle_is_no_manager() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            let caskroom = cask(root.path(), "tool", "Tool", &tool);

            assert_eq!(manager_of(&[caskroom], std::path::Path::new("Applications/Tool.app")), None);
        }

        #[test]
        fn a_caskroom_link_is_never_followed() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            // A whole Caskroom elsewhere, reached through links at each level: none of it is read.
            let elsewhere = cask(&root.path().join("elsewhere"), "tool", "Tool", &tool);
            let caskroom = root.path().join("Caskroom");
            fs::create_dir_all(caskroom.join("linked")).unwrap();
            std::os::unix::fs::symlink(elsewhere.join("tool"), caskroom.join("tool")).unwrap();
            std::os::unix::fs::symlink(elsewhere.join("tool/1.0"), caskroom.join("linked/1.0")).unwrap();
            let linked_room = root.path().join("LinkedCaskroom");
            std::os::unix::fs::symlink(&elsewhere, &linked_room).unwrap();

            assert_eq!(manager_of(&[caskroom, linked_room], &tool), None);
            assert!(manager_of(&[elsewhere], &tool).is_some());
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{InstalledBy, Manager};

        use super::{bundle, cask};
        use crate::bundle_facts::managers::CaskIndex;

        #[test]
        fn a_cask_link_to_the_bundle_names_homebrew_and_its_token() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            let other = bundle(root.path(), "Other");
            let empty = root.path().join("Empty");
            let caskroom = cask(root.path(), "tool-pro@beta", "Tool", &tool);
            cask(root.path(), "other", "Other", &other);

            let index = CaskIndex::read(&[empty, caskroom]);

            let expected = InstalledBy::new(Manager::Homebrew, "tool-pro@beta").unwrap();
            assert_eq!(index.installed_by(&tool), Some(expected));
            assert_eq!(index.installed_by(&other), Some(InstalledBy::new(Manager::Homebrew, "other").unwrap()));
            assert_eq!(
                index.installed_by(&root.path().join("Applications/./Tool.app")).unwrap().token(),
                "tool-pro@beta"
            );
        }

        #[test]
        fn a_relative_cask_link_is_resolved_lexically() {
            let root = tempfile::tempdir().unwrap();
            let tool = bundle(root.path(), "Tool");
            // `<root>/Caskroom/tool/1.0/../../../Applications/Tool.app`: `..` resolved by name.
            let caskroom = cask(root.path(), "tool", "Tool", &PathBuf::from("../../../Applications/./Tool.app"));

            let found = CaskIndex::read(&[caskroom]).installed_by(&tool);

            assert_eq!(found.map(|by| by.token().to_owned()), Some("tool".into()));
        }
    }
}
