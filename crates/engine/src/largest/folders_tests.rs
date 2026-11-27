use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ByteSize, CategoryId, DangerLevel, FoldersPage, FoldersQuery, ModuleId, NodeKind, Protection, Sizes, StateId,
    Timestamp,
};
use fresher_testing::FixedDiskState;
use serde_json::json;

use crate::Engine;
use crate::testing::{Fixture, MemorySettings, Protecting, ScriptedWalk, TestModule, scan};

const MB: u64 = 1_000_000;
const DAY: u64 = 86_400;
/// When the fixture's state was taken.
const TAKEN: u64 = 1_000 * DAY;

fn caches() -> CategoryId {
    CategoryId::new(ModuleId::new("alpha").unwrap(), "alpha.caches").unwrap()
}

/// The time `days` before the state was taken.
fn ago(days: u64) -> Timestamp {
    Timestamp::from_secs(TAKEN - days * DAY)
}

/// A state of the folders and files given, each of its size in MB, and the days since each path's newest change.
fn tree(nodes: &[(&str, NodeKind, u64)], newest: &[(&str, u64)]) -> FixedDiskState {
    let nodes = nodes.iter().map(|&(path, kind, size)| {
        (PathBuf::from(path), kind, Sizes::complete(ByteSize::new(size * MB), ByteSize::new(size * MB)))
    });
    let state = FixedDiskState::new(StateId::new(1), Timestamp::from_secs(TAKEN), nodes);
    newest.iter().fold(state, |state, &(path, days)| state.with_newest(Path::new(path), ago(days)))
}

/// An engine with the module `alpha` that has scanned `state` once, with `settings` stored and `protection` in the
/// cleaner when given.
fn engine_of(state: FixedDiskState, settings: MemorySettings, protection: Option<Protection>) -> Engine {
    let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe);
    let projects = TestModule::builder("stale-projects").stale_days_setting();
    let walk = ScriptedWalk::new().committing(vec![Arc::new(state)]);
    let mut fixture = Fixture::new(walk).module(alpha).module(projects).settings(settings);
    if let Some(protection) = protection {
        fixture = fixture.cleaner(Arc::new(Protecting::new(protection)));
    }
    let engine = fixture.engine();
    scan(&engine);
    engine
}

fn scanned(state: FixedDiskState) -> Engine {
    engine_of(state, MemorySettings::default(), None)
}

fn protecting(never: &[&str], whole: &[&str]) -> Protection {
    let paths = |paths: &[&str]| paths.iter().map(PathBuf::from).collect();
    Protection::new(paths(never), paths(whole), Vec::new()).unwrap()
}

fn ignoring(paths: &[&str]) -> MemorySettings {
    MemorySettings::with(&[("core.ignore.paths", json!(paths))])
}

const fn query() -> FoldersQuery {
    FoldersQuery { offset: 0, limit: 100 }
}

fn page(engine: &Engine) -> FoldersPage {
    engine.largest_folders(query()).unwrap()
}

/// The rows' places and names, `~/Code/app`.
fn listed(page: &FoldersPage) -> Vec<String> {
    page.rows.iter().map(|row| format!("{}/{}", row.place, row.node.name)).collect()
}

mod negative {
    use std::time::Duration;

    use fresher_core::{CommandError, Event, NodeKind, Operation};

    use super::*;
    use crate::testing::engine;

    #[test]
    fn no_scan_is_refused() {
        let engine = engine(ScriptedWalk::new());

        assert_eq!(engine.largest_folders(query()), Err(CommandError::NoScan));
    }

    #[test]
    fn a_running_scan_is_refused() {
        let state = tree(&[("/home", NodeKind::Folder, 5_000)], &[]);
        let (walk, gate) = ScriptedWalk::new().committing(vec![Arc::new(state)]).hold();
        gate.release();
        let engine = Fixture::new(walk).engine();
        scan(&engine);
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        while !matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), Event::ScanStarted { .. }) {}

        assert_eq!(engine.largest_folders(query()), Err(CommandError::Busy { running: Operation::Scan }));
        engine.cancel_scan().unwrap();
    }

    #[test]
    fn a_state_without_the_home_answers_an_empty_page() {
        let engine = scanned(tree(&[("/elsewhere", NodeKind::Folder, 5_000)], &[("/elsewhere", 900)]));

        let page = engine.largest_folders(FoldersQuery { offset: 7, limit: 10 }).unwrap();

        assert_eq!((page.rows.len(), page.total, page.offset, page.projects), (0, 0, 7, 0));
        assert_eq!((page.min_size.bytes(), page.days), (1_000 * MB, 365));
    }

    #[test]
    fn the_home_itself_is_never_listed() {
        let engine = scanned(tree(&[("/home", NodeKind::Folder, 5_000)], &[("/home", 900)]));

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn the_library_and_the_trash_are_never_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Library", NodeKind::Folder, 3_000),
            ("/home/Library/Mail", NodeKind::Folder, 2_000),
            ("/home/.Trash", NodeKind::Folder, 3_000),
        ];
        let newest = [("/home/Library", 900), ("/home/Library/Mail", 900), ("/home/.Trash", 900)];

        assert_eq!(page(&scanned(tree(&nodes, &newest))).total, 0);
    }

    #[test]
    fn a_hidden_folder_an_app_bundle_or_a_package_is_never_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/.cache", NodeKind::Folder, 2_000),
            ("/home/Apps", NodeKind::Folder, 2_000),
            ("/home/Apps/Tool.app", NodeKind::Folder, 2_000),
            ("/home/Pictures", NodeKind::Folder, 3_000),
            ("/home/Pictures/Trip.photoslibrary", NodeKind::Folder, 3_000),
        ];
        let newest = [
            ("/home/.cache", 900),
            ("/home/Apps/Tool.app", 900),
            ("/home/Pictures/Trip.photoslibrary", 900),
            ("/home/Apps", 5),
            ("/home/Pictures", 5),
        ];

        assert_eq!(page(&scanned(tree(&nodes, &newest))).total, 0);
    }

    #[test]
    fn a_standard_folder_itself_is_never_listed() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Documents", NodeKind::Folder, 3_000)];
        let engine = engine_of(
            tree(&nodes, &[("/home/Documents", 900)]),
            MemorySettings::default(),
            Some(protecting(&[], &["/home/Documents"])),
        );

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn a_protected_folder_and_what_is_inside_are_never_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Vault", NodeKind::Folder, 3_000),
            ("/home/Vault/inner", NodeKind::Folder, 2_000),
            ("/home/Work", NodeKind::Folder, 3_000),
            ("/home/Work/Secret", NodeKind::Folder, 2_000),
        ];
        let newest = [("/home/Vault", 900), ("/home/Vault/inner", 900), ("/home/Work", 900)];
        let engine = engine_of(
            tree(&nodes, &newest),
            MemorySettings::default(),
            Some(protecting(&["/home/Vault", "/home/Work/Secret"], &[])),
        );

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn an_ignored_folder_is_never_listed() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Old", NodeKind::Folder, 3_000)];
        let engine = engine_of(tree(&nodes, &[("/home/Old", 900)]), ignoring(&["/home/Old"]), None);

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn a_folder_explained_whole_is_never_listed() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Caches", NodeKind::Folder, 3_000)];
        let state = tree(&nodes, &[("/home/Caches", 900)]).with_owner(Path::new("/home/Caches"), caches());

        assert_eq!(page(&scanned(state)).total, 0);
    }

    #[test]
    fn an_ignored_folder_holding_explained_items_is_never_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Work", NodeKind::Folder, 3_000),
            ("/home/Work/node_modules", NodeKind::Folder, 2_000),
        ];
        let state = tree(&nodes, &[("/home/Work", 900)]).with_owner(Path::new("/home/Work/node_modules"), caches());
        let engine = engine_of(state, ignoring(&["/home/Work"]), None);

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn a_folder_under_an_ignored_folder_is_never_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Old", NodeKind::Folder, 3_000),
            ("/home/Old/inner", NodeKind::Folder, 2_000),
        ];
        let engine = engine_of(tree(&nodes, &[("/home/Old/inner", 900)]), ignoring(&["/home/Old"]), None);

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn a_folder_a_cleanup_removed_is_not_listed() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Gone", NodeKind::Folder, 3_000)];
        let engine = scanned(tree(&nodes, &[("/home/Gone", 900)]));
        assert_eq!(page(&engine).total, 1);

        engine.shared.results.lock().unwrap().remove(&[], &[], &[PathBuf::from("/home/Gone")]);

        assert_eq!(page(&engine).total, 0);
    }

    #[test]
    fn a_folder_with_a_recent_change_is_not_listed() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Fresh", NodeKind::Folder, 3_000)];

        assert_eq!(page(&scanned(tree(&nodes, &[("/home/Fresh", 10)]))).total, 0);
    }

    #[test]
    fn a_folder_below_the_setting_is_not_listed() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Small", NodeKind::Folder, 600)];
        let state = tree(&nodes, &[("/home/Small", 900)]);

        assert_eq!(page(&scanned(state)).total, 0);
    }
}

mod positive {
    use fresher_core::{LeftFor, NodeKind};

    use super::*;

    fn caches_setting(choice: &str) -> MemorySettings {
        MemorySettings::with(&[("core.largest.folder-min-size", json!(choice))])
    }

    #[test]
    fn a_stale_folder_is_listed_with_its_place_and_age() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Code", NodeKind::Folder, 4_000),
            ("/home/Code/app", NodeKind::Folder, 2_500),
        ];
        let page = page(&scanned(tree(&nodes, &[("/home/Code/app", 400)])));

        assert_eq!(listed(&page), ["~/Code/app"]);
        let row = &page.rows[0];
        assert_eq!((row.newest, row.left_for, row.project), (ago(400), LeftFor::Months(13), false));
        assert_eq!((page.home.allocated.bytes(), page.min_size.bytes(), page.days), (9_000 * MB, 1_000 * MB, 365));
    }

    #[test]
    fn the_stale_children_of_a_standard_folder_are_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Documents", NodeKind::Folder, 5_000),
            ("/home/Documents/a", NodeKind::Folder, 2_000),
            ("/home/Documents/b", NodeKind::Folder, 2_000),
        ];
        let newest = [("/home/Documents", 400), ("/home/Documents/a", 400), ("/home/Documents/b", 5)];
        let engine =
            engine_of(tree(&nodes, &newest), MemorySettings::default(), Some(protecting(&[], &["/home/Documents"])));

        assert_eq!(listed(&page(&engine)), ["~/Documents/a"]);
    }

    #[test]
    fn a_folder_holding_an_ignored_path_lists_its_other_stale_children() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Work", NodeKind::Folder, 5_000),
            ("/home/Work/keep", NodeKind::Folder, 2_000),
            ("/home/Work/other", NodeKind::Folder, 2_000),
        ];
        let newest = [("/home/Work", 400), ("/home/Work/keep", 400), ("/home/Work/other", 400)];
        let engine = engine_of(tree(&nodes, &newest), ignoring(&["/home/Work/keep"]), None);

        assert_eq!(listed(&page(&engine)), ["~/Work/other"]);
    }

    #[test]
    fn a_folder_holding_explained_items_is_listed_whole() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Work", NodeKind::Folder, 3_000),
            ("/home/Work/node_modules", NodeKind::Folder, 2_000),
        ];
        let state = tree(&nodes, &[("/home/Work", 400)]).with_owner(Path::new("/home/Work/node_modules"), caches());

        let page = page(&scanned(state));

        assert_eq!(listed(&page), ["~/Work"]);
        assert_eq!(page.rows[0].node.sizes.allocated.bytes(), 3_000 * MB);
    }

    #[test]
    fn a_folder_with_a_git_folder_is_a_project() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/app", NodeKind::Folder, 3_000),
            ("/home/app/.git", NodeKind::Folder, 10),
            ("/home/plain", NodeKind::Folder, 2_000),
        ];
        let page = page(&scanned(tree(&nodes, &[("/home/app", 400), ("/home/plain", 400)])));

        let projects: Vec<(&str, bool)> = page.rows.iter().map(|row| (row.node.name.as_str(), row.project)).collect();
        assert_eq!(projects, [("app", true), ("plain", false)]);
        assert_eq!(page.projects, 1);
    }

    #[test]
    fn a_folder_with_a_collapsed_git_is_a_project() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/app", NodeKind::Folder, 3_000),
            ("/home/app/.git", NodeKind::Collapsed, 10),
        ];
        let page = page(&scanned(tree(&nodes, &[("/home/app", 400)])));

        assert!(page.rows[0].project);
    }

    #[test]
    fn the_days_follow_the_stale_projects_setting() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Recent", NodeKind::Folder, 3_000)];
        let state = || tree(&nodes, &[("/home/Recent", 100)]);
        let ninety = MemorySettings::with(&[("stale-projects.unused-for-days", json!(90))]);

        let by_default = page(&scanned(state()));
        let by_setting = page(&engine_of(state(), ninety, None));

        assert_eq!((by_default.total, by_default.days), (0, 365));
        assert_eq!((by_setting.total, by_setting.days), (1, 90));
    }

    #[test]
    fn the_size_follows_its_setting() {
        let nodes = [("/home", NodeKind::Folder, 9_000), ("/home/Mid", NodeKind::Folder, 600)];
        let state = tree(&nodes, &[("/home/Mid", 900)]);

        let page = page(&engine_of(state, caches_setting("500-mb"), None));

        assert_eq!((page.total, page.min_size.bytes()), (1, 500 * MB));
    }

    #[test]
    fn a_stale_folder_inside_a_folder_named_library_below_the_top_is_listed() {
        let nodes = [
            ("/home", NodeKind::Folder, 9_000),
            ("/home/Public", NodeKind::Folder, 5_000),
            ("/home/Public/Library", NodeKind::Folder, 2_000),
        ];
        let state = tree(&nodes, &[("/home/Public", 400), ("/home/Public/Library", 400)]);
        // A standard folder is entered, so its children are asked as a root of their own.
        let engine = engine_of(state, MemorySettings::default(), Some(protecting(&[], &["/home/Public"])));

        assert_eq!(listed(&page(&engine)), ["~/Public/Library"]);
    }

    #[test]
    fn rows_come_largest_first_and_page_with_the_totals() {
        let nodes = [
            ("/home", NodeKind::Folder, 20_000),
            ("/home/b", NodeKind::Folder, 2_000),
            ("/home/a", NodeKind::Folder, 2_000),
            ("/home/c", NodeKind::Folder, 3_000),
        ];
        let engine = scanned(tree(&nodes, &[("/home/a", 400), ("/home/b", 400), ("/home/c", 400)]));

        let first = engine.largest_folders(FoldersQuery { offset: 0, limit: 2 }).unwrap();
        let rest = engine.largest_folders(FoldersQuery { offset: 2, limit: 2 }).unwrap();
        let past = engine.largest_folders(FoldersQuery { offset: 5, limit: 2 }).unwrap();

        assert_eq!(listed(&first), ["~/c", "~/a"]);
        assert_eq!(listed(&rest), ["~/b"]);
        assert_eq!(listed(&past), Vec::<String>::new());
        for page in [&first, &rest, &past] {
            assert_eq!((page.total, page.sizes.allocated.bytes()), (3, 7_000 * MB));
        }
        assert_eq!(first.offset, 0);
        assert_eq!(rest.offset, 2);
    }
}

mod verdict {
    use fresher_core::{Offer, Withheld};

    use super::super::{Verdict, verdict};
    use crate::largest::judge::Ignored;

    mod negative {
        use super::{Ignored, Offer, Verdict, Withheld, verdict};

        #[test]
        fn an_ignored_folder_is_dropped_whatever_the_judgement() {
            for judged in [
                (Offer::Offered, true),
                (Offer::Withheld(Withheld::Ignored), false),
                (Offer::Withheld(Withheld::StandardFolder), true),
            ] {
                assert_eq!(verdict(Ignored::AtOrAbove, judged), Verdict::Drop, "{judged:?}");
            }
        }

        #[test]
        fn a_folder_holding_an_ignored_path_is_entered_not_listed() {
            for judged in [(Offer::Offered, true), (Offer::Withheld(Withheld::Ignored), true)] {
                assert_eq!(verdict(Ignored::Inside, judged), Verdict::Enter, "{judged:?}");
            }
        }

        #[test]
        fn ignored_wins_over_holding_explained_items() {
            let holds = (Offer::Withheld(Withheld::HoldsExplained), true);

            assert_eq!(verdict(Ignored::AtOrAbove, holds), Verdict::Drop);
            assert_eq!(verdict(Ignored::Inside, holds), Verdict::Enter);
        }

        #[test]
        fn a_withheld_folder_that_opens_is_entered() {
            for reason in [Withheld::StandardFolder, Withheld::ReadInPart] {
                assert_eq!(verdict(Ignored::No, (Offer::Withheld(reason), true)), Verdict::Enter, "{reason:?}");
            }
        }

        #[test]
        fn a_withheld_folder_that_does_not_open_is_dropped() {
            for reason in [Withheld::Explained, Withheld::Library, Withheld::Hidden, Withheld::Protected] {
                assert_eq!(verdict(Ignored::No, (Offer::Withheld(reason), false)), Verdict::Drop, "{reason:?}");
            }
        }
    }

    mod positive {
        use super::{Ignored, Offer, Verdict, Withheld, verdict};

        #[test]
        fn an_offered_folder_is_listed() {
            assert_eq!(verdict(Ignored::No, (Offer::Offered, true)), Verdict::List);
            assert_eq!(verdict(Ignored::No, (Offer::Offered, false)), Verdict::List);
        }

        #[test]
        fn a_folder_holding_explained_items_is_listed() {
            let holds = (Offer::Withheld(Withheld::HoldsExplained), true);

            assert_eq!(verdict(Ignored::No, holds), Verdict::List);
        }
    }
}
