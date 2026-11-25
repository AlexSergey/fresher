//! The app-leftovers module end to end: a fixture sandbox's home with a removed app's leftovers in the nine forms and
//! its container, a sandboxed app with only a container and its caches (not claimed: the module reads no container,
//! so nothing proves it), an installed app's and Apple's files, and another module's cache and older IDE version
//! beside them under `Library/Caches`; the real scanner (its reader seen through the facts lens),
//! the modules the app registers, the real settings file, scan cache and journal, and the real `DiskCleaner` reaching
//! only inside the sandbox, with a fake Trash. The clock, the installed and open apps, Spotlight's answers and the
//! change journal's answers are fakes a flow sets. The flows of the design's section 6, "Core e2e"; flow 1's report
//! is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test app_leftovers` rewrites
//! `fixtures/golden/app-leftovers-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "app_leftovers_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::Duration;

    use fresher_core::{
        DangerLevel, IgnoreTarget, IgnoredEntry, ItemOutcome, MeaningGroup, ModuleGroup, PutBackOutcome,
    };
    use fresher_testing::{FIXTURE_NOW, FakeClock, FakeInstalledApps, FakeRunningApps, TreeReport, running_app};
    use serde_json::{Value, json};

    use crate::flow::{Avoid, Flow, MODULE, Setup, app, installed_apps, left_out};
    use crate::tree::contents;

    const CACHES: &str = "app-leftovers.caches";
    const SETTINGS: &str = "app-leftovers.settings";
    const DATA: &str = "app-leftovers.data";
    const DAY: u64 = 24 * 60 * 60;

    /// The removed app.
    const GONE_ID: &str = "com.example.Gone";
    /// The removed app's nine leftovers, one per form, in the order `Flow::findings` sorts them.
    const GONE: [(&str, &str); 9] = [
        (CACHES, "home/Library/Caches/com.example.Gone"),
        (CACHES, "home/Library/Logs/com.example.Gone"),
        (CACHES, "home/Library/Saved Application State/com.example.Gone.savedState"),
        (DATA, "home/Library/Application Support/com.example.Gone"),
        (SETTINGS, "home/Library/Cookies/com.example.Gone.binarycookies"),
        (SETTINGS, "home/Library/HTTPStorages/com.example.Gone"),
        (SETTINGS, "home/Library/HTTPStorages/com.example.Gone.binarycookies"),
        (SETTINGS, "home/Library/Preferences/com.example.Gone.plist"),
        (SETTINGS, "home/Library/WebKit/com.example.Gone"),
    ];
    /// The removed sandboxed app's caches: nothing proves the app, so they are never claimed.
    const BOXED_CACHES: &str = "home/Library/Caches/com.example.Boxed";
    /// The two apps' containers, which the module does not read.
    const CONTAINERS: [&str; 2] =
        ["home/Library/Containers/com.example.Gone", "home/Library/Containers/com.example.Boxed"];
    /// The other modules' answers under `Library/Caches`: Yarn's cache and `WebStorm`'s older version.
    const YARN: (&str, &str) = ("dev-caches.yarn", "home/Library/Caches/Yarn");
    const WEBSTORM: (&str, &str) = ("jetbrains.caches", "home/Library/Caches/JetBrains/WebStorm2024.3");
    /// The files never claimed: the installed app's, Apple's, and the version of `WebStorm` in use.
    const KEPT: [&str; 6] = [
        "home/Library/Caches/com.example.Editor",
        "home/Library/Saved Application State/com.example.Editor.savedState",
        "home/Library/Preferences/com.example.Editor.plist",
        "home/Library/Caches/com.apple.Safari",
        "home/Library/Saved Application State/com.apple.Safari.savedState",
        "home/Library/Caches/JetBrains/WebStorm2025.3",
    ];
    /// The folders a cleanup of the caches category changes (the leftovers' and the Trash), for the change journal
    /// of the rescan after it. `home/.Trash` is named too: the rescan checks a sample of the folders the journal does
    /// not name against the disk, and the Trash, changed but unnamed, would make it refuse the rescan.
    const CLEANED: [&str; 4] =
        ["home/Library/Caches", "home/Library/Logs", "home/Library/Saved Application State", "home/.Trash"];

    fn found(leftovers: &[(&str, &str)]) -> Vec<(String, String)> {
        let mut found: Vec<(String, String)> =
            leftovers.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
    }

    /// `leftovers` and the other modules' two answers.
    fn with_others(leftovers: &[(&str, &str)]) -> Vec<(String, String)> {
        found(&leftovers.iter().copied().chain([YARN, WEBSTORM]).collect::<Vec<_>>())
    }

    /// Every leftover of the home and the other modules' two answers.
    fn everything() -> Vec<(String, String)> {
        with_others(&GONE)
    }

    /// `leftovers` but those of `category`.
    fn but(leftovers: &[(&'static str, &'static str)], category: &str) -> Vec<(&'static str, &'static str)> {
        leftovers.iter().copied().filter(|(of, _)| *of != category).collect()
    }

    /// The findings of the module only.
    fn ours(flow: &Flow) -> Vec<(String, String)> {
        flow.findings().into_iter().filter(|(category, _)| category.starts_with("app-leftovers.")).collect()
    }

    /// The findings of `module` only.
    fn of_module(flow: &Flow, module: &str) -> Vec<(String, String)> {
        let prefix = format!("{module}.");
        flow.findings().into_iter().filter(|(category, _)| category.starts_with(&prefix)).collect()
    }

    /// The counts left out with `reason`, as `(caches, data, settings)` items.
    fn kept(reason: &str, (caches, data, settings): (u32, u32, u32)) -> Vec<(String, String, u32)> {
        [("caches", caches), ("data", data), ("settings", settings)]
            .into_iter()
            .filter(|(_, items)| *items > 0)
            .map(|(category, items)| (category.to_owned(), reason.to_owned(), items))
            .collect()
    }

    fn contents_of(flow: &Flow, paths: &[&str]) -> Vec<BTreeMap<PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&flow.path(path))).collect()
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path("home")).unwrap()).unwrap()
    }

    /// The categories of the overview's totals, sorted.
    fn totals(flow: &Flow) -> Vec<String> {
        let mut totals: Vec<String> =
            flow.engine.overview().unwrap().totals.iter().map(|total| total.category.as_str().to_owned()).collect();
        totals.sort();
        totals
    }

    fn total_bytes(flow: &Flow, leftovers: &[(&str, &str)]) -> u64 {
        leftovers
            .iter()
            .map(|(_, path)| contents(&flow.path(path)).values().map(|bytes| bytes.len() as u64).sum::<u64>())
            .sum()
    }

    /// A folder made unreadable (mode 000) until the guard is dropped, whatever the flow's outcome.
    struct Locked {
        path: PathBuf,
        mode: u32,
    }

    impl Locked {
        fn new(path: &Path) -> Self {
            let mode = fs::metadata(path).unwrap().permissions().mode();
            fs::set_permissions(path, fs::Permissions::from_mode(0o000)).unwrap();
            Self { path: path.to_path_buf(), mode }
        }
    }

    impl Drop for Locked {
        fn drop(&mut self) {
            // Restored even while a failed assertion unwinds, so the temporary folder can be removed after.
            let _restored = fs::set_permissions(&self.path, fs::Permissions::from_mode(self.mode));
        }
    }

    /// Flow 1: Apps → Removed apps' leftovers offers the removed apps' leftovers beside the other modules' answers
    /// under `Library/Caches`; "Remove everything" selects the three categories and the plan names High; the
    /// leftovers go to the Trash and the journal and come back byte for byte; the containers, the installed app's,
    /// Apple's and the other modules' files stay.
    pub(crate) fn scan_confirm_trash_and_put_back() -> Value {
        let flow = Flow::new();
        let events = flow.scan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
        let module = flow.engine.modules().into_iter().find(|module| module.id.as_str() == MODULE).unwrap();
        assert_eq!(module.group, ModuleGroup::Apps);
        let leftovers: Vec<(&str, &str)> = GONE.to_vec();
        let before = contents_of(&flow, &leftovers.iter().map(|(_, path)| *path).collect::<Vec<_>>());
        assert!(before.iter().all(|files| !files.is_empty() && files.values().all(|bytes| !bytes.is_empty())));
        let kept = contents_of(&flow, &KEPT);
        assert!(kept.iter().all(|files| !files.is_empty()));

        let view = flow.engine.prepare_cleanup(&flow.choice("everything")).unwrap();

        assert_eq!(view.summary.count, 9);
        assert_eq!(view.summary.highest, DangerLevel::High);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        let mut planned: Vec<&str> = leftovers.iter().map(|(_, path)| *path).collect();
        planned.sort_unstable();
        assert_eq!(outcomes.iter().map(|(path, _)| path.as_str()).collect::<Vec<_>>(), planned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(planned.iter().all(|path| !flow.path(path).exists()));
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 9);
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 9);
        assert_eq!(contents_of(&flow, &KEPT), kept, "the installed app's, Apple's and the IDE in use's files stay");
        assert!(CONTAINERS.iter().all(|path| flow.path(path).is_dir()), "the containers stay");
        assert!(flow.path(BOXED_CACHES).is_dir(), "the caches of an app nothing proves stay");
        assert_eq!(flow.findings(), with_others(&[]), "the other modules' answers stay");
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let after = contents_of(&flow, &leftovers.iter().map(|(_, path)| *path).collect::<Vec<_>>());
        assert_eq!(after, before, "put back byte for byte");
        assert_eq!(contents_of(&flow, &KEPT), kept);
        let report = json!({
            "offered": everything(),
            "leftOut": left_out(&events),
            "plan": view,
            "outcomes": outcomes,
            "treeAfterCleanup": tree_after_cleanup,
            "putBack": back,
            "journal": flow.journal(),
            "treeAfterPutBack": tree(&flow),
        });
        flow.relative(&report)
    }

    /// Flow 2: "Keep settings" selects the caches category alone and the plan names Safe; after the cleanup, a rescan
    /// claims nothing of the module: the saved state, the removed app's only proof, went with the caches, so its
    /// settings and data stay unclaimed.
    pub(crate) fn keeping_settings_cleans_only_the_caches() {
        let flow = Flow::with(Setup { changed: vec![CLEANED.to_vec()], ..Setup::default() });
        flow.scan();

        let view = flow.engine.prepare_cleanup(&flow.choice("keep-settings")).unwrap();

        assert_eq!(view.summary.highest, DangerLevel::Safe);
        let caches: Vec<(&str, &str)> = GONE.iter().copied().filter(|(of, _)| *of == CACHES).collect();
        let (_, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert_eq!(
            outcomes.iter().map(|(path, _)| path.clone()).collect::<Vec<_>>(),
            found(&caches).into_iter().map(|(_, path)| path).collect::<Vec<_>>()
        );
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");

        let events = flow.rescan();

        assert_eq!(flow.findings(), with_others(&[]));
        assert_eq!(left_out(&events), []);
    }

    /// Flow 3: the removed app installed again between two scans: a rescan claims nothing of it.
    pub(crate) fn an_app_installed_again_keeps_its_files() {
        let again: Vec<_> = installed_apps().into_iter().chain([app(GONE_ID, "Gone", "2.0")]).collect();
        // One listing of the installed apps per scan: the first scan sees the app removed, the rescan installed.
        let installed = FakeInstalledApps::new(installed_apps()).then(again);
        let flow = Flow::with(Setup { installed, changed: vec![Vec::new()], ..Setup::default() });
        flow.scan();
        assert_eq!(flow.findings(), everything());

        let events = flow.rescan();

        assert_eq!(flow.findings(), with_others(&[]));
        assert_eq!(left_out(&events), []);
    }

    /// Flow 4: the removed app open from a disk image: its leftovers are left out `running`; it quits, and a rescan
    /// offers them.
    pub(crate) fn an_app_open_from_a_disk_image_keeps_its_files() {
        // One listing of the open apps per scan: the first scan sees the app open, the rescan sees it quit.
        let running = FakeRunningApps::new(vec![running_app("Gone", &[GONE_ID]).unwrap()]).then(Vec::new());
        let flow = Flow::with(Setup { running, changed: vec![Vec::new()], ..Setup::default() });

        let events = flow.scan();

        assert_eq!(flow.findings(), with_others(&[]));
        assert_eq!(left_out(&events), kept("running", (3, 1, 5)));

        let events = flow.rescan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
    }

    /// Flow 5: the leftovers changed 20 days ago, within the setting's 30: left out `used-recently`; 12 days on, a
    /// rescan that reads nothing again offers them. (Past the journal's 14 days the scanner trusts no rescan.)
    pub(crate) fn files_changed_lately_wait_for_the_days() {
        let clock = Arc::new(FakeClock::at(FIXTURE_NOW - 20 * DAY));
        let flow = Flow::with(Setup { clock: Arc::clone(&clock), changed: vec![Vec::new()], ..Setup::default() });

        let events = flow.scan();

        assert_eq!(flow.findings(), with_others(&[]));
        assert_eq!(left_out(&events), kept("used-recently", (3, 1, 5)));

        clock.advance(Duration::from_secs(12 * DAY));
        let events = flow.rescan();

        assert_eq!(flow.findings(), everything());
        assert_eq!(left_out(&events), []);
    }

    /// Flow 6: Spotlight finds the removed app elsewhere: its leftovers are left out `installed-elsewhere`; Spotlight
    /// cannot see every app: every removed app's leftovers are left out `spotlight-unread`.
    pub(crate) fn spotlight_keeps_what_it_finds_or_cannot_see() {
        let mut far = app(GONE_ID, "Gone", "1.0");
        far.path = "/Volumes/Drive/Gone.app".into();
        let installed = FakeInstalledApps::new(installed_apps()).with_elsewhere(vec![far]);
        let mut flow = Flow::with(Setup { installed, ..Setup::default() });

        let events = flow.scan();

        assert_eq!(flow.findings(), with_others(&[]));
        assert_eq!(left_out(&events), kept("installed-elsewhere", (3, 1, 5)));

        let installed = FakeInstalledApps::new(installed_apps()).failing_search("the index is off");
        flow.restart(Setup { installed, ..Setup::default() });
        let events = flow.scan();

        assert_eq!(flow.findings(), with_others(&[]));
        assert_eq!(left_out(&events), kept("spotlight-unread", (3, 1, 5)));
    }

    /// Flow 7: `Library/Logs` cannot be read: every proven id is left out `scan-again` (the removed app's logs
    /// unseen), while `dev-caches`' answer under `Library/Caches` is the same as with the logs read. The guard gives
    /// the folder its mode back, whatever the outcome.
    pub(crate) fn a_place_not_read_keeps_every_app() {
        let flow = Flow::new();
        flow.scan();
        let dev_caches = of_module(&flow, "dev-caches");
        assert_eq!(dev_caches, found(&[YARN]));
        let logs = flow.path("home/Library/Logs");
        let unreadable = Locked::new(&logs);

        let events = flow.scan();

        assert_eq!(ours(&flow), []);
        assert_eq!(left_out(&events), kept("scan-again", (2, 1, 5)));
        assert_eq!(of_module(&flow, "dev-caches"), dev_caches);
        drop(unreadable);
        assert!(fs::read_dir(&logs).is_ok(), "the folder's mode is given back");
    }

    /// Flow 8: the places that may prompt avoided, as the app composes them: the module reads no container, so the
    /// removed app proven by its saved state is offered as when the containers are seen, and the container-only app is
    /// not claimed either way.
    pub(crate) fn containers_are_not_opened_until_asked() {
        let flow = Flow::with(Setup { avoid: Avoid::PlacesThatPrompt, ..Setup::default() });

        let events = flow.scan();

        assert_eq!(flow.findings(), with_others(&GONE));
        assert_eq!(left_out(&events), []);
        assert!(CONTAINERS.iter().all(|path| flow.path(path).is_dir()), "the containers stay");
    }

    /// Flow 10: the storage map counts the removed app's offered leftovers in a line named by its id, not installed.
    pub(crate) fn the_map_names_a_removed_app_by_its_id() {
        let flow = Flow::new();
        flow.scan();

        let groups = flow.engine.storage_meaning(0).unwrap();

        let (app, sizes, categories) = groups
            .iter()
            .find_map(|group| match group {
                MeaningGroup::App { app, sizes, categories, .. } if app.id.as_str() == GONE_ID => {
                    Some((app, sizes, categories))
                }
                _ => None,
            })
            .expect("the removed app has a line");
        assert_eq!((app.name.as_str(), app.installed), (GONE_ID, false));
        let mut shares: Vec<(&str, u64)> =
            categories.iter().map(|share| (share.category.as_str(), share.sizes.logical.bytes())).collect();
        shares.sort_unstable();
        let bytes = |category: &str| {
            total_bytes(&flow, &GONE.iter().copied().filter(|(of, _)| *of == category).collect::<Vec<_>>())
        };
        assert_eq!(shares, [(CACHES, bytes(CACHES)), (DATA, bytes(DATA)), (SETTINGS, bytes(SETTINGS))]);
        assert_eq!(sizes.logical.bytes(), shares.iter().map(|(_, bytes)| bytes).sum::<u64>());
    }

    /// Flow 11: the data category ignored: its findings are in no total while the caches and the settings stay;
    /// restoring brings it back without a scan.
    pub(crate) fn ignoring_the_data_keeps_the_rest() {
        let flow = Flow::new();
        flow.scan();
        let all_totals = totals(&flow);
        assert!(all_totals.iter().any(|category| category == DATA));
        let found_bytes = |flow: &Flow| flow.engine.overview().unwrap().found.logical.bytes();
        let all_found = found_bytes(&flow);
        let leftovers: Vec<(&str, &str)> = GONE.to_vec();

        flow.engine.ignore(&IgnoreTarget::Category { category: DATA.into() }).unwrap();

        assert_eq!(flow.findings(), with_others(&but(&leftovers, DATA)));
        assert_eq!(totals(&flow), all_totals.iter().filter(|category| *category != DATA).cloned().collect::<Vec<_>>());
        let data: Vec<(&str, &str)> = leftovers.iter().copied().filter(|(of, _)| *of == DATA).collect();
        assert_eq!(found_bytes(&flow), all_found - total_bytes(&flow, &data));

        flow.engine.restore_ignored(&IgnoredEntry::Category { category: DATA.into() }).unwrap();

        assert_eq!(flow.findings(), everything());
        assert_eq!(totals(&flow), all_totals);
        assert_eq!(found_bytes(&flow), all_found);
    }
}

/// What the module's categories can do with the access of a system: an engine on an empty home with the app's
/// `app-leftovers` module only, its access the fake's states. `cfg(test)` lets Clippy's in-test allowances apply to
/// these helpers, as the crate's other helper modules have it.
#[cfg(test)]
mod access {
    use std::sync::Arc;

    use fresher_core::{Access, Availability, Capability};
    use fresher_engine::{Engine, Ports, Registry};
    use fresher_module_app_leftovers::AppLeftoversModule;
    use fresher_store::{DataFolder, FileSettingsStore};
    use fresher_testing::{FIXTURE_NOW, FakeAccess, FakeClock, FakeCommandRunner, FakeSystemUi, NaiveWalk};

    /// Sandbox mode's stand-in, as the app's `UnknownAccess` answers: Full Disk Access `Unknown`, every other
    /// permission `NotAsked`.
    pub(crate) fn unknown() -> FakeAccess {
        let state = |capability| {
            (capability, if capability == Capability::AllFiles { Access::Unknown } else { Access::NotAsked })
        };
        FakeAccess::with(&Capability::SYSTEM.map(state))
    }

    /// Each category of `app-leftovers` with its availability under `access`, in the module's order.
    pub(crate) fn availability(access: FakeAccess) -> Vec<(String, Availability)> {
        let (folder, home) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        let mut registry = Registry::new().unwrap();
        registry.register(Box::new(AppLeftoversModule::new().unwrap())).unwrap();
        let ports = Ports {
            walk: Arc::new(NaiveWalk::new(home.path())),
            settings: Arc::new(FileSettingsStore::new(DataFolder::open(folder.path()).unwrap())),
            clock: Arc::new(FakeClock::at(FIXTURE_NOW)),
            commands: Arc::new(FakeCommandRunner::default()),
            access: Arc::new(access),
            system_ui: Arc::new(FakeSystemUi::default()),
            system: fresher_testing::fake_system(home.path()),
        };
        let engine = Engine::new(registry, ports, fresher_core::HighLevel::Offered).unwrap();
        let module = engine.modules().into_iter().next().unwrap();
        module
            .categories
            .into_iter()
            .map(|category| (category.descriptor.id().as_str().to_owned(), category.availability))
            .collect()
    }
}

mod negative {
    use crate::flows::{
        a_place_not_read_keeps_every_app, an_app_installed_again_keeps_its_files,
        an_app_open_from_a_disk_image_keeps_its_files, containers_are_not_opened_until_asked,
        files_changed_lately_wait_for_the_days, ignoring_the_data_keeps_the_rest,
        spotlight_keeps_what_it_finds_or_cannot_see,
    };

    #[test]
    fn an_app_installed_again_has_nothing_claimed_by_the_rescan() {
        an_app_installed_again_keeps_its_files();
    }

    #[test]
    fn an_open_app_is_left_out_running_and_offered_by_a_rescan_once_it_quits() {
        an_app_open_from_a_disk_image_keeps_its_files();
    }

    #[test]
    fn leftovers_changed_within_the_days_are_left_out_and_offered_by_a_rescan_once_they_pass() {
        files_changed_lately_wait_for_the_days();
    }

    #[test]
    fn an_app_spotlight_finds_or_a_search_that_cannot_see_every_app_leaves_the_leftovers_out() {
        spotlight_keeps_what_it_finds_or_cannot_see();
    }

    #[test]
    fn an_unreadable_logs_folder_leaves_every_id_out_and_dev_caches_answer_unchanged() {
        a_place_not_read_keeps_every_app();
    }

    #[test]
    fn with_containers_avoided_a_container_only_id_is_not_claimed() {
        containers_are_not_opened_until_asked();
    }

    #[test]
    fn an_ignored_category_is_in_no_total_until_restored() {
        ignoring_the_data_keeps_the_rest();
    }

    /// The module reads no container, so it needs no permission: under sandbox mode's stand-in (nothing granted)
    /// every category is ready.
    #[test]
    fn without_data_from_other_apps_every_category_is_still_ready() {
        use fresher_core::Availability;

        let availability = crate::access::availability(crate::access::unknown());

        assert_eq!(availability.len(), 3);
        assert!(availability.iter().all(|(_, availability)| *availability == Availability::Ready), "{availability:?}");
    }
}

mod positive {
    use crate::flows::{
        keeping_settings_cleans_only_the_caches, scan_confirm_trash_and_put_back, the_map_names_a_removed_app_by_its_id,
    };
    use crate::support::check_golden;

    #[test]
    fn the_leftovers_go_to_the_trash_and_come_back_byte_for_byte() {
        check_golden("app-leftovers-flow", &scan_confirm_trash_and_put_back());
    }

    #[test]
    fn keeping_the_settings_cleans_the_caches_and_a_rescan_claims_nothing_without_the_proof() {
        keeping_settings_cleans_only_the_caches();
    }

    #[test]
    fn the_map_counts_the_leftovers_in_the_removed_apps_line() {
        the_map_names_a_removed_app_by_its_id();
    }
}
