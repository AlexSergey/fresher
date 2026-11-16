//! The coverage of a scan. Before the walk: which guarded places the walk may enter. A place whose capability is
//! allowed is walked; a place the user opted into is tried once (the only moment the system may ask the user, inside
//! a scan the user started); every other place is skipped without being touched. After the walk: what was not seen.

use std::path::PathBuf;

use fresher_core::{
    Access, AccessProbe, AccessStatus, Capability, Coverage, GuardedFolder, NotEntered, Unreadable, UnreadableCount,
    WalkSummary,
};

/// The capabilities a core switch opts into, `core.access.<id>`; a volume is opted into by being a scan root.
pub(crate) const OPT_INS: [Capability; 4] = [
    Capability::Folder(GuardedFolder::Desktop),
    Capability::Folder(GuardedFolder::Documents),
    Capability::Folder(GuardedFolder::Downloads),
    Capability::OtherAppsData,
];

/// The key of the switch that opts into `capability`.
pub(crate) fn opt_in_key(capability: Capability) -> String {
    format!("core.access.{}", capability.id())
}

/// What the walk must not enter, and why, once per capability.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) avoid: Vec<PathBuf>,
    pub(crate) not_entered: Vec<NotEntered>,
}

/// Decides every guarded place under `roots`. With every file allowed nothing is tried; a capability is tried at most
/// once, and its answer stands for its other places.
pub(crate) fn before_walk(
    probe: &dyn AccessProbe,
    status: &AccessStatus,
    roots: &[PathBuf],
    opted_in: &[Capability],
) -> Entry {
    decide(probe, status, roots, |capability| may_try(capability, opted_in))
}

/// Decides every guarded place under `roots` for a restore, which reads no disk: nothing is tried, so a place stands
/// as the last probe left it.
pub(crate) fn before_restore(probe: &dyn AccessProbe, status: &AccessStatus, roots: &[PathBuf]) -> Entry {
    decide(probe, status, roots, |_| false)
}

fn decide(
    probe: &dyn AccessProbe,
    status: &AccessStatus,
    roots: &[PathBuf],
    may_try: impl Fn(Capability) -> bool,
) -> Entry {
    let all_files = status.access(&Capability::AllFiles) == Access::Granted;
    let mut tried: Vec<(Capability, Access)> = Vec::new();
    let mut entry = Entry::default();
    for place in probe.guarded_places(roots) {
        let capability = place.capability;
        let known = status.access(&capability);
        if all_files || known.allows() {
            continue;
        }
        let access = if may_try(capability) {
            if let Some(&(_, answer)) = tried.iter().find(|(tried, _)| *tried == capability) {
                answer
            } else {
                let answer = probe.try_place(&place);
                tried.push((capability, answer));
                answer
            }
        } else {
            known
        };
        if access == Access::Granted {
            continue;
        }
        entry.avoid.push(place.path);
        if !entry.not_entered.iter().any(|not_entered| not_entered.capability == capability) {
            entry.not_entered.push(NotEntered { capability, access });
        }
    }
    entry
}

/// A volume is the user's own opt-in (a scan root on it); the others need their switch on. Every file can only be
/// granted in the system's settings, never by trying.
fn may_try(capability: Capability, opted_in: &[Capability]) -> bool {
    matches!(capability, Capability::Volume(_)) || opted_in.contains(&capability)
}

/// The coverage of the scan: the places not entered, and the walk's unreadable folders (none when the walk gave no
/// summary). A folder refused by the system's privacy protection is unlocked by every-file access.
pub(crate) fn after_walk(not_entered: Vec<NotEntered>, summary: Option<&WalkSummary>) -> Coverage {
    let counted = summary.into_iter().flat_map(|summary| &summary.unreadable);
    let unreadable = counted
        .filter(|&(_, &folders)| folders > 0)
        .map(|(&reason, &folders)| UnreadableCount {
            reason,
            folders,
            unlocked_by: (reason == Unreadable::Guarded).then_some(Capability::AllFiles),
        })
        .collect();
    Coverage { not_entered, unreadable }
}

#[cfg(test)]
mod tests {
    use fresher_core::{Capability, Event, GuardedFolder};

    use crate::testing::{MemorySettings, ScriptedWalk};

    const DESKTOP: Capability = Capability::Folder(GuardedFolder::Desktop);
    const DOCUMENTS: Capability = Capability::Folder(GuardedFolder::Documents);

    /// The settings with the scan root `/home` and the switches `on` turned on.
    fn opted_into(on: &[&str]) -> MemorySettings {
        let mut values = vec![("core.scan.roots", serde_json::json!(["/home"]))];
        values.extend(on.iter().map(|key| (*key, serde_json::json!(true))));
        MemorySettings::with(&values)
    }

    fn walk() -> ScriptedWalk {
        ScriptedWalk::new()
    }

    fn coverage_event(events: &[Event]) -> Option<&fresher_core::Coverage> {
        events.iter().find_map(|event| match event {
            Event::ScanCoverage { coverage, .. } => Some(coverage),
            _ => None,
        })
    }

    mod negative {
        use std::path::PathBuf;

        use fresher_core::{
            Access, Capability, Coverage, NotEntered, ScanOutcome, Unreadable, UnreadableCount, VolumeKind,
        };
        use fresher_testing::FakeAccess;

        use super::{DESKTOP, DOCUMENTS, coverage_event, opted_into, walk};
        use crate::coverage::after_walk;
        use crate::testing::{Fixture, scan, wait_for_finish};

        #[test]
        fn a_guarded_place_that_is_not_opted_in_is_never_tried() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DESKTOP, Access::NotAsked)])
                .home("/home")
                .guard("Desktop", DESKTOP)
                .answering("Desktop", Access::Granted);
            let walk = walk();
            let avoided = walk.avoided();
            let fixture = Fixture::new(walk).access(access).settings(opted_into(&[]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(probe.tried(), []);
            assert_eq!(*avoided.lock().unwrap(), [vec![PathBuf::from("/home/Desktop")]]);
            let not_entered = vec![NotEntered { capability: DESKTOP, access: Access::NotAsked }];
            assert_eq!(coverage_event(&events), Some(&Coverage { not_entered, unreadable: vec![] }));
        }

        #[test]
        fn an_opted_in_place_that_is_denied_is_skipped_and_reported() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DESKTOP, Access::NotAsked)])
                .home("/home")
                .guard("Desktop", DESKTOP)
                .answering("Desktop", Access::Denied);
            let walk = walk();
            let avoided = walk.avoided();
            let fixture = Fixture::new(walk).access(access).settings(opted_into(&["core.access.desktop"]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(probe.tried().len(), 1);
            assert_eq!(*avoided.lock().unwrap(), [vec![PathBuf::from("/home/Desktop")]]);
            let not_entered = vec![NotEntered { capability: DESKTOP, access: Access::Denied }];
            assert_eq!(coverage_event(&events).map(|coverage| &coverage.not_entered), Some(&not_entered));
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.outcome), Some(ScanOutcome::Partial));
        }

        #[test]
        fn a_restore_tries_no_guarded_place_even_opted_in() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DESKTOP, Access::NotAsked)])
                .home("/home")
                .guard("Desktop", DESKTOP)
                .answering("Desktop", Access::Granted);
            let fixture = Fixture::new(walk()).access(access).settings(opted_into(&["core.access.desktop"]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();
            let events = engine.subscribe();

            engine.restore().unwrap();

            let events = wait_for_finish(&events);
            assert_eq!(probe.tried(), []);
            let not_entered = vec![NotEntered { capability: DESKTOP, access: Access::NotAsked }];
            assert_eq!(coverage_event(&events).map(|coverage| &coverage.not_entered), Some(&not_entered));
        }

        #[test]
        fn a_capability_is_tried_once_and_named_once() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DOCUMENTS, Access::NotAsked)])
                .home("/home")
                .guard("Documents", DOCUMENTS)
                .guard("Documents/Inner", DOCUMENTS)
                .answering("Documents", Access::Denied);
            let fixture = Fixture::new(walk()).access(access).settings(opted_into(&["core.access.documents"]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(probe.tried().len(), 1);
            assert_eq!(coverage_event(&events).map(|coverage| coverage.not_entered.len()), Some(1));
        }

        #[test]
        fn a_volume_that_is_not_granted_is_tried_without_a_switch() {
            let removable = Capability::Volume(VolumeKind::Removable);
            let access = FakeAccess::with(&[(removable, Access::NotAsked)])
                .home("/home")
                .guard("Drive", removable)
                .answering("Drive", Access::Unknown);
            let fixture = Fixture::new(walk()).access(access).settings(opted_into(&[]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(probe.tried().len(), 1);
            let not_entered = vec![NotEntered { capability: removable, access: Access::Unknown }];
            assert_eq!(coverage_event(&events).map(|coverage| &coverage.not_entered), Some(&not_entered));
        }

        #[test]
        fn guarded_folders_make_the_coverage_incomplete_and_the_scan_partial() {
            let walk = walk().unreadable_by(Unreadable::Guarded, 4);
            let engine = Fixture::new(walk).settings(opted_into(&[])).engine();

            let events = scan(&engine);

            let coverage = coverage_event(&events).unwrap();
            assert!(!coverage.is_complete());
            let guarded =
                UnreadableCount { reason: Unreadable::Guarded, folders: 4, unlocked_by: Some(Capability::AllFiles) };
            assert_eq!(coverage.unreadable, [guarded]);
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.outcome), Some(ScanOutcome::Partial));
        }

        #[test]
        fn coverage_holds_no_path() {
            let access = FakeAccess::with(&[(DESKTOP, Access::NotAsked)]).home("/home").guard("Desktop", DESKTOP);
            let walk = walk().unreadable_by(Unreadable::Guarded, 1).unreadable_by(Unreadable::Failed, 2);
            let engine = Fixture::new(walk).access(access).settings(opted_into(&[])).engine();

            let events = scan(&engine);

            let json = serde_json::to_string(coverage_event(&events).unwrap()).unwrap();
            assert!(!json.contains('/'), "{json}");
            let last = serde_json::to_string(&engine.overview().unwrap().last_scan.unwrap().coverage).unwrap();
            assert_eq!(last, json);
        }

        #[test]
        fn a_walk_with_no_summary_counts_no_unreadable_folder() {
            assert_eq!(after_walk(vec![], None), Coverage::default());
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{Access, Capability, Coverage, Event, ScanOutcome, Unreadable, UnreadableCount};
        use fresher_testing::FakeAccess;

        use super::{DESKTOP, DOCUMENTS, coverage_event, opted_into, walk};
        use crate::testing::{Fixture, scan};

        #[test]
        fn with_all_files_granted_no_place_is_tried_and_all_are_walked() {
            let access = FakeAccess::with(&[
                (Capability::AllFiles, Access::Granted),
                (DESKTOP, Access::NotAsked),
                (DOCUMENTS, Access::NotAsked),
            ])
            .home("/home")
            .guard("Desktop", DESKTOP)
            .guard("Documents", DOCUMENTS);
            let walk = walk();
            let avoided = walk.avoided();
            let fixture = Fixture::new(walk).access(access).settings(opted_into(&["core.access.desktop"]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(probe.tried(), []);
            assert_eq!(*avoided.lock().unwrap(), [Vec::<PathBuf>::new()]);
            assert_eq!(coverage_event(&events), Some(&Coverage::default()));
        }

        #[test]
        fn an_opted_in_place_that_is_granted_is_walked() {
            let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied), (DESKTOP, Access::NotAsked)])
                .home("/home")
                .guard("Desktop", DESKTOP)
                .answering("Desktop", Access::Granted);
            let walk = walk();
            let avoided = walk.avoided();
            let fixture = Fixture::new(walk).access(access).settings(opted_into(&["core.access.desktop"]));
            let probe = std::sync::Arc::clone(&fixture.access);
            let engine = fixture.engine();

            let events = scan(&engine);

            assert_eq!(probe.tried().iter().map(|place| place.capability).collect::<Vec<_>>(), [DESKTOP]);
            assert_eq!(*avoided.lock().unwrap(), [Vec::<PathBuf>::new()]);
            assert_eq!(coverage_event(&events), Some(&Coverage::default()));
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.outcome), Some(ScanOutcome::Completed));
        }

        #[test]
        fn guarded_unreadable_folders_are_unlocked_by_all_files() {
            let walk = walk().unreadable_by(Unreadable::Guarded, 2).unreadable_by(Unreadable::NoPermission, 1);
            let engine = Fixture::new(walk).settings(opted_into(&[])).engine();

            let events = scan(&engine);

            assert_eq!(
                coverage_event(&events).map(|coverage| coverage.unreadable.clone()),
                Some(vec![
                    UnreadableCount { reason: Unreadable::NoPermission, folders: 1, unlocked_by: None },
                    UnreadableCount {
                        reason: Unreadable::Guarded,
                        folders: 2,
                        unlocked_by: Some(Capability::AllFiles)
                    },
                ])
            );
        }

        #[test]
        fn coverage_is_sent_before_scan_finished_and_kept_in_the_last_scan() {
            let access = FakeAccess::with(&[(DESKTOP, Access::NotAsked)]).home("/home").guard("Desktop", DESKTOP);
            let engine = Fixture::new(walk()).access(access).settings(opted_into(&[])).engine();

            let events = scan(&engine);

            let tail: Vec<&Event> = events.iter().rev().take(2).collect();
            assert!(matches!(tail[0], Event::ScanFinished { .. }), "{tail:?}");
            let sent = coverage_event(&events).cloned();
            assert!(matches!(tail[1], Event::ScanCoverage { .. }), "{tail:?}");
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.coverage), sent);
        }

        #[test]
        fn a_complete_scan_has_complete_coverage() {
            let engine = Fixture::new(walk()).settings(opted_into(&[])).engine();

            let events = scan(&engine);

            assert_eq!(coverage_event(&events), Some(&Coverage::default()));
            assert_eq!(engine.overview().unwrap().last_scan.map(|last| last.outcome), Some(ScanOutcome::Completed));
        }
    }
}
