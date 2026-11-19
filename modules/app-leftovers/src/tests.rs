//! The module as built, and the module through `on_match` and `finish`, on a recording sink, with every system fact
//! faked and plain `Match` values in a home at `/home`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{
    BundleId, ByteSize, CancelToken, EntryFacts, EntryKind, Inside, InstalledApps, InstalledList, LeftOutItem, Match,
    Module as _, ModuleContext, NewFinding, PlaceSeen, SettingsSchema, Sizes, StoredSettings, SystemError, SystemFacts,
    SystemPorts, Timestamp,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::AppLeftoversModule;

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_767_225_600;
const GONE: &str = "com.example.Gone";
const OTHER: &str = "com.example.Other";

/// The module's interests, by index, and the folder under `/home/Library` each is in.
const CACHES: usize = 0;
const LOGS: usize = 1;
const SAVED_STATE: usize = 2;
const PREFERENCES: usize = 3;
const HTTP: usize = 4;
const COOKIES: usize = 6;
const SUPPORT: usize = 7;
const FOLDERS: [&str; 8] = [
    "Caches",
    "Logs",
    "Saved Application State",
    "Preferences",
    "HTTPStorages",
    "WebKit",
    "Cookies",
    "Application Support",
];

/// Keeps everything the module reports, in order.
#[derive(Debug, Default, PartialEq, Eq)]
struct Recorded {
    findings: Vec<NewFinding>,
    left_out: Vec<LeftOutItem>,
}

impl Recorded {
    /// Each left-out as its category, reason and allocated bytes.
    fn left_out(&self) -> Vec<(&str, &str, u64)> {
        self.left_out
            .iter()
            .map(|item| (item.category.as_str(), item.reason.as_str(), item.sizes.allocated.bytes()))
            .collect()
    }

    /// Each finding's identity.
    fn identities(&self) -> Vec<&str> {
        self.findings.iter().map(|finding| finding.key().identity()).collect()
    }
}

impl fresher_core::FindingSink for Recorded {
    fn push(&mut self, finding: NewFinding) {
        self.findings.push(finding);
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        self.left_out.push(left_out);
    }
}

/// The installed apps of a fake, recording the ids Spotlight is asked for, one list per ask; built `cancelling`, each
/// ask cancels the scan.
struct Spotlight {
    apps: FakeInstalledApps,
    asked: Mutex<Vec<Vec<String>>>,
    cancels: bool,
}

impl Spotlight {
    fn new(apps: FakeInstalledApps) -> Self {
        Self { apps, asked: Mutex::new(Vec::new()), cancels: false }
    }

    fn cancelling() -> Self {
        Self { cancels: true, ..Self::new(FakeInstalledApps::default()) }
    }

    fn asked(&self) -> Vec<Vec<String>> {
        self.asked.lock().unwrap().clone()
    }
}

impl InstalledApps for Spotlight {
    fn list(&self) -> Result<InstalledList, SystemError> {
        self.apps.list()
    }

    fn find(&self, ids: &[BundleId], cancel: &CancelToken) -> Result<Vec<PathBuf>, SystemError> {
        self.asked.lock().unwrap().push(ids.iter().map(|id| id.as_str().to_owned()).collect());
        if self.cancels {
            cancel.cancel();
        }
        self.apps.find(ids, cancel)
    }

    fn search_complete(&self, cancel: &CancelToken) -> Result<(), SystemError> {
        self.apps.search_complete(cancel)
    }
}

/// An entry named `name` of `kind` in the place of `interest`, of `bytes` bytes, read in full and unchanged for `days`.
fn entry(interest: usize, name: &str, kind: EntryKind, days: u64) -> Match {
    let path = Path::new("/home/Library").join(FOLDERS[interest]).join(name);
    let time = Some(Timestamp::from_secs(NOW - days * DAY));
    let bytes = ByteSize::new(100 + u64::try_from(interest).unwrap());
    Match {
        interest,
        path,
        kind,
        sizes: Sizes::complete(bytes, bytes),
        modified: time,
        changed: time,
        accessed: None,
        inside: (kind == EntryKind::Folder).then_some(Inside { newest_change: time, ..Inside::EMPTY }),
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

/// A folder named `name` in the place of `interest`, unchanged for 40 days.
fn folder(interest: usize, name: &str) -> Match {
    entry(interest, name, EntryKind::Folder, 40)
}

/// A file named `name` in the place of `interest`, unchanged for 40 days.
fn file(interest: usize, name: &str) -> Match {
    entry(interest, name, EntryKind::File, 40)
}

/// The leftovers of `id` in every place.
fn every_form(id: &str) -> Vec<Match> {
    vec![
        folder(CACHES, id),
        folder(LOGS, id),
        folder(SAVED_STATE, &format!("{id}.savedState")),
        file(PREFERENCES, &format!("{id}.plist")),
        folder(HTTP, id),
        file(HTTP, &format!("{id}.binarycookies")),
        folder(5, id),
        file(COOKIES, &format!("{id}.binarycookies")),
        folder(SUPPORT, id),
    ]
}

/// The system and the walk a scan runs with: every place read.
struct Scan {
    apps: Arc<Spotlight>,
    running: Arc<FakeRunningApps>,
    places: Vec<PlaceSeen>,
}

impl Scan {
    fn new() -> Self {
        Self::with_apps(FakeInstalledApps::default())
    }

    fn with_apps(apps: FakeInstalledApps) -> Self {
        Self::with_spotlight(Spotlight::new(apps))
    }

    fn with_spotlight(spotlight: Spotlight) -> Self {
        let places = vec![PlaceSeen::Read; FOLDERS.len()];
        Self { apps: Arc::new(spotlight), running: Arc::new(FakeRunningApps::default()), places }
    }

    fn running(self, running: FakeRunningApps) -> Self {
        Self { running: Arc::new(running), ..self }
    }

    fn place(mut self, interest: usize, seen: PlaceSeen) -> Self {
        self.places[interest] = seen;
        self
    }

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(&self, matches: &[Match]) -> Recorded {
        let module = AppLeftoversModule::new().unwrap();
        let cancel = CancelToken::new();
        let mut out = Recorded::default();
        self.on_matches(&module, &cancel, matches, &mut out);
        self.with_context(&module, &cancel, |cx| module.finish(&cx, &mut out).unwrap());
        out
    }

    fn on_matches(&self, module: &AppLeftoversModule, cancel: &CancelToken, matches: &[Match], out: &mut Recorded) {
        for found in matches {
            self.with_context(module, cancel, |cx| module.on_match(found, &cx, out).unwrap());
        }
    }

    fn with_context<T>(
        &self,
        module: &AppLeftoversModule,
        cancel: &CancelToken,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::<Spotlight>::clone(&self.apps),
            running: Arc::<FakeRunningApps>::clone(&self.running),
        });
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings::default())
            .module(module.descriptor().id());
        call(
            ModuleContext::new(&clock, &commands, &system, &settings, cancel).complete_walk().with_places(&self.places),
        )
    }
}

fn app(id: &str) -> fresher_core::InstalledApp {
    fresher_testing::installed_app(id, "App", "1").unwrap()
}

mod negative {
    use fresher_core::{
        BundleId, CancelToken, EntryKind, KnownFolder, Match, Module as _, ModuleError, Pattern, PlaceSeen,
    };
    use fresher_testing::{FakeInstalledApps, FakeRunningApps, running_app};

    use super::{
        CACHES, COOKIES, GONE, OTHER, PREFERENCES, Recorded, SAVED_STATE, SUPPORT, Scan, Spotlight, app, entry,
        every_form, file, folder,
    };
    use crate::AppLeftoversModule;

    /// App containers neither prove an id nor are offered (plan 006, open question 1, option (b)): the module asks
    /// the walk for nothing under them, nor needs the permission that guards them.
    #[test]
    fn the_containers_are_no_place_of_the_module_and_need_no_permission() {
        let module = AppLeftoversModule::new().unwrap();

        let containers = module
            .interests()
            .iter()
            .filter(|interest| matches!(interest.pattern(), Pattern::Under(KnownFolder::Containers, _)))
            .count();
        assert_eq!(containers, 0);
        assert!(module.descriptor().categories().iter().all(|category| category.needs().is_empty()));
    }

    #[test]
    fn a_link_an_other_entry_the_wrong_kind_or_a_match_of_no_interest_is_not_claimed() {
        let proof = format!("{GONE}.savedState");
        for stray in [
            entry(SAVED_STATE, &proof, EntryKind::Symlink, 40),
            entry(SAVED_STATE, &proof, EntryKind::Other, 40),
            file(SAVED_STATE, &proof),
            Match { interest: 99, ..folder(SAVED_STATE, &proof) },
        ] {
            let recorded = Scan::new().run(&[stray, folder(CACHES, GONE)]);

            assert_eq!(recorded, Recorded::default());
        }
    }

    #[test]
    fn a_name_that_gives_no_id_is_not_claimed() {
        let recorded = Scan::new().run(&[
            folder(SAVED_STATE, "Notes.savedState"),
            file(COOKIES, "Cookies.binarycookies"),
            folder(CACHES, "Notes"),
        ]);

        assert_eq!(recorded, Recorded::default());
    }

    #[test]
    fn an_id_with_no_saved_state_is_not_claimed_and_spotlight_is_not_asked() {
        let scan = Scan::new().place(SAVED_STATE, PlaceSeen::Absent);
        let matches = [folder(CACHES, GONE), file(PREFERENCES, &format!("{GONE}.plist")), folder(SUPPORT, GONE)];

        assert_eq!(scan.run(&matches), Recorded::default());
        assert_eq!(scan.apps.asked(), Vec::<Vec<String>>::new());
    }

    #[test]
    fn version_like_folders_and_files_without_a_proof_are_never_offered() {
        let mut matches = Vec::new();
        for name in ["IntelliJIdea2019.3", "PyCharm2019.2", "Python3.11", "1.2.3"] {
            matches.extend([folder(CACHES, name), folder(SUPPORT, name), file(PREFERENCES, &format!("{name}.plist"))]);
        }

        assert_eq!(Scan::new().run(&matches), Recorded::default());
    }

    #[test]
    fn an_apple_id_is_never_claimed() {
        assert_eq!(Scan::new().run(&every_form("com.apple.Safari")), Recorded::default());
    }

    #[test]
    fn a_left_out_id_reports_every_leftover_in_its_rows_category_with_its_sizes() {
        let scan = Scan::new().running(FakeRunningApps::new(vec![running_app("Gone", &[GONE]).unwrap()]));

        let recorded = scan.run(&[
            folder(SAVED_STATE, &format!("{GONE}.savedState")),
            folder(SUPPORT, GONE),
            folder(CACHES, GONE),
        ]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("app-leftovers.data", "app-leftovers.left-out.running", 107),
                ("app-leftovers.caches", "app-leftovers.left-out.running", 100),
                ("app-leftovers.caches", "app-leftovers.left-out.running", 102),
            ]
        );
        assert_eq!(scan.apps.asked(), Vec::<Vec<String>>::new());
    }

    #[test]
    fn a_place_not_read_in_full_keeps_every_id() {
        let recorded = Scan::new().place(CACHES, PlaceSeen::Unread).run(&every_form(GONE));

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out.len(), 9);
        assert!(recorded.left_out().iter().all(|(_, reason, _)| *reason == "app-leftovers.left-out.scan-again"));
    }

    #[test]
    fn a_helper_whose_app_spotlight_finds_elsewhere_is_installed_elsewhere() {
        let helper = "com.example.App.Helper";
        let scan = Scan::with_apps(FakeInstalledApps::default().with_elsewhere(vec![app("com.example.App")]));

        let recorded = scan.run(&[folder(SAVED_STATE, &format!("{helper}.savedState")), folder(CACHES, helper)]);

        assert_eq!(recorded.findings, []);
        assert!(
            recorded.left_out().iter().all(|(_, reason, _)| *reason == "app-leftovers.left-out.installed-elsewhere")
        );
        assert_eq!(scan.apps.asked(), [vec![helper, "com.example.App", "com.example"]]);
    }

    #[test]
    fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
        let scan = Scan::new();
        let module = AppLeftoversModule::new().unwrap();
        let mut out = Recorded::default();
        scan.on_matches(&module, &CancelToken::new(), &every_form(GONE), &mut out);

        let next = CancelToken::new();
        scan.with_context(&module, &next, |cx| module.finish(&cx, &mut out).unwrap());

        assert_eq!(out, Recorded::default());
    }

    #[test]
    fn a_cancel_during_finish_ends_cancelled() {
        let scan = Scan::new();
        let module = AppLeftoversModule::new().unwrap();
        let (cancel, mut out) = (CancelToken::new(), Recorded::default());
        scan.on_matches(&module, &cancel, &every_form(GONE), &mut out);
        cancel.cancel();

        let result = scan.with_context(&module, &cancel, |cx| module.finish(&cx, &mut out));

        assert_eq!(result, Err(ModuleError::Cancelled));
        assert_eq!(out, Recorded::default());
    }

    #[test]
    fn a_cancel_between_ids_ends_cancelled_before_the_next_id_is_decided() {
        let scan = Scan::with_spotlight(Spotlight::cancelling());
        let module = AppLeftoversModule::new().unwrap();
        let (cancel, mut out) = (CancelToken::new(), Recorded::default());
        let mut matches = every_form(GONE);
        matches.extend(every_form(OTHER));
        scan.on_matches(&module, &cancel, &matches, &mut out);

        let result = scan.with_context(&module, &cancel, |cx| module.finish(&cx, &mut out));

        assert_eq!(result, Err(ModuleError::Cancelled));
        assert_eq!(scan.apps.asked(), [vec![GONE, "com.example"]]);
        let apps: Vec<&str> = out.findings.iter().filter_map(|finding| finding.app().map(BundleId::as_str)).collect();
        assert_eq!(apps, [GONE; 9]);
        assert_eq!(out.left_out, []);
    }
}

mod positive {
    use fresher_core::{BundleId, Inside, Match, Module as _, ModuleGroup, Removal, Subject, TextArg};

    use super::{CACHES, GONE, OTHER, SAVED_STATE, Scan, every_form, folder};
    use crate::AppLeftoversModule;

    #[test]
    fn the_module_is_in_the_group_apps_with_three_categories_nine_problems_and_two_choices() {
        let module = AppLeftoversModule::new().unwrap();
        let descriptor = module.descriptor();

        assert_eq!(descriptor.id().as_str(), "app-leftovers");
        assert_eq!(descriptor.group(), ModuleGroup::Apps);
        let categories: Vec<&str> = descriptor.categories().iter().map(|category| category.id().as_str()).collect();
        assert_eq!(categories, ["app-leftovers.caches", "app-leftovers.settings", "app-leftovers.data"]);
        assert_eq!(descriptor.problems().len(), 9);
        let choices: Vec<(&str, Vec<&str>)> = descriptor
            .choices()
            .iter()
            .map(|choice| (choice.name(), choice.categories().iter().map(|id| id.as_str()).collect()))
            .collect();
        assert_eq!(choices, [("everything", categories), ("keep-settings", vec!["app-leftovers.caches"])]);
        assert_eq!(module.interests().len(), 8);
    }

    #[test]
    fn every_form_of_an_app_that_is_gone_is_offered_to_the_trash_for_its_id() {
        let scan = Scan::new();

        let recorded = scan.run(&every_form(GONE));

        assert_eq!(recorded.left_out, []);
        assert_eq!(
            recorded.identities(),
            [
                "support/com.example.Gone",
                "caches/com.example.Gone",
                "cookies/com.example.Gone.binarycookies",
                "http-storage/com.example.Gone",
                "http-cookies/com.example.Gone.binarycookies",
                "logs/com.example.Gone",
                "preferences/com.example.Gone.plist",
                "saved-state/com.example.Gone.savedState",
                "web-data/com.example.Gone",
            ]
        );
        let categories: Vec<&str> = recorded.findings.iter().map(|finding| finding.category().as_str()).collect();
        assert_eq!(categories.iter().filter(|category| **category == "app-leftovers.caches").count(), 3);
        assert_eq!(categories.iter().filter(|category| **category == "app-leftovers.settings").count(), 5);
        assert_eq!(categories.iter().filter(|category| **category == "app-leftovers.data").count(), 1);
        for finding in &recorded.findings {
            assert!(matches!(
                (finding.subject(), finding.removal()),
                (Subject::Path(subject), Removal::MoveToTrash(removed)) if subject == removed
            ));
            assert_eq!(finding.app().map(BundleId::as_str), Some(GONE));
            assert_eq!(finding.title().args().get("id"), Some(&TextArg::Text(GONE.into())));
            assert_eq!(finding.reason().args().get("days"), Some(&TextArg::Count(30)));
        }
        assert_eq!(scan.apps.asked(), [vec![GONE, "com.example"]]);
    }

    #[test]
    fn the_same_matches_in_two_orders_give_the_same_answers() {
        let mut matches = every_form(GONE);
        matches.extend(every_form(OTHER));
        let forward = Scan::new().run(&matches);
        matches.reverse();

        assert_eq!(Scan::new().run(&matches), forward);
        assert_eq!(forward.findings.len(), 18);
    }

    #[test]
    fn spotlight_is_asked_once_per_id_that_reaches_it() {
        let scan = Scan::new();
        let mut matches = every_form(GONE);
        matches.extend(every_form(OTHER));
        matches.push(folder(CACHES, "com.example.Unproven"));

        scan.run(&matches);

        let first: Vec<String> = scan.apps.asked().into_iter().map(|ids| ids[0].clone()).collect();
        assert_eq!(first, [GONE, OTHER]);
    }

    #[test]
    fn an_item_with_a_socket_inside_is_left_out_while_the_others_are_offered() {
        let cache = folder(CACHES, GONE);
        let socket = Match { inside: cache.inside.map(|inside| Inside { special: true, ..inside }), ..cache };

        let recorded = Scan::new().run(&[folder(SAVED_STATE, &format!("{GONE}.savedState")), socket]);

        assert_eq!(recorded.identities(), ["saved-state/com.example.Gone.savedState"]);
        assert_eq!(recorded.left_out(), [("app-leftovers.caches", "app-leftovers.left-out.special", 100)]);
    }
}
