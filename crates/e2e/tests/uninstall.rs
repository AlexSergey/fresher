//! Uninstall an app end to end: a fixture sandbox's apps folder with an app, its helper and its data in every place of
//! its footprint, an app with no data and a look-alike's cache; the real scanner, the system's listing of the
//! sandbox's apps folders with the stated bundle reader, the real settings file, scan cache and journal, and the real
//! `DiskCleaner` reaching only inside the sandbox, with a fake Trash. Both choices move exactly their paths and put
//! back brings the whole app back; every way the disk or the running apps change between the plan and the act moves
//! nothing, and no ordinary cleanup ever moves a bundle. The keep-data flow's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test uninstall` rewrites `fixtures/golden/uninstall-flow.json`;
//! the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "uninstall_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    use fresher_core::{
        Access, CleanupPlanView, CommandError, Confirmation, ItemOutcome, LeftOut, NotOffered, PutBackOutcome,
        SkipReason, UninstallChoice,
    };
    use fresher_testing::TreeReport;
    use serde_json::{Value, json};

    use crate::flow::{self, BundleModule, EDITOR_ID, EXECUTABLE, HELPER_ID, Home, Setup};
    use crate::tree::contents;

    const BUNDLE: &str = "Applications/Editor.app";
    const CACHES: &str = "home/Library/Caches/com.example.Editor";
    const HELPER_CACHES: &str = "home/Library/Caches/com.example.Editor.Helper";
    const LOGS: &str = "home/Library/Logs/com.example.Editor";
    const SAVED_STATE: &str = "home/Library/Saved Application State/com.example.Editor.savedState";
    const AGENT: &str = "home/Library/LaunchAgents/com.example.Editor.plist";
    const WEB_DATA: &str = "home/Library/WebKit/com.example.Editor";
    const SETTINGS: &str = "home/Library/Preferences/com.example.Editor.plist";
    const SUPPORT: &str = "home/Library/Application Support/com.example.Editor";
    const GROUP: &str = "home/Library/Group Containers/TEAM123456.com.example.shared";
    /// What keep-data moves, in the plan's order: the bundle first, then the parts in the footprint table's order.
    const KEEP_DATA: [&str; 6] = [BUNDLE, CACHES, HELPER_CACHES, LOGS, SAVED_STATE, AGENT];
    /// What everything moves besides keep-data's paths, in the table's order.
    const DATA: [&str; 4] = [WEB_DATA, SETTINGS, SUPPORT, GROUP];
    /// The cache of `com.example.Editorx`, whose id only starts like Editor's.
    const LOOK_ALIKE: &str = "home/Library/Caches/com.example.Editorx";
    const VIEWER: &str = "Applications/Viewer.app";
    const SECOND_COPY: &str = "home/Applications/Editor.app";
    /// `app-caches`' category of an installed app's caches.
    const APP_CACHES: &str = "app-caches.library-caches";

    /// Every path of Editor, in the plan's order of `everything`.
    fn every_part() -> Vec<&'static str> {
        KEEP_DATA.iter().chain(&DATA).copied().collect()
    }

    /// Every file at or under `paths`, with its bytes.
    fn files(home: &Home, paths: &[&str]) -> BTreeMap<PathBuf, Vec<u8>> {
        paths.iter().flat_map(|path| contents(&home.path(path))).collect()
    }

    /// Every file of the apps folder and of the home's Library: what must be where it was when nothing moves.
    fn snapshot(home: &Home) -> BTreeMap<PathBuf, Vec<u8>> {
        files(home, &["Applications", "home/Library"])
    }

    /// The bytes of `files`, sorted: what the Trash holds once they went, whatever its names.
    fn bytes_of(files: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<Vec<u8>> {
        let mut bytes: Vec<Vec<u8>> = files.values().cloned().collect();
        bytes.sort();
        bytes
    }

    fn trashed_bytes(home: &Home) -> Vec<Vec<u8>> {
        bytes_of(&contents(&home.sandbox.trash()))
    }

    fn trash_entries(home: &Home) -> usize {
        fs::read_dir(home.sandbox.trash()).unwrap().count()
    }

    /// The confirmation of `view` to the Trash, as the dialog gives it.
    fn confirmation(view: &CleanupPlanView) -> Confirmation {
        confirmation_by(view, "trash")
    }

    /// The confirmation of `view` by `method`.
    fn confirmation_by(view: &CleanupPlanView, method: &str) -> Confirmation {
        let summary = view.summary;
        let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                           "highest": summary.highest, "method": method });
        serde_json::from_value(echo).unwrap()
    }

    fn tree(home: &Home, folder: &str) -> Value {
        serde_json::to_value(TreeReport::of(&home.path(folder)).unwrap()).unwrap()
    }

    fn exists(home: &Home, path: &str) -> bool {
        fs::symlink_metadata(home.path(path)).is_ok()
    }

    fn no_item_journalled(home: &Home) -> bool {
        home.journal().iter().all(|record| record["type"] != "item")
    }

    /// The scan's findings of `app-caches`, by path.
    fn app_caches(home: &Home) -> Vec<String> {
        home.findings().into_iter().filter(|(category, _)| category == APP_CACHES).map(|(_, path)| path).collect()
    }

    /// The look-alike's cache is where it was, byte for byte.
    fn the_look_alike_stays(home: &Home, before: &BTreeMap<PathBuf, Vec<u8>>) {
        let look_alike: BTreeMap<&PathBuf, &Vec<u8>> =
            before.iter().filter(|(path, _)| path.starts_with(home.path(LOOK_ALIKE))).collect();
        assert!(!look_alike.is_empty());
        for (path, bytes) in look_alike {
            assert_eq!(&fs::read(path).unwrap(), bytes, "the look-alike's cache stays");
        }
    }

    /// The scanned home and the plan of `choice` for Editor.
    fn planned(home: &Home, choice: UninstallChoice) -> CleanupPlanView {
        home.scan();
        home.uninstall(BUNDLE, choice).unwrap()
    }

    /// Confirms `view` and asserts its one item ended skipped `reason` and nothing moved: every file in place, the
    /// Trash empty, the app still listed.
    fn moves_nothing(home: &Home, view: &CleanupPlanView, reason: SkipReason) {
        let before = snapshot(home);

        let (_, events) = home.clean(view);

        assert_eq!(home.outcomes(view, &events), [(BUNDLE.to_owned(), ItemOutcome::Skipped { reason })]);
        assert_eq!(snapshot(home), before, "nothing of the app moved");
        assert_eq!(trash_entries(home), 0);
        assert!(home.app_id(BUNDLE).is_some(), "the app is still listed");
        the_look_alike_stays(home, &before);
    }

    /// Confirms `view` and asserts the confirmation is refused with `refused` and nothing was acted on: every file
    /// as it was, the Trash empty, no item journalled.
    fn refused_at_confirm(home: &Home, view: &CleanupPlanView, refused: &CommandError) {
        let before = snapshot(home);

        let error = home.engine.confirm_cleanup(&confirmation(view)).unwrap_err();

        assert_eq!(&error, refused);
        assert_eq!(snapshot(home), before, "nothing was acted on");
        assert_eq!(trash_entries(home), 0);
        assert!(no_item_journalled(home));
        the_look_alike_stays(home, &before);
    }

    // The negative flows.

    /// A wrong module claims every bundle of both apps folders, the scan walks them, and the user confirms its whole
    /// category: the engine's second wall refuses each bundle as protected, the real cleaner never acts, and every
    /// bundle stays byte for byte.
    pub(crate) fn an_ordinary_cleanup_never_moves_a_bundle() {
        let extra = r#"
            [[app]]
            path = "home/Applications/Sketch.app"
            bundle-id = "com.example.Sketch"
            name = "Sketch"
            version = "1.0"
        "#;
        let home = Home::with(Setup { extra, modules: vec![Box::new(BundleModule::new())], ..Setup::default() });
        home.walk_the_apps_folder();
        home.scan();
        let found: Vec<String> = home.findings().into_iter().map(|(_, path)| path).collect();
        assert_eq!(found, [BUNDLE, VIEWER, "home/Applications/Sketch.app"]);
        let before = files(&home, &["Applications", "home/Applications"]);
        let category = json!({ "findings": [], "categories": [{ "category": "bundles.apps", "except": [] }] });

        let view = home.engine.prepare_cleanup(&serde_json::from_value(category).unwrap()).unwrap();

        let reasons: Vec<&SkipReason> = view.left_out.iter().map(LeftOut::reason).collect();
        assert_eq!(reasons, [&SkipReason::Protected; 3], "every bundle is left out as protected");
        assert_eq!(view.summary.count, 0);
        assert_eq!(home.engine.confirm_cleanup(&confirmation(&view)), Err(CommandError::NothingToClean));
        assert_eq!(files(&home, &["Applications", "home/Applications"]), before, "every bundle is in place");
        assert_eq!(trash_entries(&home), 0);
        assert!(no_item_journalled(&home));
    }

    /// A wrong module claims the `Contents` inside every bundle of both apps folders, the scan walks them, and the user
    /// confirms its whole category: each is left out as protected (the engine refuses the apps folders, and the
    /// cleaner's own protection never opens a path inside a bundle), the real cleaner never acts, nothing moves.
    pub(crate) fn an_ordinary_cleanup_never_moves_a_path_inside_a_bundle() {
        let home = Home::with(Setup { modules: vec![Box::new(BundleModule::inside())], ..Setup::default() });
        home.walk_the_apps_folder();
        home.scan();
        let found: Vec<String> = home.findings().into_iter().map(|(_, path)| path).collect();
        assert_eq!(found, [format!("{BUNDLE}/Contents"), format!("{VIEWER}/Contents")]);
        let before = files(&home, &["Applications"]);
        let category = json!({ "findings": [], "categories": [{ "category": "bundles.apps", "except": [] }] });

        let view = home.engine.prepare_cleanup(&serde_json::from_value(category).unwrap()).unwrap();

        let reasons: Vec<&SkipReason> = view.left_out.iter().map(LeftOut::reason).collect();
        assert_eq!(reasons, [&SkipReason::Protected; 2], "every path inside a bundle is left out as protected");
        assert_eq!(view.summary.count, 0);
        assert_eq!(home.engine.confirm_cleanup(&confirmation(&view)), Err(CommandError::NothingToClean));
        for inside in [format!("{BUNDLE}/Contents"), format!("{BUNDLE}/Contents/Info.plist")] {
            let refused = flow::protection(&home.sandbox).check(&home.path(&inside));
            assert_eq!(refused, Err(fresher_core::Protected::Never), "the cleaner's own protection refuses {inside}");
        }
        assert_eq!(files(&home, &["Applications"]), before, "every bundle is in place");
        assert_eq!(trash_entries(&home), 0);
        assert!(no_item_journalled(&home));
    }

    /// The support folder is swapped for another folder of the same contents after the plan: the item is skipped
    /// `Replaced` and no path of it moves.
    pub(crate) fn a_part_replaced_after_the_plan_moves_nothing() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        let support = home.path(SUPPORT);
        let aside = support.with_extension("old");
        fs::rename(&support, &aside).unwrap();
        fs::create_dir(&support).unwrap();
        fs::copy(aside.join("library.db"), support.join("library.db")).unwrap();

        moves_nothing(&home, &view, SkipReason::Replaced);
    }

    /// A file of the caches is held open when the cleanup acts: the item is skipped in use and nothing moves.
    pub(crate) fn a_part_held_open_moves_nothing() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::KeepData);
        home.open_files.hold(home.path(CACHES).join("Cache.db"), "Editor");

        moves_nothing(&home, &view, SkipReason::InUse { app: "Editor".into() });
    }

    /// The support folder grew past the margin after the plan: the item is skipped `Grown` and nothing moves.
    pub(crate) fn a_part_grown_moves_nothing() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        fs::write(home.path(SUPPORT).join("grown.db"), vec![7_u8; 80 * 1024 * 1024]).unwrap();

        moves_nothing(&home, &view, SkipReason::Grown);
    }

    /// The app started after the plan: its executable is mapped when the cleanup acts, the item is skipped in use and
    /// nothing moves.
    pub(crate) fn the_app_opened_after_the_plan_moves_nothing() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::KeepData);
        let executable = home.path(BUNDLE).join("Contents").join("MacOS").join("Editor");
        assert_eq!(fs::read(&executable).unwrap(), EXECUTABLE);
        home.open_files.hold(executable, "Editor");

        moves_nothing(&home, &view, SkipReason::InUse { app: "Editor".into() });
    }

    /// The app's helper runs when the plan is asked for: the list says the app is open and the plan is refused.
    pub(crate) fn the_app_open_at_prepare_is_refused() {
        let home = Home::new();
        home.scan();
        home.running.open("Editor Helper", &[HELPER_ID]);
        let before = snapshot(&home);

        let error = home.uninstall(BUNDLE, UninstallChoice::KeepData).unwrap_err();

        assert_eq!(error, CommandError::UninstallRefused { reason: NotOffered::Open });
        let app = home.app_id(BUNDLE).unwrap();
        let offer = serde_json::to_value(home.engine.app(app).unwrap().summary.offer).unwrap();
        assert_eq!((&offer["kind"], &offer["reason"]), (&json!("not-offered"), &json!("open")), "{offer}");
        assert_eq!(snapshot(&home), before);
        assert_eq!(trash_entries(&home), 0);
    }

    /// The app starts between the plan and its confirmation: the confirmation asks the running apps again and is
    /// refused, nothing acted on.
    pub(crate) fn the_app_opened_before_the_confirmation_is_refused() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::KeepData);
        home.running.open("Editor", &[EDITOR_ID]);

        refused_at_confirm(&home, &view, &CommandError::UninstallRefused { reason: NotOffered::Open });
        assert!(home.app_id(BUNDLE).is_some(), "the app is still listed");
    }

    /// The plan of everything never offers a permanent delete, and a confirmation that asks for one anyway is refused
    /// as not matching the plan: nothing is acted on, the app stays listed.
    pub(crate) fn a_permanent_uninstall_is_refused() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        assert!(!view.permanent_available, "an uninstall always goes to the Trash");
        let before = snapshot(&home);

        let error = home.engine.confirm_cleanup(&confirmation_by(&view, "permanent")).unwrap_err();

        assert_eq!(error, CommandError::PlanMismatch);
        assert_eq!(snapshot(&home), before, "nothing was acted on");
        assert_eq!(trash_entries(&home), 0);
        assert!(no_item_journalled(&home));
        assert!(home.app_id(BUNDLE).is_some(), "the app is still listed");
    }

    /// The bundle is moved aside after the plan and a link of its name leads to it: the bundle is no longer on disk
    /// as listed, so the confirmation is refused and nothing is acted on.
    pub(crate) fn a_bundle_replaced_by_a_link_after_the_plan_is_refused() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        let aside = home.path("Applications/Editor.real");
        fs::rename(home.path(BUNDLE), &aside).unwrap();
        symlink(&aside, home.path(BUNDLE)).unwrap();

        refused_at_confirm(&home, &view, &CommandError::UninstallRefused { reason: NotOffered::Unmeasured });
        assert!(fs::symlink_metadata(home.path(BUNDLE)).unwrap().is_symlink(), "the link is in place");
    }

    /// The bundle is renamed to another case of its name after the plan, which this volume takes for the same name:
    /// its canonical path is not the listed one, so the confirmation is refused and nothing is acted on.
    pub(crate) fn a_bundle_renamed_after_the_plan_is_refused() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        fs::rename(home.path(BUNDLE), home.path("Applications/editor.app")).unwrap();

        refused_at_confirm(&home, &view, &CommandError::UninstallRefused { reason: NotOffered::Unmeasured });
        let names: Vec<_> =
            fs::read_dir(home.path("Applications")).unwrap().map(|entry| entry.unwrap().file_name()).collect();
        assert!(names.iter().any(|name| name == "editor.app"), "{names:?}");
    }

    /// A second copy of the app with the same id in the home's apps folder: every part may be the other copy's, so the
    /// plan of everything holds the bundle alone, and the data and the other copy stay once it went.
    pub(crate) fn a_second_copy_keeps_the_data() {
        let extra = r#"
            [[app]]
            path = "home/Applications/Editor.app"
            bundle-id = "com.example.Editor"
            name = "Editor"
            version = "1.0"
        "#;
        let home = Home::with(Setup { extra, ..Setup::default() });
        let view = planned(&home, UninstallChoice::Everything);
        assert_eq!(home.planned(&view), [BUNDLE]);
        let parts = files(&home, &every_part()[1..]);
        let copy = files(&home, &[SECOND_COPY]);

        let (_, events) = home.clean(&view);

        assert_eq!(home.outcomes(&view, &events), [(BUNDLE.to_owned(), ItemOutcome::Moved)]);
        assert!(!exists(&home, BUNDLE));
        assert_eq!(files(&home, &every_part()[1..]), parts, "the data stays");
        assert_eq!(files(&home, &[SECOND_COPY]), copy, "the other copy stays");
        assert!(home.app_id(SECOND_COPY).is_some(), "the other copy is still listed");
    }

    /// The look-alike's cache is never one of Editor's parts, and an uninstall of everything leaves it in place.
    pub(crate) fn the_look_alike_s_cache_stays() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        let app = home.app_id(BUNDLE).unwrap();
        let items = home.engine.app(app).unwrap().items;
        assert!(items.iter().all(|item| !item.path.starts_with(home.path(LOOK_ALIKE))), "{items:?}");
        assert!(home.planned(&view).iter().all(|path| path != LOOK_ALIKE));
        let before = files(&home, &[LOOK_ALIKE]);

        home.clean(&view);

        assert!(!exists(&home, BUNDLE), "the uninstall acted");
        the_look_alike_stays(&home, &before);
    }

    /// A scan after the plan makes a new state: the plan is gone, and nothing moves.
    pub(crate) fn a_new_scan_kills_the_plan() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        home.scan();

        refused_at_confirm(&home, &view, &CommandError::PlanGone);
        assert!(home.app_id(BUNDLE).is_some());
    }

    /// The bundle moved one folder down after the plan, and a scan listed it there: the plan is of another bundle path,
    /// it is gone, and nothing moves.
    pub(crate) fn a_bundle_moved_and_scanned_again_kills_the_plan() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        let moved = "Applications/Tools/Editor.app";
        fs::create_dir(home.path("Applications/Tools")).unwrap();
        fs::rename(home.path(BUNDLE), home.path(moved)).unwrap();
        home.scan();
        assert!(home.app_id(moved).is_some(), "the scan lists the bundle where it is now");

        refused_at_confirm(&home, &view, &CommandError::PlanGone);
    }

    // The positive flows.

    /// Keep-data moves exactly the bundle, its caches and its helper's, its logs, its saved windows and its launch
    /// agent to the Trash as one item, the bundle first in the journal; web data, settings, support and the group
    /// container stay; the app leaves the list and `app-caches`' findings of its caches leave the results. The report
    /// is the golden file.
    pub(crate) fn keep_data_moves_the_app_and_its_caches_logs_saved_state_and_agent() -> Value {
        let home = Home::new();
        home.scan();
        assert_eq!(app_caches(&home), [CACHES, HELPER_CACHES]);
        let view = home.uninstall(BUNDLE, UninstallChoice::KeepData).unwrap();
        assert_eq!(home.planned(&view), KEEP_DATA);
        let moving = files(&home, &KEEP_DATA);
        let staying = files(&home, &DATA);
        let before = snapshot(&home);

        let (_, events) = home.clean(&view);

        let outcomes = home.outcomes(&view, &events);
        assert_eq!(outcomes, [(BUNDLE.to_owned(), ItemOutcome::Moved)]);
        assert!(KEEP_DATA.iter().all(|path| !exists(&home, path)), "every path of the choice went");
        assert_eq!(trash_entries(&home), KEEP_DATA.len());
        assert_eq!(trashed_bytes(&home), bytes_of(&moving), "exactly those files are in the Trash");
        assert_eq!(files(&home, &DATA), staying, "the data stays");
        let journal = home.journal();
        let item = journal.iter().find(|record| record["type"] == "item").unwrap();
        let originals: Vec<&Value> = item["trashed"].as_array().unwrap().iter().map(|part| &part["original"]).collect();
        let expected: Vec<Value> = KEEP_DATA.iter().map(|path| json!(home.path(path))).collect();
        assert_eq!(originals, expected.iter().collect::<Vec<_>>(), "the bundle first, then the parts");
        assert_eq!(home.app_id(BUNDLE), None, "the app leaves the list");
        assert!(home.app_id(VIEWER).is_some());
        assert_eq!(app_caches(&home), Vec::<String>::new(), "the caches' findings leave the results");
        the_look_alike_stays(&home, &before);
        let report = json!({
            "plan": view,
            "outcomes": outcomes,
            "journal": journal,
            "appsAfter": tree(&home, "Applications"),
            "homeAfter": tree(&home, "home"),
        });
        home.relative(&report)
    }

    /// Everything moves every path of the app, the group container included while other apps' data is granted; with
    /// it denied, the group container is kept and stays.
    pub(crate) fn everything_moves_every_part() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        assert_eq!(home.planned(&view), every_part());
        let moving = files(&home, &every_part());
        let before = snapshot(&home);

        let (_, events) = home.clean(&view);

        assert_eq!(home.outcomes(&view, &events), [(BUNDLE.to_owned(), ItemOutcome::Moved)]);
        assert!(every_part().iter().all(|path| !exists(&home, path)), "every part went");
        assert_eq!(trash_entries(&home), every_part().len());
        assert_eq!(trashed_bytes(&home), bytes_of(&moving));
        the_look_alike_stays(&home, &before);

        let home = Home::with(Setup { other_apps_data: Access::Denied, ..Setup::default() });
        let view = planned(&home, UninstallChoice::Everything);
        let without_group: Vec<&str> = every_part().into_iter().filter(|path| *path != GROUP).collect();
        assert_eq!(home.planned(&view), without_group);
        let group = files(&home, &[GROUP]);

        home.clean(&view);

        assert!(without_group.iter().all(|path| !exists(&home, path)));
        assert_eq!(files(&home, &[GROUP]), group, "the group container stays without access to other apps' data");
    }

    /// Put back of an everything cleanup brings every part back byte for byte, beside a file written into `Caches`
    /// since; the journal holds the bundle first, so put back takes it last.
    pub(crate) fn put_back_brings_the_whole_app_back() {
        let home = Home::new();
        let view = planned(&home, UninstallChoice::Everything);
        let before = files(&home, &every_part());
        let (cleanup, _) = home.clean(&view);
        assert!(every_part().iter().all(|path| !exists(&home, path)));
        let written = home.path("home/Library/Caches/written-since.db");
        fs::write(&written, b"written after the uninstall").unwrap();

        let back = home.engine.put_back(cleanup, None).unwrap();

        assert_eq!(back.items.len(), 1);
        assert_eq!(back.items[0].outcome, PutBackOutcome::PutBack, "{back:?}");
        assert_eq!(files(&home, &every_part()), before, "every part is back byte for byte");
        assert_eq!(fs::read(&written).unwrap(), b"written after the uninstall", "the new file stays");
        assert_eq!(trash_entries(&home), 0);
        let journal = home.journal();
        let item = journal.iter().find(|record| record["type"] == "item").unwrap();
        let originals: Vec<&Value> = item["trashed"].as_array().unwrap().iter().map(|part| &part["original"]).collect();
        let expected: Vec<Value> = every_part().iter().map(|path| json!(home.path(path))).collect();
        assert_eq!(originals, expected.iter().collect::<Vec<_>>(), "the bundle first: put back takes it last");
    }

    /// Viewer has no data: its plan is one path, the bundle, moved alone.
    pub(crate) fn an_app_with_no_data_is_one_path() {
        let home = Home::new();
        home.scan();
        let view = home.uninstall(VIEWER, UninstallChoice::Everything).unwrap();
        assert_eq!(home.planned(&view), [VIEWER]);
        let bundle = files(&home, &[VIEWER]);

        let (_, events) = home.clean(&view);

        assert_eq!(home.outcomes(&view, &events), [(VIEWER.to_owned(), ItemOutcome::Moved)]);
        assert!(!exists(&home, VIEWER));
        assert_eq!(trash_entries(&home), 1);
        assert_eq!(trashed_bytes(&home), bytes_of(&bundle));
        assert_eq!(home.app_id(VIEWER), None);
        assert!(home.app_id(BUNDLE).is_some(), "Editor stays");
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn an_ordinary_cleanup_never_moves_a_bundle_in_either_apps_folder() {
        flows::an_ordinary_cleanup_never_moves_a_bundle();
    }

    #[test]
    fn an_ordinary_cleanup_never_moves_a_path_inside_a_bundle() {
        flows::an_ordinary_cleanup_never_moves_a_path_inside_a_bundle();
    }

    #[test]
    fn a_permanent_uninstall_is_refused() {
        flows::a_permanent_uninstall_is_refused();
    }

    #[test]
    fn a_part_replaced_after_the_plan_moves_nothing() {
        flows::a_part_replaced_after_the_plan_moves_nothing();
    }

    #[test]
    fn a_part_held_open_moves_nothing() {
        flows::a_part_held_open_moves_nothing();
    }

    #[test]
    fn a_part_grown_past_the_margin_moves_nothing() {
        flows::a_part_grown_moves_nothing();
    }

    #[test]
    fn the_app_opened_after_the_plan_moves_nothing() {
        flows::the_app_opened_after_the_plan_moves_nothing();
    }

    #[test]
    fn the_app_open_at_prepare_is_refused() {
        flows::the_app_open_at_prepare_is_refused();
    }

    #[test]
    fn the_app_opened_between_the_plan_and_the_confirmation_is_refused_at_confirm() {
        flows::the_app_opened_before_the_confirmation_is_refused();
    }

    #[test]
    fn a_bundle_replaced_by_a_link_after_the_plan_is_refused_at_confirm() {
        flows::a_bundle_replaced_by_a_link_after_the_plan_is_refused();
    }

    #[test]
    fn a_bundle_renamed_to_another_case_after_the_plan_is_refused_at_confirm() {
        flows::a_bundle_renamed_after_the_plan_is_refused();
    }

    #[test]
    fn a_second_copy_keeps_the_data() {
        flows::a_second_copy_keeps_the_data();
    }

    #[test]
    fn the_look_alike_s_cache_stays() {
        flows::the_look_alike_s_cache_stays();
    }

    #[test]
    fn a_new_scan_kills_the_plan() {
        flows::a_new_scan_kills_the_plan();
    }

    #[test]
    fn a_bundle_moved_and_scanned_again_kills_the_plan() {
        flows::a_bundle_moved_and_scanned_again_kills_the_plan();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn keep_data_moves_the_app_and_its_caches_logs_saved_state_and_agent() {
        check_golden("uninstall-flow", &flows::keep_data_moves_the_app_and_its_caches_logs_saved_state_and_agent());
    }

    #[test]
    fn everything_moves_every_part() {
        flows::everything_moves_every_part();
    }

    #[test]
    fn put_back_brings_the_whole_app_back() {
        flows::put_back_brings_the_whole_app_back();
    }

    #[test]
    fn an_app_with_no_data_is_one_path() {
        flows::an_app_with_no_data_is_one_path();
    }
}
