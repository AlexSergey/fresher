//! What macOS lets the app see: the core's `AccessProbe` over `fresher-macos`'s privacy checks.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use fresher_core::{
    Access, AccessProbe, AccessStatus, Capability, CapabilityState, GuardedFolder, GuardedPlace, VolumeKind,
    expand_home,
};
use fresher_macos::{Guard, Probe, fda_candidates, full_disk_access, try_inside, try_place, volume_guard};

use crate::{MacosVersion, guarded_root};

/// The first macOS whose privacy protection guards the data of other apps.
const OTHER_APPS_DATA_SINCE: u16 = 14;

/// The access of the user whose home folder the probe was built with. Never prompts but in `try_place`; remembers
/// the answer of every try for the session, so the status shows what a scan learned without a new open.
#[derive(Debug)]
pub struct MacAccess {
    home: PathBuf,
    version: Option<MacosVersion>,
    tried: Mutex<Vec<(Capability, Access)>>,
}

impl MacAccess {
    /// The access of `home`, on the macOS `version` (`None` when it could not be read: every guard then applies).
    pub fn new(home: PathBuf, version: Option<MacosVersion>) -> Self {
        Self { home, version, tried: Mutex::new(Vec::new()) }
    }

    fn last_try(&self, capability: Capability) -> Option<Access> {
        let tried = self.tried.lock().unwrap_or_else(PoisonError::into_inner);
        tried.iter().find(|(known, _)| *known == capability).map(|&(_, access)| access)
    }

    fn remember(&self, capability: Capability, access: Access) {
        let mut tried = self.tried.lock().unwrap_or_else(PoisonError::into_inner);
        match tried.iter_mut().find(|(known, _)| *known == capability) {
            Some(entry) => entry.1 = access,
            None => tried.push((capability, access)),
        }
    }

    /// Whether this macOS guards the data of other apps; an unknown version does (fail closed).
    fn guards_other_apps_data(&self) -> bool {
        self.version.is_none_or(|version| version.major >= OTHER_APPS_DATA_SINCE)
    }

    fn state(&self, capability: Capability, all_files: Access) -> Access {
        match capability {
            Capability::AllFiles => all_files,
            Capability::OtherAppsData if !self.guards_other_apps_data() => Access::NotApplicable,
            Capability::Folder(_) | Capability::OtherAppsData if all_files == Access::Granted => Access::Granted,
            _ => self.last_try(capability).unwrap_or(Access::NotAsked),
        }
    }
}

impl AccessProbe for MacAccess {
    fn status(&self) -> AccessStatus {
        let all_files = access_of(full_disk_access(&fda_candidates(&self.home)));
        let state = |capability| CapabilityState::new(capability, self.state(capability, all_files));
        AccessStatus { capabilities: Capability::SYSTEM.into_iter().map(state).collect() }
    }

    /// `roots` as the settings write them: `~` and `~/…` are the home folder, as the walk expands them. Names are
    /// compared without regard to ASCII case; a root that meets no guarded place as written is followed through its
    /// links (never into a guarded place) and compared again, so a root reported is the path the walk opens.
    fn guarded_places(&self, roots: &[PathBuf]) -> Vec<GuardedPlace> {
        let guarded = fresher_macos::guarded_places(&self.home);
        // The places of the canonical home, worked out once, and only when a root needs its links followed.
        let mut real_guarded: Option<Vec<(PathBuf, Guard)>> = None;
        let mut places: Vec<GuardedPlace> = Vec::new();
        for root in roots.iter().map(|root| expand_home(root, &self.home)) {
            if let Some(guard) = volume_guard(&root) {
                // A root on a volume that may be guarded: only the volume's own guard, its links are not followed.
                add(&mut places, &root, guard);
                continue;
            }
            if add_places(&mut places, &root, &guarded) {
                continue;
            }
            let real_guarded = real_guarded.get_or_insert_with(|| {
                let real_home = fs::canonicalize(&self.home).unwrap_or_else(|_| self.home.clone());
                fresher_macos::guarded_places(&real_home)
            });
            let stop = |path: &Path| {
                real_guarded.iter().any(|(place, _)| guarded_root::is_within(path, place))
                    || volume_guard(path).is_some()
            };
            if let Some(real) = guarded_root::resolve(&root, stop).filter(|real| *real != root) {
                add_places(&mut places, &real, real_guarded);
            }
        }
        places
    }

    fn try_place(&self, place: &GuardedPlace) -> Access {
        let probe = match place.capability {
            Capability::OtherAppsData => try_inside(&place.path),
            _ => try_place(&place.path),
        };
        let access = access_of(probe);
        self.remember(place.capability, access);
        access
    }
}

/// Adds the guarded places of `guarded` that `root` holds or is inside of: a place inside the root as itself, a root
/// inside a place as the root. `true` when one matched.
fn add_places(places: &mut Vec<GuardedPlace>, root: &Path, guarded: &[(PathBuf, Guard)]) -> bool {
    let mut matched = false;
    for (path, guard) in guarded {
        if guarded_root::is_within(path, root) {
            add(places, path, *guard);
            matched = true;
        } else if guarded_root::is_within(root, path) {
            add(places, root, *guard);
            matched = true;
        }
    }
    matched
}

fn add(places: &mut Vec<GuardedPlace>, path: &Path, guard: Guard) {
    let place = GuardedPlace { path: path.to_path_buf(), capability: capability_of(guard) };
    if !places.contains(&place) {
        places.push(place);
    }
}

const fn access_of(probe: Probe) -> Access {
    match probe {
        Probe::Allowed => Access::Granted,
        Probe::Refused => Access::Denied,
        Probe::Missing | Probe::Failed => Access::Unknown,
    }
}

const fn capability_of(guard: Guard) -> Capability {
    match guard {
        Guard::FullDiskAccess => Capability::AllFiles,
        Guard::OtherAppsData => Capability::OtherAppsData,
        Guard::Desktop => Capability::Folder(GuardedFolder::Desktop),
        Guard::Documents => Capability::Folder(GuardedFolder::Documents),
        Guard::Downloads => Capability::Folder(GuardedFolder::Downloads),
        Guard::RemovableVolume => Capability::Volume(VolumeKind::Removable),
        Guard::NetworkVolume => Capability::Volume(VolumeKind::Network),
    }
}

/// The guard behind a capability the system grants; a tool has none.
pub(crate) const fn guard_of(capability: Capability) -> Option<Guard> {
    match capability {
        Capability::AllFiles => Some(Guard::FullDiskAccess),
        Capability::OtherAppsData => Some(Guard::OtherAppsData),
        Capability::Folder(GuardedFolder::Desktop) => Some(Guard::Desktop),
        Capability::Folder(GuardedFolder::Documents) => Some(Guard::Documents),
        Capability::Folder(GuardedFolder::Downloads) => Some(Guard::Downloads),
        Capability::Volume(VolumeKind::Removable) => Some(Guard::RemovableVolume),
        Capability::Volume(VolumeKind::Network) => Some(Guard::NetworkVolume),
        Capability::Tool(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use fresher_core::{Capability, GuardedFolder};

    use crate::{MacAccess, MacosVersion};

    const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);
    const DOCUMENTS: Capability = Capability::Folder(GuardedFolder::Documents);
    const SONOMA: MacosVersion = MacosVersion { major: 14, minor: 0 };

    fn access(home: &Path) -> MacAccess {
        MacAccess::new(home.to_path_buf(), Some(SONOMA))
    }

    /// Takes every permission from a folder; gives them back when dropped, whatever the test's outcome.
    struct Locked<'a>(&'a Path);

    impl<'a> Locked<'a> {
        fn new(folder: &'a Path) -> Self {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(folder, fs::Permissions::from_mode(0o000)).unwrap();
            Self(folder)
        }
    }

    impl Drop for Locked<'_> {
        fn drop(&mut self) {
            use std::os::unix::fs::PermissionsExt as _;
            let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o755));
        }
    }

    /// The first Full Disk Access candidate, readable.
    fn readable_candidate(home: &Path) {
        let folder = home.join("Library/Application Support/com.apple.TCC");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("TCC.db"), b"x").unwrap();
    }

    mod negative {
        use std::fs;
        use std::path::PathBuf;

        use fresher_core::{Access, AccessProbe, Capability, GuardedPlace};

        use super::{DESKTOP, DOCUMENTS, Locked, access};
        use crate::{MacAccess, MacosVersion, macos_version};

        #[test]
        fn a_home_without_any_candidate_gives_unknown_full_disk_access() {
            let home = tempfile::tempdir().unwrap();

            assert_eq!(access(home.path()).status().access(&Capability::AllFiles), Access::Unknown);
        }

        #[test]
        fn without_full_disk_access_the_folders_are_not_asked() {
            let home = tempfile::tempdir().unwrap();
            for name in ["Desktop", "Documents", "Downloads", "Library/Containers"] {
                fs::create_dir_all(home.path().join(name)).unwrap();
            }

            let status = access(home.path()).status();

            assert_eq!(status.capabilities.len(), Capability::SYSTEM.len());
            for capability in Capability::SYSTEM.into_iter().filter(|capability| *capability != Capability::AllFiles) {
                assert_eq!(status.access(&capability), Access::NotAsked, "{capability:?}");
            }
        }

        #[test]
        fn a_guarded_place_outside_every_root_is_not_reported() {
            let (home, elsewhere) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());

            assert_eq!(access(home.path()).guarded_places(&[elsewhere.path().to_path_buf()]), []);
            assert_eq!(access(home.path()).guarded_places(&[]), []);
        }

        #[test]
        fn a_sibling_with_a_longer_name_is_not_inside_a_guarded_place() {
            let home = tempfile::tempdir().unwrap();
            let root = home.path().join("Documents Old");

            assert_eq!(access(home.path()).guarded_places(&[root]), []);
        }

        #[test]
        fn a_root_linked_outside_every_guarded_place_is_not_reported() {
            let (home, elsewhere) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            std::os::unix::fs::symlink(elsewhere.path(), home.path().join("Link")).unwrap();

            assert_eq!(access(home.path()).guarded_places(&[PathBuf::from("~/Link")]), []);
        }

        #[test]
        fn other_apps_data_is_not_applicable_on_macos_13() {
            let home = tempfile::tempdir().unwrap();
            let ventura = MacAccess::new(home.path().to_path_buf(), Some(MacosVersion { major: 13, minor: 6 }));

            let status = ventura.status();

            assert_eq!(status.access(&Capability::OtherAppsData), Access::NotApplicable);
            assert_eq!(status.access(&DOCUMENTS), Access::NotAsked);
        }

        #[test]
        fn an_unreadable_system_version_keeps_other_apps_data_guarded() {
            let (home, system) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
            let unknown = MacAccess::new(home.path().to_path_buf(), macos_version(system.path()));

            assert_eq!(unknown.status().access(&Capability::OtherAppsData), Access::NotAsked);
        }

        #[test]
        fn a_place_that_cannot_be_told_is_remembered_as_unknown() {
            let home = tempfile::tempdir().unwrap();
            let desktop = home.path().join("Desktop");
            fs::create_dir(&desktop).unwrap();
            let locked = Locked::new(&desktop);
            let probe = access(home.path());

            let answer = probe.try_place(&GuardedPlace { path: desktop.clone(), capability: DESKTOP });

            drop(locked);
            assert_eq!(answer, Access::Unknown);
            assert_eq!(probe.status().access(&DESKTOP), Access::Unknown);
            let missing = GuardedPlace { path: home.path().join("Documents"), capability: DOCUMENTS };
            assert_eq!(probe.try_place(&missing), Access::Unknown);
        }
    }

    mod positive {
        use std::fs;
        use std::path::PathBuf;

        use fresher_core::{Access, AccessProbe, Capability, GuardedFolder, GuardedPlace};

        use super::{DESKTOP, DOCUMENTS, access, readable_candidate};
        use crate::{MacAccess, MacosVersion};

        #[test]
        fn a_readable_candidate_grants_all_files_and_every_folder() {
            let home = tempfile::tempdir().unwrap();
            readable_candidate(home.path());

            let status = access(home.path()).status();

            assert_eq!(status.access(&Capability::AllFiles), Access::Granted);
            for capability in
                [DESKTOP, DOCUMENTS, Capability::Folder(GuardedFolder::Downloads), Capability::OtherAppsData]
            {
                assert_eq!(status.access(&capability), Access::Granted, "{capability:?}");
            }
        }

        #[test]
        fn a_tried_folder_reports_its_answer_in_the_status() {
            let home = tempfile::tempdir().unwrap();
            let desktop = home.path().join("Desktop");
            fs::create_dir(&desktop).unwrap();
            let probe = access(home.path());

            let answer = probe.try_place(&GuardedPlace { path: desktop, capability: DESKTOP });

            assert_eq!(answer, Access::Granted);
            let status = probe.status();
            assert_eq!(status.access(&DESKTOP), Access::Granted);
            assert_eq!(status.access(&DOCUMENTS), Access::NotAsked);
        }

        #[test]
        fn other_apps_data_is_tried_inside_its_folder() {
            let home = tempfile::tempdir().unwrap();
            let containers = home.path().join("Library/Containers");
            fs::create_dir_all(containers.join("com.example.app")).unwrap();
            let probe = access(home.path());

            let answer = probe.try_place(&GuardedPlace { path: containers, capability: Capability::OtherAppsData });

            assert_eq!(answer, Access::Granted);
            assert_eq!(probe.status().access(&Capability::OtherAppsData), Access::Granted);
        }

        #[test]
        fn the_home_written_as_a_tilde_reports_every_place_under_the_home() {
            let home = tempfile::tempdir().unwrap();

            let places = access(home.path()).guarded_places(&[PathBuf::from("~")]);

            assert_eq!(places.len(), 12);
            assert!(places.iter().all(|place| place.path.starts_with(home.path())), "{places:?}");
            assert!(places.contains(&GuardedPlace { path: home.path().join("Desktop"), capability: DESKTOP }));
            assert!(places.contains(&GuardedPlace {
                path: home.path().join("Library/Containers"),
                capability: Capability::OtherAppsData
            }));
        }

        #[test]
        fn a_tilde_root_inside_a_guarded_place_is_reported_as_the_path_the_walk_opens() {
            let home = tempfile::tempdir().unwrap();

            let places = access(home.path()).guarded_places(&[PathBuf::from("~/Documents/Projects")]);

            assert_eq!(places, [GuardedPlace { path: home.path().join("Documents/Projects"), capability: DOCUMENTS }]);
        }

        #[test]
        fn a_root_written_in_another_case_is_inside_its_guarded_place() {
            let home = tempfile::tempdir().unwrap();

            let places = access(home.path()).guarded_places(&[PathBuf::from("~/desktop/Notes")]);

            assert_eq!(places, [GuardedPlace { path: home.path().join("desktop/Notes"), capability: DESKTOP }]);
            let library = access(home.path()).guarded_places(&[PathBuf::from("~/LIBRARY")]);
            assert!(library.contains(&GuardedPlace {
                path: home.path().join("Library/Containers"),
                capability: Capability::OtherAppsData
            }));
        }

        #[test]
        fn a_root_linked_onto_a_guarded_place_is_reported_as_the_place_it_opens() {
            let home = tempfile::tempdir().unwrap();
            let desktop = fs::canonicalize(home.path()).unwrap().join("Desktop");
            fs::create_dir(&desktop).unwrap();
            std::os::unix::fs::symlink(&desktop, home.path().join("Link")).unwrap();

            let places = access(home.path()).guarded_places(&[PathBuf::from("~/Link")]);

            assert_eq!(places, [GuardedPlace { path: desktop, capability: DESKTOP }]);
        }

        #[test]
        fn a_root_linked_into_a_guarded_place_is_reported_as_the_folder_it_opens() {
            let home = tempfile::tempdir().unwrap();
            let documents = fs::canonicalize(home.path()).unwrap().join("Documents");
            fs::create_dir_all(documents.join("Projects")).unwrap();
            std::os::unix::fs::symlink("Documents/Projects", home.path().join("Projects")).unwrap();

            let places = access(home.path()).guarded_places(&[home.path().join("Projects")]);

            assert_eq!(places, [GuardedPlace { path: documents.join("Projects"), capability: DOCUMENTS }]);
        }

        #[test]
        fn a_root_inside_a_guarded_place_is_reported_itself() {
            let home = tempfile::tempdir().unwrap();
            let root = home.path().join("Documents/Projects");

            let places = access(home.path()).guarded_places(std::slice::from_ref(&root));

            assert_eq!(places, [GuardedPlace { path: root, capability: DOCUMENTS }]);
        }

        #[test]
        fn guarded_places_of_the_home_root_are_reported_with_their_capability() {
            let home = tempfile::tempdir().unwrap();
            let unknown_version = MacAccess::new(home.path().to_path_buf(), None);

            let places = unknown_version.guarded_places(&[home.path().to_path_buf()]);

            assert_eq!(places.len(), 12);
            let capability_of = |relative: &str| {
                places.iter().find(|place| place.path == home.path().join(relative)).map(|place| place.capability)
            };
            assert_eq!(capability_of("Pictures"), Some(Capability::AllFiles));
            assert_eq!(capability_of("Library/Containers"), Some(Capability::OtherAppsData));
            assert_eq!(capability_of("Library/Group Containers"), Some(Capability::OtherAppsData));
            assert_eq!(capability_of("Desktop"), Some(DESKTOP));
            assert_eq!(capability_of("Library/Application Support/AddressBook"), Some(Capability::AllFiles));
        }

        #[test]
        fn overlapping_roots_report_each_place_once() {
            let home = tempfile::tempdir().unwrap();
            let roots = [home.path().to_path_buf(), home.path().join("Library")];
            let probe = MacAccess::new(home.path().to_path_buf(), Some(MacosVersion { major: 27, minor: 2 }));

            assert_eq!(probe.guarded_places(&roots).len(), 12);
        }
    }
}
