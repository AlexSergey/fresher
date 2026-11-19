//! The decision on plain values: each row's keep before its offer, then the precedence of the rows, then properties.

use fresher_core::{ByteSize, Inside, InstalledApp, Receipt, RunningApp, Sizes, SystemVersion, Timestamp};
use fresher_testing::{changed_at, installed_app, running_app};

use crate::decide::Scan;
use crate::gathered::Entry;
use crate::installers::Words;
use crate::name::names_app;
use crate::unfinished::{Form, Row, Suffixes};

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_767_225_600;
const GIGABYTE: u64 = 1_000_000_000;
const SONOMA: SystemVersion = SystemVersion { major: 14, minor: 0 };

fn days_ago(days: u64) -> Timestamp {
    Timestamp::from_secs(NOW - days * DAY)
}

/// A walked scan of 30 days that saw Downloads in full.
fn scan() -> Scan {
    Scan { now: Timestamp::from_secs(NOW), days: 30, restored: false, place_complete: true }
}

fn restored() -> Scan {
    Scan { restored: true, ..scan() }
}

/// A file of `bytes` bytes named `name`, unchanged for `days` days.
fn file_of(name: &str, days: u64, bytes: u64) -> Entry {
    let time = Some(days_ago(days));
    let size = ByteSize::new(bytes);
    Entry {
        name: name.into(),
        path: format!("/home/Downloads/{name}").into(),
        kind: Form::File,
        sizes: Sizes::complete(size, size),
        modified: time,
        changed: time,
        inside: None,
        holds: Vec::new(),
    }
}

/// A file of 100 bytes named `name`, unchanged for `days` days.
fn file(name: &str, days: u64) -> Entry {
    file_of(name, days, 100)
}

/// A bundle named `name` holding `holds`, read in full, nothing inside it changed for `days` days.
fn bundle(name: &str, holds: &str, days: u64) -> Entry {
    let time = Some(days_ago(days));
    Entry {
        kind: Form::Bundle,
        inside: Some(Inside { newest_change: time, ..Inside::EMPTY }),
        holds: vec![holds.into()],
        ..file(name, days)
    }
}

/// The shipped row of `suffix`.
fn row(suffix: &str) -> Row {
    Suffixes::shipped().unwrap().rows().iter().find(|row| row.suffix == suffix).unwrap().clone()
}

fn chrome() -> Vec<RunningApp> {
    vec![running_app("Google Chrome", &["com.google.Chrome"]).unwrap()]
}

fn safari() -> Vec<RunningApp> {
    vec![running_app("Safari", &["com.apple.Safari"]).unwrap()]
}

/// Firefox `version`, its bundle changed `days` days ago.
fn firefox(version: &str, days: u64) -> InstalledApp {
    changed_at(installed_app("org.mozilla.firefox", "Firefox", version).unwrap(), NOW - days * DAY)
}

/// What the name rule finds for `stem` among `apps`.
fn by_name(stem: &str, apps: &[InstalledApp]) -> crate::decide::Proof {
    crate::decide::Proof::Name(names_app(stem, apps, &Words::shipped().unwrap()))
}

fn receipt(package: &str, days: u64) -> Receipt {
    Receipt { package_file: package.into(), installed: days_ago(days) }
}

mod negative {
    use fresher_core::{Inside, Timestamp};

    use super::{
        GIGABYTE, NOW, SONOMA, bundle, by_name, chrome, days_ago, file, file_of, firefox, receipt, restored, row,
        safari, scan,
    };
    use crate::decide::{
        AppsState, Proof, Reason, Scan, Verdict, installer, large, macos_installer, placeholder, receipt_proof,
        resumable, unfinished,
    };
    use crate::gathered::Entry;
    use crate::name::Named;

    fn kept(reason: Reason) -> Verdict {
        Verdict::LeftOut(reason)
    }

    #[test]
    fn an_unfinished_file_of_an_open_browser_is_kept() {
        let part = file("movie.mp4.crdownload", 40);

        assert_eq!(unfinished(&part, &row(".crdownload"), Ok(&chrome()), &scan()), kept(Reason::BrowserOpen));
    }

    #[test]
    fn unfinished_files_and_bundles_are_kept_when_the_running_apps_cannot_be_listed() {
        let part = file("movie.mp4.crdownload", 40);
        let download = bundle("trip.zip.download", "Info.plist", 40);

        assert_eq!(unfinished(&part, &row(".crdownload"), Err(()), &scan()), kept(Reason::NoSystem));
        assert_eq!(resumable(&download, &row(".download"), Err(()), &scan()), kept(Reason::NoSystem));
    }

    #[test]
    fn an_unfinished_file_changed_within_the_days_is_kept() {
        for days in [0, 3, 29] {
            let part = file("movie.mp4.part", days);

            assert_eq!(unfinished(&part, &row(".part"), Ok(&[]), &scan()), kept(Reason::Recent), "{days}");
        }
    }

    #[test]
    fn an_unfinished_file_changed_in_the_future_is_kept() {
        let future = Some(Timestamp::from_secs(NOW + 10));
        for part in [
            Entry { changed: future, ..file("a.part", 400) },
            Entry { modified: future, ..file("a.part", 400) },
            Entry { changed: None, ..file("a.part", 400) },
            Entry { modified: None, ..file("a.part", 400) },
        ] {
            assert_eq!(unfinished(&part, &row(".part"), Ok(&[]), &scan()), kept(Reason::Recent), "{part:?}");
        }
    }

    #[test]
    fn a_placeholder_that_is_not_empty_is_never_claimed() {
        let full = file("song.flac", 40);

        assert!(!placeholder(true, Some(&full), &scan()));
        assert!(!placeholder(true, None, &scan()));
        assert!(!placeholder(false, Some(&file_of("song.flac", 40, 0)), &scan()));
        assert!(!placeholder(true, Some(&Entry { holds: Vec::new(), ..bundle("song.flac", "x", 40) }), &scan()));
    }

    #[test]
    fn no_placeholder_when_the_place_was_not_seen_in_full() {
        let empty = file_of("song.flac", 40, 0);

        assert!(!placeholder(true, Some(&empty), &Scan { place_complete: false, ..scan() }));
        assert!(!placeholder(true, Some(&empty), &restored()));
    }

    #[test]
    fn a_resumable_download_without_its_info_plist_is_not_claimed() {
        let download = bundle("trip.download", "Contents", 40);

        assert_eq!(resumable(&download, &row(".download"), Ok(&[]), &scan()), Verdict::NotClaimed);
        assert_eq!(resumable(&file("trip.download", 40), &row(".download"), Ok(&[]), &scan()), Verdict::NotClaimed);
    }

    #[test]
    fn a_resumable_download_of_an_open_safari_or_changed_lately_is_kept() {
        let download = bundle("trip.zip.download", "Info.plist", 40);
        let lately = bundle("trip.zip.download", "Info.plist", 3);

        assert_eq!(resumable(&download, &row(".download"), Ok(&safari()), &scan()), kept(Reason::BrowserOpen));
        assert_eq!(resumable(&lately, &row(".download"), Ok(&[]), &scan()), kept(Reason::Recent));
    }

    #[test]
    fn a_bundle_read_in_part_is_kept_to_scan_again() {
        let in_part = |name: &str, holds: &str| {
            let read = bundle(name, holds, 40);
            Entry { inside: read.inside.map(|inside| Inside { complete: false, ..inside }), ..read }
        };

        let download = in_part("trip.zip.download", "Info.plist");
        let macos = in_part("Install macOS Sonoma.app", "Contents");
        assert_eq!(resumable(&download, &row(".download"), Ok(&[]), &scan()), kept(Reason::ScanAgain));
        assert_eq!(macos_installer(&macos, SONOMA, Ok(SONOMA), &scan()), kept(Reason::ScanAgain));
    }

    #[test]
    fn a_bundle_holding_a_special_or_foreign_entry_is_kept() {
        let with = |name: &str, holds: &str, inside: Inside| Entry {
            inside: Some(Inside { newest_change: Some(days_ago(40)), ..inside }),
            ..bundle(name, holds, 40)
        };
        let special = Inside { special: true, ..Inside::EMPTY };
        let foreign = Inside { foreign: true, ..Inside::EMPTY };
        let (download, macos) = (("trip.zip.download", "Info.plist"), ("Install macOS Sonoma.app", "Contents"));

        let resume = |inside| resumable(&with(download.0, download.1, inside), &row(".download"), Ok(&[]), &scan());
        let old_os = |inside| macos_installer(&with(macos.0, macos.1, inside), SONOMA, Ok(SONOMA), &scan());
        assert_eq!(resume(special), kept(Reason::Special));
        assert_eq!(resume(foreign), kept(Reason::NotYours));
        assert_eq!(old_os(special), kept(Reason::Special));
        assert_eq!(old_os(foreign), kept(Reason::NotYours));
    }

    #[test]
    fn a_macos_installer_without_contents_is_not_claimed() {
        let app = bundle("Install macOS Sonoma.app", "Info.plist", 40);

        assert_eq!(macos_installer(&app, SONOMA, Ok(SONOMA), &scan()), Verdict::NotClaimed);
    }

    #[test]
    fn a_macos_installer_on_an_unknown_macos_is_kept() {
        let app = bundle("Install macOS Sonoma.app", "Contents", 40);

        assert_eq!(macos_installer(&app, SONOMA, Err(()), &scan()), kept(Reason::NoSystem));
    }

    #[test]
    fn a_macos_installer_for_a_newer_macos_is_kept() {
        let app = bundle("Install macOS Tahoe.app", "Contents", 40);
        let tahoe = fresher_core::SystemVersion { major: 26, minor: 0 };
        let running = fresher_core::SystemVersion { major: 15, minor: 4 };

        assert_eq!(macos_installer(&app, tahoe, Ok(running), &scan()), kept(Reason::NewerMacos));
    }

    #[test]
    fn an_installer_whose_app_changed_before_it_is_kept() {
        let apps = [firefox("133.0", 30)];
        for dmg in [file("Firefox 133.0.dmg", 20), Entry { changed: None, ..file("Firefox 133.0.dmg", 40) }] {
            let proof = by_name("Firefox 133.0", &apps);
            assert_eq!(installer(&dmg, proof, AppsState::Complete(&apps), &scan()), kept(Reason::NotOlder), "{dmg:?}");
        }
        let unknown = [fresher_core::InstalledApp { changed: None, ..firefox("133.0", 10) }];
        let proof = by_name("Firefox 133.0", &unknown);
        assert_eq!(
            installer(&file("Firefox 133.0.dmg", 20), proof, AppsState::Complete(&unknown), &scan()),
            kept(Reason::NotOlder)
        );
    }

    #[test]
    fn an_installer_newer_than_its_app_is_kept() {
        let apps = [firefox("133.0", 10)];
        let dmg = file("Firefox 140.0.dmg", 20);

        let proof = by_name("Firefox 140.0", &apps);
        assert_eq!(installer(&dmg, proof, AppsState::Complete(&apps), &scan()), kept(Reason::NotOlder));
    }

    #[test]
    fn an_installer_whose_version_cannot_be_compared_with_its_apps_is_kept() {
        for version in ["1.0b2", "beta"] {
            let apps = [firefox(version, 10)];
            let dmg = file("Firefox 133.0.dmg", 20);

            let proof = by_name("Firefox 133.0", &apps);
            assert_eq!(installer(&dmg, proof, AppsState::Complete(&apps), &scan()), kept(Reason::NotOlder));
        }
        let none = [fresher_core::InstalledApp { version: None, ..firefox("1", 10) }];
        let proof = by_name("Firefox 133.0", &none);
        assert_eq!(
            installer(&file("Firefox 133.0.dmg", 20), proof, AppsState::Complete(&none), &scan()),
            kept(Reason::NotOlder)
        );
    }

    #[test]
    fn an_installer_when_the_app_list_is_partial_is_kept() {
        let apps = [firefox("133.0", 10)];
        let dmg = file("Firefox 133.0.dmg", 20);

        assert_eq!(
            installer(&dmg, by_name("Firefox 133.0", &apps), AppsState::Partial, &scan()),
            kept(Reason::AppsUnread)
        );
    }

    #[test]
    fn an_installer_when_the_apps_cannot_be_listed_is_kept() {
        let dmg = file("Firefox 133.0.dmg", 20);

        assert_eq!(installer(&dmg, Proof::Name(Named::None), AppsState::Unlisted, &scan()), kept(Reason::NoSystem));
    }

    #[test]
    fn an_installer_naming_no_app_is_not_claimed() {
        let apps = [firefox("133.0", 10)];
        for stem in ["install", "notes-backup", "Firefox (1)"] {
            let dmg = file(&format!("{stem}.dmg"), 20);

            let complete = installer(&dmg, by_name(stem, &apps), AppsState::Complete(&apps), &scan());
            let partial = installer(&dmg, by_name(stem, &apps), AppsState::Partial, &scan());
            assert_eq!((complete, partial), (Verdict::NotClaimed, Verdict::NotClaimed), "{stem}");
        }
        let several = Proof::Name(Named::Several);
        assert_eq!(installer(&file("x.dmg", 20), several, AppsState::Complete(&apps), &scan()), Verdict::NotClaimed);
    }

    #[test]
    fn a_receipt_older_than_the_file_or_of_another_name_proves_nothing() {
        let pkg = file("Tool-3.1.pkg", 20);

        assert_eq!(receipt_proof(&pkg, &[receipt("Tool-3.1.pkg", 30), receipt("Other.pkg", 10)]), None);
        assert_eq!(receipt_proof(&Entry { changed: None, ..pkg }, &[receipt("Tool-3.1.pkg", 10)]), None);
    }

    #[test]
    fn a_restored_scan_keeps_everything_it_would_offer() {
        let apps = [firefox("133.0", 10)];
        let empty = file_of("song.flac", 40, 0);

        assert_eq!(unfinished(&file("a.part", 40), &row(".part"), Ok(&[]), &restored()), kept(Reason::ScanAgain));
        assert!(!placeholder(true, Some(&empty), &restored()));
        let download = bundle("trip.zip.download", "Info.plist", 40);
        assert_eq!(resumable(&download, &row(".download"), Ok(&[]), &restored()), kept(Reason::ScanAgain));
        let macos = bundle("Install macOS Sonoma.app", "Contents", 40);
        assert_eq!(macos_installer(&macos, SONOMA, Ok(SONOMA), &restored()), kept(Reason::ScanAgain));
        let dmg = file("Firefox 133.0.dmg", 20);
        let by_firefox = by_name("Firefox 133.0", &apps);
        assert_eq!(installer(&dmg, by_firefox, AppsState::Complete(&apps), &restored()), kept(Reason::ScanAgain));
        let receipt = Proof::Receipt { installed: days_ago(10) };
        assert_eq!(
            installer(&file("Tool.pkg", 20), receipt, AppsState::Unlisted, &restored()),
            kept(Reason::ScanAgain)
        );
        assert_eq!(large(&file_of("video.mov", 400, 2 * GIGABYTE), &restored()), kept(Reason::ScanAgain));
    }

    #[test]
    fn a_dmg_never_falls_to_large_untouched() {
        for name in ["huge.dmg", "huge.DMG", "Huge.Pkg", ".dmg"] {
            assert_eq!(large(&file_of(name, 400, 20 * GIGABYTE), &scan()), Verdict::NotClaimed, "{name}");
        }
    }

    #[test]
    fn a_big_file_changed_within_the_days_is_kept() {
        let video = file_of("video.mov", 5, 2 * GIGABYTE);
        let unknown = Entry { changed: None, ..file_of("video.mov", 400, 2 * GIGABYTE) };

        assert_eq!(large(&video, &scan()), kept(Reason::Recent));
        assert_eq!(large(&unknown, &scan()), kept(Reason::Recent));
    }

    #[test]
    fn a_file_under_one_gigabyte_is_not_claimed() {
        assert_eq!(large(&file_of("video.mov", 400, GIGABYTE - 1), &scan()), Verdict::NotClaimed);
        assert_eq!(large(&file_of("video.mov", 400, 900_000_000), &restored()), Verdict::NotClaimed);
        let folder = Entry { kind: crate::unfinished::Form::Bundle, ..file_of("big", 400, 2 * GIGABYTE) };
        assert_eq!(large(&folder, &scan()), Verdict::NotClaimed);
    }

    mod precedence {
        use super::super::{GIGABYTE, bundle, by_name, chrome, file, file_of, firefox, restored, row};
        use crate::decide::{
            AppsState, Proof, Reason, Verdict, installer, large, macos_installer, resumable, unfinished,
        };
        use crate::name::Named;

        #[test]
        fn restored_comes_before_everything() {
            let scan_again = Verdict::LeftOut(Reason::ScanAgain);
            let part = file("a.crdownload", 3);
            assert_eq!(unfinished(&part, &row(".crdownload"), Err(()), &restored()), scan_again);
            assert_eq!(unfinished(&part, &row(".crdownload"), Ok(&chrome()), &restored()), scan_again);
            let download = bundle("trip.zip.download", "Info.plist", 3);
            assert_eq!(resumable(&download, &row(".download"), Err(()), &restored()), scan_again);
            let macos = bundle("Install macOS Tahoe.app", "Contents", 40);
            let tahoe = fresher_core::SystemVersion { major: 26, minor: 0 };
            assert_eq!(macos_installer(&macos, tahoe, Err(()), &restored()), scan_again);
            assert_eq!(large(&file_of("video.mov", 3, 2 * GIGABYTE), &restored()), scan_again);
            let apps = [firefox("133.0", 30)];
            let newer = file("Firefox 140.0.dmg", 20);
            let proof = by_name("Firefox 140.0", &apps);
            assert_eq!(installer(&newer, proof, AppsState::Complete(&apps), &restored()), scan_again);
        }

        /// The installer table's rows 1 to 3 come before a restored scan (design, section 3.4).
        #[test]
        fn an_installer_not_claimed_unlisted_or_partial_stays_so_in_a_restored_scan() {
            let apps = [firefox("133.0", 10)];
            let dmg = file("Firefox 133.0.dmg", 20);
            let none = Proof::Name(Named::None);

            assert_eq!(installer(&dmg, none.clone(), AppsState::Complete(&apps), &restored()), Verdict::NotClaimed);
            assert_eq!(installer(&dmg, none, AppsState::Unlisted, &restored()), Verdict::LeftOut(Reason::NoSystem));
            let proof = by_name("Firefox 133.0", &apps);
            assert_eq!(installer(&dmg, proof, AppsState::Partial, &restored()), Verdict::LeftOut(Reason::AppsUnread));
        }

        #[test]
        fn an_open_browser_comes_before_the_age() {
            let part = file("a.crdownload", 3);

            assert_eq!(
                unfinished(&part, &row(".crdownload"), Ok(&chrome()), &super::super::scan()),
                Verdict::LeftOut(Reason::BrowserOpen)
            );
        }
    }

    mod properties {
        use fresher_core::{RunningApp, Timestamp};
        use proptest::prelude::*;

        use super::super::{GIGABYTE, NOW, by_name, chrome, file, file_of, firefox, row, safari, scan};
        use crate::decide::{AppsState, Verdict, installer, large, resumable, unfinished};
        use crate::gathered::Entry;

        fn offered(verdict: &Verdict) -> bool {
            matches!(verdict, Verdict::Offered { .. })
        }

        fn time() -> impl Strategy<Value = Option<Timestamp>> {
            prop_oneof![Just(None), (0..800 * super::super::DAY).prop_map(|ago| Some(Timestamp::from_secs(NOW - ago)))]
        }

        fn running() -> impl Strategy<Value = Vec<RunningApp>> {
            prop_oneof![Just(Vec::new()), Just(chrome()), Just(safari())]
        }

        proptest! {
            #[test]
            fn adding_a_running_browser_never_turns_a_keep_into_an_offer(
                modified in time(), changed in time(), running in running(), days in 7..400u32,
            ) {
                let scan = crate::decide::Scan { days, ..scan() };
                let part = Entry { modified, changed, ..file("a.crdownload", 0) };
                let download = Entry { modified, changed, ..super::super::bundle("t.download", "Info.plist", 0) };
                let download = Entry { inside: download.inside.map(|inside| fresher_core::Inside { newest_change: changed, ..inside }), ..download };
                let (part_row, bundle_row) = (row(".crdownload"), row(".download"));
                let mut more = running.clone();
                more.extend(chrome());
                more.extend(safari());

                if !offered(&unfinished(&part, &part_row, Ok(&running), &scan)) {
                    prop_assert!(!offered(&unfinished(&part, &part_row, Ok(&more), &scan)));
                }
                if !offered(&resumable(&download, &bundle_row, Ok(&running), &scan)) {
                    prop_assert!(!offered(&resumable(&download, &bundle_row, Ok(&more), &scan)));
                }
                prop_assert!(!offered(&unfinished(&part, &part_row, Ok(&more), &scan)));
            }

            #[test]
            fn a_later_change_never_turns_a_keep_into_an_offer(
                modified in time(), changed in time(), later in 0..800 * super::super::DAY, days in 7..400u32,
            ) {
                let scan = crate::decide::Scan { days, ..scan() };
                let move_on = |time: Option<Timestamp>| time.map(|time| Timestamp::from_secs(time.secs() + later));
                let before = Entry { modified, changed, ..file_of("video.mov", 0, 2 * GIGABYTE) };
                let after = Entry { modified: move_on(modified), changed: move_on(changed), ..before.clone() };
                let part_before = Entry { name: "a.part".into(), ..before.clone() };
                let part_after = Entry { name: "a.part".into(), ..after.clone() };

                if !offered(&large(&before, &scan)) {
                    prop_assert!(!offered(&large(&after, &scan)));
                }
                if !offered(&unfinished(&part_before, &row(".part"), Ok(&[]), &scan)) {
                    prop_assert!(!offered(&unfinished(&part_after, &row(".part"), Ok(&[]), &scan)));
                }
            }

            #[test]
            fn a_later_app_change_never_turns_an_offer_into_a_keep(
                app_days in 0..400u64, earlier in 0..400u64, installer_days in 0..400u64,
                version in prop_oneof![Just("133.0"), Just("140.1"), Just("120")],
            ) {
                let dmg = file("Firefox 133.0.dmg", installer_days);
                let then = [firefox(version, app_days)];
                let later = [firefox(version, app_days.saturating_sub(earlier))];
                let first = installer(&dmg, by_name("Firefox 133.0", &then), AppsState::Complete(&then), &scan());
                let second = installer(&dmg, by_name("Firefox 133.0", &later), AppsState::Complete(&later), &scan());

                if offered(&first) {
                    prop_assert!(offered(&second), "{first:?} then {second:?}");
                }
            }
        }
    }
}

mod positive {
    use fresher_core::{BundleId, ByteSize, DangerLevel};

    use super::{GIGABYTE, SONOMA, bundle, by_name, days_ago, file, file_of, firefox, receipt, row, scan};
    use crate::categories::Category;
    use crate::decide::{
        AppsState, OfferReason, Proof, Verdict, installer, large, macos_installer, placeholder, placeholder_name,
        receipt_proof, resumable, unfinished,
    };

    fn offered(reason: OfferReason) -> Verdict {
        Verdict::Offered { reason }
    }

    #[test]
    fn an_old_unfinished_download_of_a_closed_browser_is_offered() {
        for (name, suffix) in [
            ("a.crdownload", ".crdownload"),
            ("b.mp4.part", ".part"),
            ("c.opdownload", ".opdownload"),
            ("d.partial", ".partial"),
        ] {
            let part = file(name, 30);

            let verdict = unfinished(&part, &row(suffix), Ok(&super::safari()), &scan());
            assert_eq!(verdict, offered(OfferReason::Unfinished { days: 30 }), "{name}");
        }
    }

    #[test]
    fn its_empty_placeholder_is_offered_with_it() {
        let part = file("song.flac.part", 40);

        assert_eq!(placeholder_name(&part, &row(".part")), Some("song.flac"));
        assert_eq!(placeholder_name(&file("x.crdownload", 40), &row(".crdownload")), None);
        assert!(placeholder(true, Some(&file_of("song.flac", 40, 0)), &scan()));
    }

    #[test]
    fn an_old_resumable_download_is_offered() {
        let download = bundle("trip.zip.download", "Info.plist", 30);

        let verdict = resumable(&download, &row(".download"), Ok(&super::chrome()), &scan());
        assert_eq!(verdict, offered(OfferReason::SafariResumable { days: 30 }));
    }

    #[test]
    fn a_proven_installer_is_offered_for_its_app() {
        for (stem, app_version) in [("Firefox 133.0", "133.0"), ("Firefox 133.0", "133.0.1"), ("firefox", "140.0")] {
            let apps = [firefox(app_version, 10)];
            let dmg = file(&format!("{stem}.dmg"), 20);

            let verdict = installer(&dmg, by_name(stem, &apps), AppsState::Complete(&apps), &scan());
            let app = BundleId::new("org.mozilla.firefox").unwrap();
            let reason = OfferReason::Installer { app, name: "Firefox".into(), version: Some(app_version.into()) };
            assert_eq!(verdict, offered(reason), "{stem} with {app_version}");
        }
        let at_once = [firefox("133.0", 20)];
        let verdict =
            installer(&file("Firefox.dmg", 20), by_name("Firefox", &at_once), AppsState::Complete(&at_once), &scan());
        assert!(matches!(verdict, Verdict::Offered { .. }));
    }

    #[test]
    fn an_installer_of_an_app_without_a_version_is_offered_without_one() {
        let apps = [fresher_core::InstalledApp { version: None, ..firefox("1", 10) }];

        let verdict =
            installer(&file("Firefox.dmg", 20), by_name("Firefox", &apps), AppsState::Complete(&apps), &scan());
        let app = BundleId::new("org.mozilla.firefox").unwrap();
        assert_eq!(verdict, offered(OfferReason::Installer { app, name: "Firefox".into(), version: None }));
    }

    #[test]
    fn a_package_with_its_receipt_is_offered_without_an_app() {
        let pkg = file("Tool-3.1.pkg", 20);

        let installed = receipt_proof(&pkg, &[receipt("tool-3.1.PKG", 10), receipt("Tool-3.1.pkg", 20)]);
        assert_eq!(installed, Some(days_ago(10)));
        let verdict = installer(&pkg, Proof::Receipt { installed: days_ago(10) }, AppsState::Partial, &scan());
        assert_eq!(verdict, offered(OfferReason::InstallerReceipt { installed: days_ago(10) }));
    }

    #[test]
    fn an_old_macos_installer_is_offered() {
        let app = bundle("Install macOS Sonoma.app", "Contents", 1);
        let running = fresher_core::SystemVersion { major: 15, minor: 1 };

        assert_eq!(
            macos_installer(&app, SONOMA, Ok(running), &scan()),
            offered(OfferReason::MacosInstaller { running })
        );
        assert_eq!(
            macos_installer(&app, SONOMA, Ok(SONOMA), &scan()),
            offered(OfferReason::MacosInstaller { running: SONOMA })
        );
    }

    #[test]
    fn a_big_untouched_file_is_offered_at_high() {
        for bytes in [GIGABYTE, 2 * GIGABYTE] {
            let video = file_of("video.mov", 30, bytes);

            let reason = OfferReason::LargeUntouched { size: ByteSize::new(bytes), since: days_ago(30) };
            assert_eq!(large(&video, &scan()), offered(reason));
        }
        assert_eq!(Category::LargeUntouched.danger(), DangerLevel::High);
    }

    mod precedence {
        use super::super::{days_ago, file, scan};
        use crate::decide::{AppsState, OfferReason, Proof, Verdict, installer};

        #[test]
        fn a_receipt_proof_needs_no_app_list() {
            let pkg = file("Tool-3.1.pkg", 20);

            for apps in [AppsState::Unlisted, AppsState::Partial, AppsState::Complete(&[])] {
                let verdict = installer(&pkg, Proof::Receipt { installed: days_ago(10) }, apps, &scan());
                let reason = OfferReason::InstallerReceipt { installed: days_ago(10) };
                assert_eq!(verdict, Verdict::Offered { reason });
            }
        }
    }
}
