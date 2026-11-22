use std::sync::Arc;

use fresher_core::{AppId, FindingId, InstalledApp, ItemOutcome, NodeKind, StateId, UninstallChoice};
use fresher_testing::{FakeCleaner, FakeInstalledApps, FakeSystemUi};

use super::super::fixture::{app, facts, fixture, idle, state};
use crate::Engine;
use crate::cleanup::testing::{echo, wait_for_cleanup};
use crate::testing::scan;

/// The editor and its data: a cache, a log, saved windows, a launch agent, its settings, its files and a link.
fn editor_home() -> fresher_testing::FixedDiskState {
    state(
        1,
        &[
            ("/home/Library/Caches/com.example.Editor", NodeKind::Folder, 10),
            ("/home/Library/Logs/com.example.Editor", NodeKind::Folder, 1),
            ("/home/Library/Saved Application State/com.example.Editor.savedState", NodeKind::Folder, 2),
            ("/home/Library/LaunchAgents/com.example.Editor.updater.plist", NodeKind::File, 1),
            ("/home/Library/LaunchAgents/com.example.Editor.plist", NodeKind::File, 1),
            ("/home/Library/Preferences/com.example.Editor.plist", NodeKind::File, 3),
            ("/home/Library/Application Support/com.example.Editor", NodeKind::Folder, 20),
            ("/home/Library/WebKit/com.example.Editor", NodeKind::Link, 5),
        ],
    )
}

fn editor() -> InstalledApp {
    app("com.example.Editor", "Editor")
}

/// An engine after one scan of `editor_home`, the editor's bundle 100 bytes, showing in `system_ui`.
fn scanned(system_ui: FakeSystemUi) -> (Engine, Arc<FakeSystemUi>) {
    let apps = FakeInstalledApps::new(vec![editor()]).with_bundle(editor().path, Ok(facts(100, None)));
    let fixture = fixture(vec![editor_home()], Arc::new(apps), idle()).system_ui(system_ui);
    let system_ui = Arc::clone(&fixture.system_ui);
    let engine = fixture.engine();
    scan(&engine);
    (engine, system_ui)
}

const EDITOR: AppId = AppId::new(StateId::new(1), 0);
const VIEWER: AppId = AppId::new(StateId::new(1), 1);

/// An engine after one scan of `editor_home` with the editor and a viewer installed, once an uninstall of the editor
/// ended `outcome`.
fn uninstalled(outcome: ItemOutcome) -> Engine {
    let viewer = app("com.example.Viewer", "Viewer");
    let apps = FakeInstalledApps::new(vec![editor(), viewer.clone()])
        .with_bundle(editor().path, Ok(facts(100, None)))
        .with_bundle(viewer.path, Ok(facts(100, None)));
    let cleaner = FakeCleaner::default().ending(FindingId::new(1), outcome);
    let engine = fixture(vec![editor_home()], Arc::new(apps), idle()).cleaner(Arc::new(cleaner)).engine();
    scan(&engine);
    let view = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
    assert_eq!(view.groups[0].items[0].finding, FindingId::new(1));
    let events = engine.subscribe();
    engine.confirm_cleanup(&echo(&view, "trash")).unwrap();
    wait_for_cleanup(&events);
    engine
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{
        Access, AppId, AppOffer, BundleFacts, Capability, CommandError, InstalledBy, ItemOutcome, Manager, StateId,
        SystemUiError,
    };
    use fresher_testing::{FakeInstalledApps, FakeSystemUi, SystemUiCall};

    use super::super::super::fixture::{facts, fixture, idle, state};
    use super::{EDITOR, VIEWER, editor, scanned, uninstalled};
    use crate::testing::{ScriptedWalk, engine, scan};

    #[test]
    fn the_page_before_any_scan_is_no_scan() {
        let engine = engine(ScriptedWalk::new());

        assert_eq!(engine.app(EDITOR), Err(CommandError::NoScan));
        assert_eq!(engine.reveal_app(EDITOR), Err(CommandError::NoScan));
    }

    #[test]
    fn an_id_of_another_state_is_unknown() {
        let (engine, system_ui) = scanned(FakeSystemUi::default());
        let id = AppId::new(StateId::new(9), 0);

        assert_eq!(engine.app(id), Err(CommandError::UnknownApp { id }));
        assert_eq!(engine.reveal_app(id), Err(CommandError::UnknownApp { id }));
        assert_eq!(system_ui.calls(), []);
    }

    #[test]
    fn an_index_past_the_list_is_unknown() {
        let (engine, system_ui) = scanned(FakeSystemUi::default());
        let id = AppId::new(StateId::new(1), 1);

        assert_eq!(engine.app(id), Err(CommandError::UnknownApp { id }));
        assert_eq!(engine.reveal_app(id), Err(CommandError::UnknownApp { id }));
        assert_eq!(system_ui.calls(), []);
    }

    #[test]
    fn an_id_of_an_earlier_state_is_unknown_after_a_rescan() {
        let (engine, _) = scanned(FakeSystemUi::default());
        scan(&engine);

        assert_eq!(engine.app(EDITOR), Err(CommandError::UnknownApp { id: EDITOR }));
    }

    #[test]
    fn reveal_of_an_earlier_state_s_id_is_refused() {
        let (engine, system_ui) = scanned(FakeSystemUi::default());
        scan(&engine);

        assert_eq!(engine.reveal_app(EDITOR), Err(CommandError::UnknownApp { id: EDITOR }));
        assert_eq!(system_ui.calls(), []);
    }

    #[test]
    fn a_failed_reveal_is_reported() {
        let failure = SystemUiError::Failed { reason: "open exited with 1".into() };
        let (engine, system_ui) = scanned(FakeSystemUi::failing(failure));

        assert_eq!(engine.reveal_app(EDITOR), Err(CommandError::Internal));
        assert_eq!(system_ui.calls(), [SystemUiCall::Reveal("/Applications/Editor.app".into())]);
    }

    #[test]
    fn a_page_without_a_manager_gives_no_command() {
        let (engine, _) = scanned(FakeSystemUi::default());

        let page = engine.app(EDITOR).unwrap();

        assert_eq!(page.command, None);
    }

    #[test]
    fn no_notice_for_an_app_that_is_not_offered() {
        let installed_by = Some(InstalledBy::new(Manager::Homebrew, "editor").unwrap());
        let bundle = BundleFacts { installed_by, ..facts(1, None) };
        let apps = FakeInstalledApps::new(vec![editor()]).with_bundle(editor().path, Ok(bundle));
        let engine = fixture(vec![state(1, &[])], Arc::new(apps), idle()).engine();
        scan(&engine);

        let page = engine.app(EDITOR).unwrap();

        assert_eq!(page.summary.offer, AppOffer::installed_by(Manager::Homebrew));
        assert_eq!(page.notice, None);
    }

    #[test]
    fn no_notice_once_granted() {
        let engine = uninstalled(ItemOutcome::Moved);

        let page = engine.app(VIEWER).unwrap();

        assert_eq!(engine.access_status().access(&Capability::AppManagement), Access::Granted);
        assert_eq!(page.notice, None);
    }
}

mod positive {
    use std::path::PathBuf;
    use std::sync::Arc;

    use fresher_core::{
        AppOffer, BundleFacts, BundleId, ByteSize, DangerLevel, FootprintKind, InstalledBy, KeptReason, Manager,
        UninstallChoice,
    };
    use fresher_testing::{FakeInstalledApps, FakeSystemUi, SystemUiCall};

    use super::super::super::fixture::{facts, fixture, idle, state};
    use super::{EDITOR, VIEWER, editor, editor_home, scanned, uninstalled};
    use crate::testing::{TestModule, bytes, category, finding, scan};

    #[test]
    fn the_page_lists_every_part_in_order() {
        let (engine, _) = scanned(FakeSystemUi::default());

        let page = engine.app(EDITOR).unwrap();

        let items: Vec<_> = page
            .items
            .iter()
            .map(|item| (item.kind, item.path.clone(), item.kept.as_ref().map(|kept| kept.reason)))
            .collect();
        let library = |path: &str| PathBuf::from("/home/Library").join(path);
        assert_eq!(
            items,
            [
                (FootprintKind::Bundle, PathBuf::from("/Applications/Editor.app"), None),
                (FootprintKind::Caches, library("Caches/com.example.Editor"), None),
                (FootprintKind::Logs, library("Logs/com.example.Editor"), None),
                (FootprintKind::SavedState, library("Saved Application State/com.example.Editor.savedState"), None),
                (FootprintKind::LaunchAgents, library("LaunchAgents/com.example.Editor.plist"), None),
                (FootprintKind::LaunchAgents, library("LaunchAgents/com.example.Editor.updater.plist"), None),
                (FootprintKind::WebData, library("WebKit/com.example.Editor"), Some(KeptReason::Link)),
                (FootprintKind::Settings, library("Preferences/com.example.Editor.plist"), None),
                (FootprintKind::Support, library("Application Support/com.example.Editor"), None),
            ]
        );
        assert_eq!(page.items[0].sizes, bytes(100));
        assert_eq!(page.items[0].danger, DangerLevel::Medium);
        assert_eq!(page.summary.offer, AppOffer::Offered);
        assert_eq!(page.summary.sizes, bytes(143));
    }

    #[test]
    fn each_choice_counts_only_what_goes() {
        let (engine, _) = scanned(FakeSystemUi::default());

        let page = engine.app(EDITOR).unwrap();

        let choices: Vec<_> = page.choices.iter().map(|view| (view.choice, view.sizes, view.items)).collect();
        assert_eq!(choices, [(UninstallChoice::KeepData, bytes(115), 6), (UninstallChoice::Everything, bytes(138), 8)]);
        assert_eq!(page.choices[0].label.key().as_str(), "core.uninstall.choice.keep-data.label");
    }

    #[test]
    fn an_unmeasured_app_s_choices_are_not_known_in_full() {
        let apps = FakeInstalledApps::new(vec![editor()]);
        let engine = fixture(vec![editor_home()], Arc::new(apps), idle()).engine();
        scan(&engine);

        let page = engine.app(EDITOR).unwrap();

        assert_eq!(page.items[0].sizes.completeness, fresher_core::Completeness::Unknown);
        assert!(page.choices.iter().all(|view| view.sizes.completeness == fresher_core::Completeness::Unknown));
    }

    #[test]
    fn the_page_links_the_app_s_categories() {
        let caches = category("alpha.caches", DangerLevel::Safe);
        let logs = category("alpha.logs", DangerLevel::Safe);
        let other = category("alpha.other", DangerLevel::Safe);
        let module = TestModule::builder("alpha")
            .category("alpha.other", DangerLevel::Safe)
            .category("alpha.logs", DangerLevel::Safe)
            .category("alpha.caches", DangerLevel::Safe)
            .on_finish(move |_, out| {
                let id = |id: &str| BundleId::new(id).unwrap();
                out.push(
                    finding(&caches, "/home/Library/Caches/com.example.Editor", bytes(1))
                        .for_app(id("com.example.Editor")),
                );
                out.push(
                    finding(&logs, "/home/Library/Logs/com.example.Editor.Helper", bytes(1))
                        .for_app(id("com.example.editor.helper")),
                );
                out.push(
                    finding(&other, "/home/Library/Caches/com.example.Other", bytes(1))
                        .for_app(id("com.example.Other")),
                );
                Ok(())
            });
        let mut owner = editor();
        owner.helpers = vec![BundleId::new("com.example.Editor.Helper").unwrap()];
        let apps = FakeInstalledApps::new(vec![owner.clone()]).with_bundle(owner.path, Ok(facts(1, None)));
        let engine = fixture(vec![state(1, &[])], Arc::new(apps), idle()).module(module).engine();
        scan(&engine);

        let page = engine.app(EDITOR).unwrap();

        let ids: Vec<_> = page.categories.iter().map(|id| id.as_str().to_owned()).collect();
        assert_eq!(ids, ["alpha.logs", "alpha.caches"], "in the registry's order");
    }

    #[test]
    fn a_homebrew_app_s_page_gives_the_command() {
        let installed_by = Some(InstalledBy::new(Manager::Homebrew, "editor").unwrap());
        let bundle = BundleFacts { installed_by, ..facts(1, None) };
        let apps = FakeInstalledApps::new(vec![editor()]).with_bundle(editor().path, Ok(bundle));
        let engine = fixture(vec![state(1, &[])], Arc::new(apps), idle()).engine();
        scan(&engine);

        let page = engine.app(EDITOR).unwrap();

        assert_eq!(page.command.as_deref(), Some("brew uninstall --cask editor"));
        assert_eq!(page.summary.offer, AppOffer::installed_by(Manager::Homebrew));
    }

    #[test]
    fn the_page_says_macos_may_ask_before_the_first_uninstall() {
        let (engine, _) = scanned(FakeSystemUi::default());

        let page = engine.app(EDITOR).unwrap();

        assert_eq!(page.notice.unwrap().key().as_str(), "core.apps.management.not-asked");
    }

    #[test]
    fn the_page_says_what_to_do_once_denied() {
        let engine =
            uninstalled(fresher_core::ItemOutcome::Failed { reason: fresher_core::Failure::TrashRefused.text() });

        let page = engine.app(EDITOR).unwrap();

        assert_eq!(page.notice.unwrap().key().as_str(), "core.access.app-management.denied");
        assert_eq!(engine.app(VIEWER).unwrap().notice.unwrap().key().as_str(), "core.access.app-management.denied");
    }

    #[test]
    fn reveal_shows_the_bundle_in_finder() {
        let (engine, system_ui) = scanned(FakeSystemUi::default());

        engine.reveal_app(EDITOR).unwrap();

        assert_eq!(system_ui.calls(), [SystemUiCall::Reveal(PathBuf::from("/Applications/Editor.app"))]);
    }

    #[test]
    fn a_page_s_summary_is_its_row_s() {
        let (engine, _) = scanned(FakeSystemUi::default());

        let page = engine.app(EDITOR).unwrap();

        let row = engine.apps(fresher_core::AppsQuery::default()).unwrap().apps.remove(0);
        assert_eq!(page.summary, row);
        assert_eq!(row.summary.args().get("size"), Some(&fresher_core::TextArg::Size(ByteSize::new(143))));
    }
}
