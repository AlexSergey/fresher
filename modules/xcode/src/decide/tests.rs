//! The decision on plain values: one test per row, each keep before the offers, the newest versions, then a property.

use std::cell::Cell;
use std::path::Path;

use fresher_core::{ByteSize, EntryFacts, EntryKind, Inside, RunningApp, RunningSet, Sizes};

use crate::decide::{NoSystem, Verdict, decide};
use crate::gathered::Candidate;
use crate::places::{Places, Row};

const DEVELOPER: &str = "/home/Library/Developer";
const IOS: usize = 0;
const WATCH_OS: usize = 1;
const PREVIEWS: usize = 4;
const SIMULATOR_CACHES: usize = 5;
const ARCHIVES: usize = 6;

/// The shipped table's row of index `index`.
fn row(index: usize) -> Row {
    Places::shipped().unwrap().rows()[index].clone()
}

/// A folder of the user's at `path` under `~/Library/Developer`, matched by the row `index`, holding the row's sign,
/// read in full, of 1000 logical bytes.
fn folder(index: usize, path: &str) -> Candidate {
    Candidate {
        row: index,
        path: Path::new(DEVELOPER).join(path),
        kind: EntryKind::Folder,
        sizes: Sizes::complete(ByteSize::new(1000), ByteSize::new(4096)),
        facts: EntryFacts::REMOVABLE,
        holds: row(index).sign,
        inside: Some(Inside::EMPTY),
    }
}

/// An iOS device support folder of the name `name`.
fn ios(name: &str) -> Candidate {
    folder(IOS, &format!("Xcode/iOS DeviceSupport/{name}"))
}

fn running(apps: &[(&str, &[&str])]) -> RunningSet {
    let apps: Vec<RunningApp> =
        apps.iter().map(|(name, ids)| fresher_testing::running_app(name, ids).unwrap()).collect();
    RunningSet::new(apps)
}

/// The verdict on `candidate` with `open` running, and how many times the running apps were asked.
fn verdict(candidate: &Candidate, open: Result<&RunningSet, NoSystem>) -> (Verdict, usize) {
    let asked = Cell::new(0);
    let found = decide(candidate, &row(candidate.row), || {
        asked.set(asked.get() + 1);
        open
    });
    (found, asked.get())
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, Inside, Sizes};

    use super::{ARCHIVES, PREVIEWS, SIMULATOR_CACHES, folder, ios, running, verdict};
    use crate::decide::{Named, NoSystem, Reason, Verdict};
    use crate::gathered::Candidate;

    #[test]
    fn a_file_or_a_link_is_not_claimed() {
        let none = running(&[]);
        for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
            let found = Candidate { kind, ..ios("17.2 (21C62)") };

            assert_eq!(verdict(&found, Ok(&none)), (Verdict::NotClaimed, 0), "{kind:?}");
        }
    }

    #[test]
    fn a_name_of_the_wrong_shape_is_not_claimed() {
        let none = running(&[]);
        for found in [
            ios("Notes"),
            ios("17 (21A1)"),
            folder(ARCHIVES, "Xcode/Archives/March/Shop.xcarchive"),
            folder(ARCHIVES, "Xcode/Archives/2026-03-01/.xcarchive"),
        ] {
            assert_eq!(verdict(&found, Ok(&none)), (Verdict::NotClaimed, 0), "{found:?}");
        }
    }

    #[test]
    fn a_folder_without_its_sign_is_not_claimed() {
        let none = running(&[]);
        for found in [
            ios("17.2 (21C62)"),
            folder(PREVIEWS, "Xcode/UserData/Previews"),
            folder(SIMULATOR_CACHES, "CoreSimulator/Caches"),
            folder(ARCHIVES, "Xcode/Archives/2026-03-01/Shop.xcarchive"),
        ] {
            let bare = Candidate { holds: Vec::new(), ..found.clone() };
            let other = Candidate { holds: vec!["symbols".into()], ..found };

            assert_eq!(verdict(&bare, Ok(&none)), (Verdict::NotClaimed, 0), "{bare:?}");
            assert_eq!(verdict(&other, Ok(&none)), (Verdict::NotClaimed, 0), "{other:?}");
        }
    }

    #[test]
    fn a_folder_read_in_part_is_not_claimed() {
        let none = running(&[]);
        for inside in [None, Some(Inside { complete: false, ..Inside::EMPTY })] {
            let found = Candidate { inside, ..ios("17.2 (21C62)") };

            assert_eq!(verdict(&found, Ok(&none)), (Verdict::NotClaimed, 0), "{inside:?}");
        }
    }

    #[test]
    fn an_empty_folder_is_not_claimed() {
        let empty = Candidate { sizes: Sizes::ZERO, ..folder(SIMULATOR_CACHES, "CoreSimulator/Caches") };

        assert_eq!(verdict(&empty, Ok(&running(&[]))), (Verdict::NotClaimed, 0));
    }

    #[test]
    fn a_socket_inside_keeps_it_special() {
        let inside = Some(Inside { special: true, foreign: true, ..Inside::EMPTY });
        let found = Candidate { inside, ..folder(SIMULATOR_CACHES, "CoreSimulator/Caches") };

        assert_eq!(verdict(&found, Ok(&running(&[]))), (Verdict::LeftOut(Reason::Special, Named::Fixed), 0));
    }

    #[test]
    fn an_entry_not_yours_keeps_it() {
        let none = running(&[]);
        let theirs = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
        let flagged = EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE };
        let restricted = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
        let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        let caches = || folder(SIMULATOR_CACHES, "CoreSimulator/Caches");
        let kept = Verdict::LeftOut(Reason::NotYours, Named::Fixed);
        for facts in [theirs, flagged, restricted, locked] {
            assert_eq!(verdict(&Candidate { facts, ..caches() }, Ok(&none)), (kept.clone(), 0), "{facts:?}");
        }
        let foreign = Candidate { inside: Some(Inside { foreign: true, ..Inside::EMPTY }), ..caches() };
        assert_eq!(verdict(&foreign, Ok(&none)), (kept, 0));
    }

    #[test]
    fn it_is_kept_when_the_running_apps_cannot_be_listed() {
        let found = folder(PREVIEWS, "Xcode/UserData/Previews");

        assert_eq!(verdict(&found, Err(NoSystem)), (Verdict::LeftOut(Reason::NoSystem, Named::Fixed), 1));
    }

    #[test]
    fn it_is_kept_while_xcode_runs() {
        let found = folder(PREVIEWS, "Xcode/UserData/Previews");
        for id in ["com.apple.dt.Xcode", "COM.APPLE.DT.XCODE"] {
            let open = running(&[("Mail", &["com.example.Mail"]), ("Xcode", &[id])]);

            assert_eq!(verdict(&found, Ok(&open)), (Verdict::LeftOut(Reason::AppOpen, Named::Fixed), 1), "{id}");
        }
    }

    #[test]
    fn it_is_kept_while_the_simulator_runs() {
        let found = folder(ARCHIVES, "Xcode/Archives/2026-03-01/Shop.xcarchive");
        let open = running(&[("Simulator", &["com.apple.iphonesimulator"])]);

        let (decided, asked) = verdict(&found, Ok(&open));

        assert!(matches!(decided, Verdict::LeftOut(Reason::AppOpen, Named::Archive(_))), "{decided:?}");
        assert_eq!(asked, 1);
    }
}

mod positive {
    use std::collections::BTreeSet;

    use fresher_core::Inside;

    use super::{ARCHIVES, IOS, PREVIEWS, SIMULATOR_CACHES, WATCH_OS, folder, ios, row, running, verdict};
    use crate::decide::{Named, Reason, Verdict, recent};
    use crate::gathered::Candidate;
    use crate::name::{ArchiveName, Version, device_support};

    #[test]
    fn a_device_support_folder_is_offered() {
        let others = running(&[("Mail", &["com.example.Mail"]), ("Helper", &["com.apple.dt.Xcodex"])]);
        let lock_named =
            Candidate { inside: Some(Inside { lock_named: true, ..Inside::EMPTY }), ..ios("17.2 (21C62)") };
        let offered = Verdict::Offered(Named::DeviceSupport(device_support("17.2 (21C62)".as_ref()).unwrap()));

        assert_eq!(verdict(&ios("17.2 (21C62)"), Ok(&others)), (offered.clone(), 1));
        assert_eq!(verdict(&lock_named, Ok(&others)), (offered, 1));
    }

    #[test]
    fn the_previews_are_offered() {
        let found = folder(PREVIEWS, "Xcode/UserData/Previews");

        assert_eq!(verdict(&found, Ok(&running(&[]))), (Verdict::Offered(Named::Fixed), 1));
    }

    #[test]
    fn the_simulator_caches_are_offered() {
        let found = folder(SIMULATOR_CACHES, "CoreSimulator/Caches");

        assert_eq!(verdict(&found, Ok(&running(&[]))), (Verdict::Offered(Named::Fixed), 1));
    }

    #[test]
    fn an_archive_is_offered() {
        let found = folder(ARCHIVES, "Xcode/Archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive");

        let name = ArchiveName { name: "Shop 01-03-2026, 10.15".into(), date: "2026-03-01".into() };
        assert_eq!(verdict(&found, Ok(&running(&[]))), (Verdict::Offered(Named::Archive(name)), 1));
    }

    /// The verdict on a device support folder of `name`: offered, or left out with `reason`.
    fn support(name: &str, reason: Option<Reason>) -> Verdict {
        let named = Named::DeviceSupport(device_support(name.as_ref()).unwrap());
        match reason {
            Some(reason) => Verdict::LeftOut(reason, named),
            None => Verdict::Offered(named),
        }
    }

    /// The set `recent` gives, as `(platform, version)` texts.
    fn pairs(of: &BTreeSet<(String, Version)>) -> Vec<(&str, String)> {
        of.iter().map(|(platform, version)| (platform.as_str(), version.to_string())).collect()
    }

    #[test]
    fn the_two_newest_versions_of_a_platform_are_recent() {
        let (ios_row, watch_row, archive_row) = (row(IOS), row(WATCH_OS), row(ARCHIVES));
        let decided = [
            (&ios_row, support("17.9 (A1)", None)),
            (&ios_row, support("iPhone15,2 17.10 (A2)", None)),
            (&ios_row, support("17.10 (A3)", None)),
            (&ios_row, support("16.4.1 (A4)", None)),
            (&watch_row, support("10.0 (B1)", None)),
            (&archive_row, Verdict::Offered(Named::Fixed)),
            (&ios_row, Verdict::NotClaimed),
        ];

        let found = recent(decided.iter().map(|(row, verdict)| (*row, verdict)));

        assert_eq!(pairs(&found), [("iOS", "17.9".into()), ("iOS", "17.10".into()), ("watchOS", "10.0".into())]);
    }

    #[test]
    fn a_left_out_version_still_counts_as_newest() {
        let ios_row = row(IOS);
        let decided = [
            (&ios_row, support("18.0 (C1)", Some(Reason::AppOpen))),
            (&ios_row, support("17.2 (C2)", Some(Reason::NoSystem))),
            (&ios_row, support("16.0 (C3)", None)),
        ];

        let found = recent(decided.iter().map(|(row, verdict)| (*row, verdict)));

        assert_eq!(pairs(&found), [("iOS", "17.2".into()), ("iOS", "18.0".into())]);
    }
}

mod property {
    use fresher_core::{ByteSize, EntryFacts, EntryKind, Inside, Sizes};
    use proptest::prelude::*;

    use super::{ARCHIVES, IOS, PREVIEWS, SIMULATOR_CACHES, folder, running, verdict};
    use crate::decide::{NoSystem, Reason, Verdict};
    use crate::gathered::Candidate;

    /// A folder of any place, name and state, most often one the module recognizes.
    fn any_candidate() -> impl Strategy<Value = Candidate> {
        let kind = prop_oneof![6 => Just(EntryKind::Folder), 1 => Just(EntryKind::File), 1 => Just(EntryKind::Symlink)];
        let place = prop_oneof![
            Just((IOS, "Xcode/iOS DeviceSupport/17.2 (21C62)".to_owned())),
            "[0-9. ()A-Za-z]{0,14}".prop_map(|name| (IOS, format!("Xcode/iOS DeviceSupport/{name}"))),
            Just((PREVIEWS, "Xcode/UserData/Previews".to_owned())),
            Just((SIMULATOR_CACHES, "CoreSimulator/Caches".to_owned())),
            Just((ARCHIVES, "Xcode/Archives/2026-03-01/Shop.xcarchive".to_owned())),
            "[0-9-]{1,10}".prop_map(|date| (ARCHIVES, format!("Xcode/Archives/{date}/Shop.xcarchive"))),
        ];
        let flags = proptest::collection::vec(proptest::bool::weighted(0.1), 7);
        let read = prop_oneof![1 => Just(None), 8 => proptest::bool::weighted(0.9).prop_map(Some)];
        (kind, place, flags, read, 0..3_u64).prop_map(|(kind, (row, path), flags, read, bytes)| {
            let facts = EntryFacts {
                owned_by_user: !flags[0],
                user_immutable: flags[1],
                system_immutable: flags[2],
                restricted: flags[3],
                ..EntryFacts::REMOVABLE
            };
            let inside =
                read.map(|complete| Inside { special: flags[4], foreign: flags[5], complete, ..Inside::EMPTY });
            let sizes = Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes));
            let base = folder(row, &path);
            let holds = if flags[6] { Vec::new() } else { base.holds.clone() };
            Candidate { kind, sizes, facts, holds, inside, ..base }
        })
    }

    proptest! {
        #[test]
        fn a_candidate_stopped_before_row_8_never_asks_the_running_apps(
            found in any_candidate(),
            open in proptest::option::weighted(0.8, proptest::bool::ANY),
        ) {
            let xcode = running(&[("Xcode", &["com.apple.dt.Xcode"])]);
            let none = running(&[]);
            let open = match open {
                None => Err(NoSystem),
                Some(true) => Ok(&xcode),
                Some(false) => Ok(&none),
            };

            let (decided, asked) = verdict(&found, open);

            let stopped = matches!(decided, Verdict::NotClaimed | Verdict::LeftOut(Reason::Special | Reason::NotYours, _));
            prop_assert_eq!(asked, usize::from(!stopped));
        }
    }
}
