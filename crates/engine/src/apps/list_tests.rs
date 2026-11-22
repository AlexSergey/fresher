use std::sync::Arc;

use fresher_core::{AppOffer, AppSummary, NodeKind};
use fresher_testing::{FakeInstalledApps, FakeRunningApps};

use super::super::fixture::{fixture, state};
use crate::Engine;
use crate::testing::scan;

const DAY: u64 = 24 * 60 * 60;

/// An engine after one scan of an empty home, with `apps` installed and `running` running.
fn running(apps: FakeInstalledApps, running: FakeRunningApps) -> Engine {
    let engine = fixture(vec![state(1, &[("/home", NodeKind::Folder, 0)])], Arc::new(apps), Arc::new(running)).engine();
    scan(&engine);
    engine
}

fn offer(rows: &[AppSummary], name: &str) -> AppOffer {
    rows.iter().find(|row| row.name == name).map(|row| row.offer.clone()).unwrap()
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{
        AppId, AppOffer, AppsQuery, AppsView, BundleFacts, BundleId, CommandError, Completeness, EntryFacts,
        InstalledBy, Manager, NodeKind, NotOffered, Protection, StateId, UnusedFor,
    };
    use fresher_testing::{FakeInstalledApps, FakeRunningApps, running_app};

    use super::super::super::fixture::{app, at, facts, fixture, names, rows, scanned, state};
    use super::{DAY, offer, running};
    use crate::testing::{Protecting, START, ScriptedWalk, engine, scan};

    #[test]
    fn the_list_before_any_scan_is_no_scan() {
        let engine = engine(ScriptedWalk::new());

        assert_eq!(engine.apps(AppsQuery::default()), Err(CommandError::NoScan));
    }

    #[test]
    fn nothing_is_listed_when_the_apps_cannot_be_listed() {
        let engine = scanned(state(1, &[]), Arc::new(FakeInstalledApps::failing("no apps folder")));

        assert_eq!(engine.apps(AppsQuery::default()), Ok(AppsView::unlisted()));
    }

    #[test]
    fn apple_s_apps_are_not_listed() {
        let safari = app("com.apple.Safari", "Safari");
        let chess = at(app("com.example.Chess", "Chess"), "/System/Applications/Chess.app");
        let apps = FakeInstalledApps::new(vec![safari.clone(), chess.clone()])
            .with_bundle(&safari.path, Ok(facts(1, None)))
            .with_bundle(&chess.path, Ok(facts(1, None)));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(names(&engine), Vec::<String>::new());
    }

    #[test]
    fn an_app_outside_the_apps_folders_is_not_listed() {
        let outside = [
            "/home/Downloads/Editor.app",
            "/Applications/Tools/More/Deep.app",
            "/Applications/Outer.app/Contents/Inner.app",
            "/Applications/Plain",
            "/Applications/Sub.app/Inner.app",
            "/Applications/../Applications/Editor.app",
        ];
        let installed: Vec<_> = outside.iter().map(|&path| at(app("com.example.Editor", "Editor"), path)).collect();
        let apps = installed.iter().fold(FakeInstalledApps::new(installed.clone()), |apps, app| {
            apps.with_bundle(&app.path, Ok(facts(1, None)))
        });

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(names(&engine), Vec::<String>::new());
    }

    #[test]
    fn fresher_s_own_bundle_is_not_listed() {
        let own = app("com.example.Fresher", "Fresher");
        let apps = FakeInstalledApps::new(vec![own.clone()]).with_bundle(&own.path, Ok(facts(1, None)));
        let protection = Protection::new(vec![own.path.clone()], Vec::new(), Vec::new()).unwrap();
        let engine = fixture(vec![state(1, &[])], Arc::new(apps), Arc::new(FakeRunningApps::default()))
            .cleaner(Arc::new(Protecting::new(protection)))
            .engine();
        scan(&engine);

        assert_eq!(names(&engine), Vec::<String>::new());
        let id = AppId::new(StateId::new(1), 0);
        assert_eq!(engine.app(id), Err(CommandError::UnknownApp { id }));
    }

    #[test]
    fn fresher_s_own_id_is_never_listed_whatever_the_protection() {
        let ids = ["io.github.alexsergey.fresher", "io.github.alexsergey.fresher.dev", "IO.GITHUB.ALEXSERGEY.FRESHER"];
        let mut installed: Vec<_> =
            ids.iter().map(|&id| at(app(id, "Fresher"), &format!("/Applications/{id}.app"))).collect();
        let mut helper = app("com.example.Wrapper", "Wrapper");
        helper.helpers = vec![BundleId::new("io.github.alexsergey.fresher.helper").unwrap()];
        installed.push(helper);
        let apps = installed.iter().fold(FakeInstalledApps::new(installed.clone()), |apps, app| {
            apps.with_bundle(&app.path, Ok(facts(1, None)))
        });

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(names(&engine), Vec::<String>::new());
    }

    #[test]
    fn a_partial_installed_list_keeps_every_part() {
        let editor = app("com.example.Editor", "Editor");
        let apps = FakeInstalledApps::partial(vec![editor.clone()], &["/Applications/Broken.app"])
            .with_bundle(&editor.path, Ok(facts(1, None)));
        let state = state(1, &[("/home/Library/Caches/com.example.Editor", NodeKind::Folder, 4)]);

        let engine = scanned(state, Arc::new(apps));

        let page = engine.app(AppId::new(StateId::new(1), 0)).unwrap();
        assert_eq!(page.items[1].kept.as_ref().map(|kept| kept.reason), Some(fresher_core::KeptReason::Shared));
        assert_eq!(page.choices[0].items, 1, "only the bundle goes");
    }

    #[test]
    fn an_unmeasured_bundle_is_not_offered() {
        let editor = app("com.example.Editor", "Editor");
        let state = state(1, &[("/home/Library/Caches/com.example.Editor", NodeKind::Folder, 7)]);

        let engine = scanned(state, Arc::new(FakeInstalledApps::new(vec![editor])));

        let rows = rows(&engine);
        assert_eq!(rows[0].offer, AppOffer::not_offered(NotOffered::Unmeasured));
        assert_eq!(rows[0].sizes.completeness, Completeness::Unknown);
        assert_eq!(rows[0].sizes.allocated.bytes(), 7, "the data it left still counts");
    }

    #[test]
    fn a_homebrew_app_is_not_offered() {
        let editor = app("com.example.Editor", "Editor");
        let installed_by = Some(InstalledBy::new(Manager::Homebrew, "editor").unwrap());
        let bundle = BundleFacts { installed_by, ..facts(1, None) };
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(bundle));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(rows(&engine)[0].offer, AppOffer::installed_by(Manager::Homebrew));
    }

    #[test]
    fn a_root_owned_bundle_needs_an_administrator() {
        let editor = app("com.example.Editor", "Editor");
        let bundle = BundleFacts { entry: EntryFacts::default(), ..facts(1, None) };
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(bundle));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(rows(&engine)[0].offer, AppOffer::not_offered(NotOffered::NeedsAdministrator));
    }

    #[test]
    fn a_locked_bundle_is_not_offered() {
        let (locked, guarded) = (app("com.example.Locked", "Locked"), app("com.example.Guarded", "Guarded"));
        let lock = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        let guard = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
        let apps = FakeInstalledApps::new(vec![locked.clone(), guarded.clone()])
            .with_bundle(&locked.path, Ok(BundleFacts { entry: lock, ..facts(1, None) }))
            .with_bundle(&guarded.path, Ok(BundleFacts { entry: guard, ..facts(1, None) }));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        let rows = rows(&engine);
        assert_eq!(offer(&rows, "Locked"), AppOffer::not_offered(NotOffered::Locked));
        assert_eq!(offer(&rows, "Guarded"), AppOffer::not_offered(NotOffered::Protected));
    }

    #[test]
    fn an_open_app_is_not_offered() {
        let editor = app("com.example.Editor", "Editor");
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(facts(1, None)));
        let open = FakeRunningApps::new(vec![running_app("Editor", &["com.example.editor"]).unwrap()]).then(Vec::new());

        let engine = running(apps, open);

        assert_eq!(rows(&engine)[0].offer, AppOffer::not_offered(NotOffered::Open));
        assert_eq!(rows(&engine)[0].offer, AppOffer::Offered, "asked again at the next listing");
    }

    #[test]
    fn an_open_helper_keeps_its_app_from_being_offered() {
        let mut editor = app("com.example.Editor", "Editor");
        editor.helpers = vec![BundleId::new("com.example.Editor.Agent").unwrap()];
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(facts(1, None)));
        let open = FakeRunningApps::new(vec![running_app("Agent", &["com.example.Editor.Agent"]).unwrap()]);

        let engine = running(apps, open);

        assert_eq!(rows(&engine)[0].offer, AppOffer::not_offered(NotOffered::Open));
    }

    #[test]
    fn an_app_is_not_offered_when_the_running_apps_are_not_known() {
        let editor = app("com.example.Editor", "Editor");
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(facts(1, None)));

        let engine = running(apps, FakeRunningApps::failing("no process list"));

        assert_eq!(rows(&engine)[0].offer, AppOffer::not_offered(NotOffered::Open));
    }

    #[test]
    fn a_look_alike_running_leaves_the_app_offered() {
        let editor = app("com.example.Editor", "Editor");
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(facts(1, None)));
        let open = FakeRunningApps::new(vec![running_app("Editorx", &["com.example.Editorx"]).unwrap()]);

        let engine = running(apps, open);

        assert_eq!(rows(&engine)[0].offer, AppOffer::Offered);
    }

    #[test]
    fn an_app_with_no_date_is_not_in_a_filtered_list() {
        let (dated, undated) = (app("com.example.Old", "Old"), app("com.example.Unknown", "Unknown"));
        let apps = FakeInstalledApps::new(vec![dated.clone(), undated.clone()])
            .with_bundle(&dated.path, Ok(facts(1, Some(START - 400 * DAY))))
            .with_bundle(&undated.path, Ok(facts(1, None)));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        let filtered = engine.apps(AppsQuery::new(Some(UnusedFor::AYear))).unwrap();
        assert_eq!(filtered.apps.iter().map(|row| row.name.as_str()).collect::<Vec<_>>(), ["Old"]);
    }
}

mod positive {
    use std::path::Path;
    use std::sync::Arc;

    use fresher_core::{
        AppId, AppOffer, AppsQuery, AppsView, ByteSize, FootprintKind, NodeKind, PartSizes, Sizes, StateId, TextArg,
        Timestamp, UnusedFor,
    };
    use fresher_testing::FakeInstalledApps;

    use super::super::super::fixture::{app, at, facts, names, rows, scanned, state};
    use super::DAY;
    use crate::testing::{START, bytes};

    #[test]
    fn an_app_is_offered_with_its_size_and_parts() {
        let editor = app("com.example.Editor", "Editor");
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(&editor.path, Ok(facts(100, None)));
        let state = state(
            1,
            &[
                ("/home/Library/Caches/com.example.Editor", NodeKind::Folder, 10),
                ("/home/Library/Preferences/com.example.Editor.plist", NodeKind::File, 1),
                ("/home/Library/Logs/com.example.Editor", NodeKind::Link, 2),
            ],
        );

        let engine = scanned(state, Arc::new(apps));

        let view = engine.apps(AppsQuery::default()).unwrap();
        assert_eq!(view.notice, AppsView::listed(Vec::new()).notice);
        let row = &view.apps[0];
        assert_eq!(row.offer, AppOffer::Offered);
        assert_eq!((row.name.as_str(), row.version.as_deref()), ("Editor", Some("1.0")));
        assert_eq!(row.sizes, bytes(113), "the bundle and every part, the kept link too");
        assert_eq!(
            row.parts,
            [
                PartSizes { kind: FootprintKind::Bundle, sizes: bytes(100) },
                PartSizes { kind: FootprintKind::Caches, sizes: bytes(10) },
                PartSizes { kind: FootprintKind::Logs, sizes: bytes(2) },
                PartSizes { kind: FootprintKind::Settings, sizes: bytes(1) },
            ]
        );
        assert_eq!(row.app, AppId::new(StateId::new(1), 0));
    }

    #[test]
    fn an_app_in_a_folder_of_an_apps_folder_is_listed() {
        let tool = at(app("com.example.Tool", "Tool"), "/Applications/Utilities/Tool.app");
        let mine = at(app("com.example.Mine", "Mine"), "/home/Applications/Mine.app");
        let apps = FakeInstalledApps::new(vec![tool.clone(), mine.clone()])
            .with_bundle(&tool.path, Ok(facts(2, None)))
            .with_bundle(&mine.path, Ok(facts(1, None)));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(names(&engine), ["Tool", "Mine"]);
    }

    #[test]
    fn the_list_is_largest_first() {
        let installed =
            [("com.example.Small", "Small", 1), ("com.example.Big", "Big", 9), ("com.example.Also", "Also", 1)];
        let apps = installed.iter().fold(
            FakeInstalledApps::new(installed.iter().map(|&(id, name, _)| app(id, name)).collect()),
            |apps, &(id, name, size)| apps.with_bundle(app(id, name).path, Ok(facts(size, None))),
        );

        let engine = scanned(state(1, &[]), Arc::new(apps));

        assert_eq!(names(&engine), ["Big", "Also", "Small"]);
    }

    #[test]
    fn the_last_use_is_the_later_of_spotlight_and_the_data() {
        let (spotlit, busy) = (app("com.example.Spotlit", "Spotlit"), app("com.example.Busy", "Busy"));
        let apps = FakeInstalledApps::new(vec![spotlit.clone(), busy.clone()])
            .with_bundle(&spotlit.path, Ok(facts(1, Some(500))))
            .with_bundle(&busy.path, Ok(facts(1, Some(100))));
        let modified = |state: fresher_testing::FixedDiskState, path: &str, secs| {
            state.with_modified(Path::new(path), Timestamp::from_secs(secs))
        };
        let state = state(
            1,
            &[
                ("/home/Library/Caches/com.example.Spotlit", NodeKind::Folder, 1),
                ("/home/Library/Caches/com.example.Busy", NodeKind::Folder, 1),
                ("/home/Library/Logs/com.example.Busy", NodeKind::Folder, 1),
            ],
        );
        let state = modified(state, "/home/Library/Caches/com.example.Spotlit", 200);
        let state = modified(state, "/home/Library/Caches/com.example.Busy", 300);
        let state = modified(state, "/home/Library/Logs/com.example.Busy", 900);

        let engine = scanned(state, Arc::new(apps));

        let used: Vec<_> = rows(&engine).into_iter().map(|row| (row.name, row.last_used)).collect();
        assert_eq!(
            used,
            [
                ("Busy".to_owned(), Some(Timestamp::from_secs(300))),
                ("Spotlit".to_owned(), Some(Timestamp::from_secs(500)))
            ],
            "a log is not a use"
        );
    }

    #[test]
    fn the_unused_sentence_names_the_date_and_the_size() {
        let (old, recent, unknown) =
            (app("com.example.Old", "Old"), app("com.example.Recent", "Recent"), app("com.example.Never", "Never"));
        let apps = FakeInstalledApps::new(vec![old.clone(), recent.clone(), unknown.clone()])
            .with_bundle(&old.path, Ok(facts(4096, Some(START - 31 * DAY))))
            .with_bundle(&recent.path, Ok(facts(2048, Some(START - 30 * DAY))))
            .with_bundle(&unknown.path, Ok(facts(1024, None)));

        let engine = scanned(state(1, &[]), Arc::new(apps));

        let rows = rows(&engine);
        let old = &rows[0].summary;
        assert_eq!(old.key().as_str(), "core.apps.summary.unused");
        assert_eq!(old.args().get("date"), Some(&TextArg::Date(Timestamp::from_secs(START - 31 * DAY))));
        assert_eq!(old.args().get("size"), Some(&TextArg::Size(ByteSize::new(4096))));
        assert_eq!(old.args().get("app"), Some(&TextArg::Text("Old".into())));
        assert_eq!(rows[1].summary.key().as_str(), "core.apps.summary.used");
        assert_eq!(rows[2].summary.key().as_str(), "core.apps.summary.unknown");
        assert_eq!(rows[2].sizes, Sizes::complete(ByteSize::new(1024), ByteSize::new(1024)));
    }

    #[test]
    fn the_filter_keeps_apps_unused_for_its_days() {
        let ages = [
            ("com.example.A", "A", 89),
            ("com.example.B", "B", 90),
            ("com.example.C", "C", 182),
            ("com.example.D", "D", 365),
        ];
        let installed: Vec<_> = ages.iter().map(|&(id, name, _)| app(id, name)).collect();
        let apps = ages.iter().fold(FakeInstalledApps::new(installed), |apps, &(id, name, days)| {
            apps.with_bundle(app(id, name).path, Ok(facts(1, Some(START - days * DAY))))
        });

        let engine = scanned(state(1, &[]), Arc::new(apps));

        let kept = |unused| -> Vec<String> {
            engine.apps(AppsQuery::new(Some(unused))).unwrap().apps.into_iter().map(|row| row.name).collect()
        };
        assert_eq!(kept(UnusedFor::ThreeMonths), ["B", "C", "D"]);
        assert_eq!(kept(UnusedFor::SixMonths), ["C", "D"]);
        assert_eq!(kept(UnusedFor::AYear), ["D"]);
        assert_eq!(names(&engine), ["A", "B", "C", "D"]);
    }
}
