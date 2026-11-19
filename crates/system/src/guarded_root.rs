//! A scan root as the walk opens it, told against the guarded places without opening any of them: names compared
//! without regard to ASCII case (as the macOS file system does by default), and the root's links read up to the first
//! place where the reading must stop (a guarded place, or a volume that may be guarded).

use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// How many links a root may pass through before it is given up, as the system's own limit on a path.
const MAX_LINKS: usize = 32;

/// `true` when `path` is `base` or inside it, name by name, without regard to ASCII case.
pub(crate) fn is_within(path: &Path, base: &Path) -> bool {
    let mut names = path.components();
    base.components().all(|name| names.next().is_some_and(|other| other.as_os_str().eq_ignore_ascii_case(name)))
}

/// The absolute `root` with every link on its way followed, as the walk opens it; `None` when it cannot be told (a
/// relative root, a loop, a link that cannot be read). Once the path read so far is one where `stop` says to stop (a
/// guarded place or a volume's mount point), no name after it is looked at: the rest of the root is joined as
/// written, so nothing inside such a place is opened or even asked about.
pub(crate) fn resolve(root: &Path, stop: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    if !root.is_absolute() {
        return None;
    }
    let mut pending: Vec<OsString> = names_of(root);
    pending.reverse();
    let mut resolved = PathBuf::from("/");
    let mut links = 0;
    let mut stopped = false;
    while let Some(name) = pending.pop() {
        if name == ".." {
            resolved.pop();
            continue;
        }
        if name == "." || name.is_empty() {
            continue;
        }
        if stopped || stop(&resolved) {
            stopped = true;
            resolved.push(name);
            continue;
        }
        let next = resolved.join(&name);
        let is_link = fs::symlink_metadata(&next).is_ok_and(|metadata| metadata.file_type().is_symlink());
        if !is_link {
            resolved = next;
            continue;
        }
        links += 1;
        if links > MAX_LINKS {
            return None;
        }
        let target = fs::read_link(&next).ok()?;
        if target.is_absolute() {
            resolved = PathBuf::from("/");
        }
        pending.extend(names_of(&target).into_iter().rev());
    }
    Some(resolved)
}

/// The names of `path`, `..` and `.` kept as written; the root of an absolute path left out.
fn names_of(path: &Path) -> Vec<OsString> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_os_string()),
            Component::ParentDir => Some(OsString::from("..")),
            Component::CurDir => Some(OsString::from(".")),
            Component::RootDir | Component::Prefix(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::os::unix::fs::symlink;
        use std::path::Path;

        use crate::guarded_root::{is_within, resolve};

        #[test]
        fn a_sibling_with_a_longer_name_is_not_within() {
            assert!(!is_within(Path::new("/home/Documents Old"), Path::new("/home/Documents")));
            assert!(!is_within(Path::new("/home"), Path::new("/home/Documents")));
        }

        #[test]
        fn a_relative_root_or_a_loop_cannot_be_told() {
            let folder = tempfile::tempdir().unwrap();
            symlink(folder.path().join("b"), folder.path().join("a")).unwrap();
            symlink(folder.path().join("a"), folder.path().join("b")).unwrap();

            assert_eq!(resolve(Path::new("relative/root"), |_| false), None);
            assert_eq!(resolve(&folder.path().join("a"), |_| false), None);
        }

        #[test]
        fn a_link_inside_a_guarded_place_is_never_read() {
            let folder = tempfile::tempdir().unwrap();
            let guarded = std::fs::canonicalize(folder.path()).unwrap().join("Desktop");
            std::fs::create_dir(&guarded).unwrap();
            symlink("/elsewhere", guarded.join("Link")).unwrap();

            let inside = |path: &Path| is_within(path, &guarded);

            assert_eq!(resolve(&guarded.join("Link"), inside), Some(guarded.join("Link")));
        }

        #[test]
        fn a_link_that_leads_onto_a_volume_is_not_read_past_its_mount_point() {
            let folder = tempfile::tempdir().unwrap();
            let base = std::fs::canonicalize(folder.path()).unwrap();
            // A folder standing in for a volume's mount point: the stop rule names it, nothing inside is looked at.
            let volume = base.join("Volume");
            std::fs::create_dir(&volume).unwrap();
            symlink("/elsewhere", volume.join("Link")).unwrap();
            symlink(&volume, base.join("ToVolume")).unwrap();
            let asked = std::cell::RefCell::new(Vec::new());
            let at_volume = |path: &Path| {
                asked.borrow_mut().push(path.to_path_buf());
                path == volume
            };

            let resolved = resolve(&base.join("ToVolume/Link/Photos"), at_volume);

            assert_eq!(resolved, Some(volume.join("Link/Photos")));
            assert!(asked.borrow().iter().all(|path| !path.starts_with(volume.join("Link"))), "{asked:?}");
        }
    }

    mod positive {
        use std::fs;
        use std::os::unix::fs::symlink;
        use std::path::Path;

        use crate::guarded_root::{is_within, resolve};

        #[test]
        fn names_are_compared_without_regard_to_case() {
            assert!(is_within(Path::new("/home/desktop/Notes"), Path::new("/home/Desktop")));
            assert!(is_within(Path::new("/home/Desktop"), Path::new("/home/DESKTOP")));
        }

        #[test]
        fn relative_and_absolute_links_are_followed() {
            let folder = tempfile::tempdir().unwrap();
            let base = fs::canonicalize(folder.path()).unwrap();
            fs::create_dir_all(base.join("Documents/Projects")).unwrap();
            symlink("Documents/Projects", base.join("Relative")).unwrap();
            symlink(base.join("Relative"), base.join("Absolute")).unwrap();

            assert_eq!(resolve(&base.join("Absolute"), |_| false), Some(base.join("Documents/Projects")));
            assert_eq!(resolve(&base.join("Relative/../Projects/."), |_| false), Some(base.join("Documents/Projects")));
        }
    }
}
