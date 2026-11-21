//! The module as built, and the module through `on_match` and `finish`, on a recording sink, with every system fact
//! faked and plain `Match` values in a home at `/home`.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    BundleId, ByteSize, CancelToken, EntryFacts, EntryKind, Inside, InstalledApp, LeftOutItem, Match, Module as _,
    ModuleContext, NewFinding, SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::AppCachesModule;

const NOW: u64 = 1_767_225_600;
const EDITOR: &str = "com.example.Editor";
const HELPER: &str = "com.example.Editor.Helper";
const VIEWER: &str = "com.example.Viewer";

/// The interest of the shipped table's one row.
const CACHES: usize = 0;

/// Keeps everything the module reports, in order.
#[derive(Debug, Default, PartialEq, Eq)]
struct Recorded {
    findings: Vec<NewFinding>,
    left_out: Vec<LeftOutItem>,
}

impl Recorded {
    /// Each left-out as its category, reason and logical bytes.
    fn left_out(&self) -> Vec<(&str, &str, u64)> {
        self.left_out
            .iter()
            .map(|item| (item.category.as_str(), item.reason.as_str(), item.sizes.logical.bytes()))
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

/// An installed app of `id` named `name`, with the helpers `helpers`.
fn app(id: &str, name: &str, helpers: &[&str]) -> InstalledApp {
    let app = fresher_testing::installed_app(id, name, "1.0").unwrap();
    InstalledApp { helpers: helpers.iter().map(|helper| BundleId::new(*helper).unwrap()).collect(), ..app }
}

/// Editor with its helper, and a viewer.
fn installed() -> Vec<InstalledApp> {
    vec![app(EDITOR, "Editor", &[HELPER]), app(VIEWER, "Viewer", &[])]
}

/// A folder of the user's named `name` in `/home/Library/Caches`, read in full, of `bytes` logical bytes.
fn cache(name: &str, bytes: u64) -> Match {
    Match {
        interest: CACHES,
        path: Path::new("/home/Library/Caches").join(name),
        kind: EntryKind::Folder,
        sizes: Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes + 4096)),
        modified: None,
        changed: None,
        accessed: None,
        inside: Some(Inside::EMPTY),
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

/// A cache of every verdict, and entries not claimed.
fn mixed() -> Vec<Match> {
    vec![
        cache(EDITOR, 1000),
        cache(HELPER, 2000),
        cache("com.example.Viewer.Previews", 3000),
        Match { inside: Some(Inside { special: true, ..Inside::EMPTY }), ..cache("com.example.Editor.Sockets", 10) },
        Match { facts: EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE }, ..cache(VIEWER, 20) },
        cache("com.example.Gone", 30),
        cache("com.apple.Safari", 40),
        cache("com.example", 50),
        cache("Notes", 60),
        cache("com.example.Editor.Empty", 0),
        Match { kind: EntryKind::Symlink, ..cache("com.example.Editor.Link", 70) },
    ]
}

/// The system a scan runs with.
struct Scan {
    apps: Arc<FakeInstalledApps>,
    running: Arc<FakeRunningApps>,
}

impl Scan {
    fn new() -> Self {
        Self::with(FakeInstalledApps::new(installed()), FakeRunningApps::default())
    }

    fn running(running: FakeRunningApps) -> Self {
        Self::with(FakeInstalledApps::new(installed()), running)
    }

    fn with(apps: FakeInstalledApps, running: FakeRunningApps) -> Self {
        Self { apps: Arc::new(apps), running: Arc::new(running) }
    }

    fn run(&self, matches: &[Match]) -> Recorded {
        self.run_as(matches, false)
    }

    /// Hands `matches` to a new module in their order, then calls `finish`; every context restored when `restored`.
    fn run_as(&self, matches: &[Match], restored: bool) -> Recorded {
        let (module, cancel) = (AppCachesModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        for found in matches {
            self.with_context(&module, &cancel, restored, |cx| module.on_match(found, &cx, &mut out).unwrap());
        }
        self.with_context(&module, &cancel, restored, |cx| module.finish(&cx.complete_walk(), &mut out).unwrap());
        out
    }

    fn with_context<T>(
        &self,
        module: &AppCachesModule,
        cancel: &CancelToken,
        restored: bool,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::<FakeInstalledApps>::clone(&self.apps),
            running: Arc::<FakeRunningApps>::clone(&self.running),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::new(fresher_testing::NoFiles),
        });
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings::default())
            .module(module.descriptor().id());
        let cx = ModuleContext::new(&clock, &commands, &system, &settings, cancel);
        call(if restored { cx.restored() } else { cx })
    }
}

fn running(name: &str, ids: &[&str]) -> fresher_core::RunningApp {
    fresher_testing::running_app(name, ids).unwrap()
}

mod negative {
    use fresher_core::{CancelToken, Match, Module as _, ModuleError};
    use fresher_testing::{FakeInstalledApps, FakeRunningApps};

    use super::{EDITOR, HELPER, Recorded, Scan, cache, mixed, running};
    use crate::AppCachesModule;

    #[test]
    fn a_match_of_no_interest_is_not_claimed() {
        let stray = Match { interest: 99, ..cache(EDITOR, 1000) };

        assert_eq!(Scan::new().run(&[stray]), Recorded::default());
    }

    #[test]
    fn nothing_left_out_is_also_offered() {
        let scan = Scan::running(FakeRunningApps::new(vec![running("Viewer", &["com.example.Viewer"])]));
        for found in mixed() {
            let recorded = scan.run(std::slice::from_ref(&found));

            assert!(recorded.findings.len() + recorded.left_out.len() <= 1, "{found:?}: {recorded:?}");
        }

        let all = scan.run(&mixed());

        assert_eq!(all.identities(), ["library-caches/com.example.Editor", "library-caches/com.example.Editor.Helper"]);
        assert_eq!(
            all.left_out(),
            [
                ("app-caches.library-caches", "app-caches.left-out.special", 10),
                ("app-caches.library-caches", "app-caches.left-out.not-yours", 20),
                ("app-caches.library-caches", "app-caches.left-out.app-open", 3000),
            ]
        );
    }

    #[test]
    fn nothing_is_asked_or_said_when_nothing_was_gathered() {
        let scan = Scan::new();

        assert_eq!(scan.run(&[]), Recorded::default());
        assert_eq!((scan.apps.calls(), scan.running.calls()), (0, 0));
    }

    #[test]
    fn nothing_is_said_when_the_installed_apps_cannot_be_listed() {
        let scan = Scan::with(FakeInstalledApps::failing("no apps folder"), FakeRunningApps::default());

        assert_eq!(scan.run(&mixed()), Recorded::default());
        assert_eq!((scan.apps.calls(), scan.running.calls()), (1, 0));
    }

    #[test]
    fn the_running_apps_are_not_asked_when_no_cache_reaches_the_question() {
        let scan = Scan::new();

        let recorded = scan.run(&[cache("com.example.Gone", 10), cache("com.apple.Safari", 20), cache(EDITOR, 0)]);

        assert_eq!(recorded, Recorded::default());
        assert_eq!((scan.apps.calls(), scan.running.calls()), (1, 0));
    }

    #[test]
    fn every_cache_is_left_out_no_system_when_the_running_apps_cannot_be_listed() {
        let scan = Scan::running(FakeRunningApps::failing("no process list"));

        let recorded = scan.run(&[cache(EDITOR, 1000), cache(HELPER, 2000)]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("app-caches.library-caches", "app-caches.left-out.no-system", 1000),
                ("app-caches.library-caches", "app-caches.left-out.no-system", 2000),
            ]
        );
        assert_eq!(scan.running.calls(), 1);
    }

    /// Records like `Recorded`, and cancels the scan once the first finding is pushed.
    struct CancelsAtFirstFinding {
        cancel: CancelToken,
        recorded: Recorded,
    }

    impl fresher_core::FindingSink for CancelsAtFirstFinding {
        fn push(&mut self, finding: fresher_core::NewFinding) {
            self.recorded.findings.push(finding);
            self.cancel.cancel();
        }

        fn left_out(&mut self, left_out: fresher_core::LeftOutItem) {
            self.recorded.left_out.push(left_out);
        }
    }

    #[test]
    fn a_scan_cancelled_before_finish_reports_nothing() {
        let (scan, module, cancel) = (Scan::new(), AppCachesModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        for found in mixed() {
            scan.with_context(&module, &cancel, false, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        cancel.cancel();
        let finished = scan.with_context(&module, &cancel, false, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out, Recorded::default());
    }

    #[test]
    fn a_cancelled_scan_stops_between_candidates() {
        let (scan, module, cancel) = (Scan::new(), AppCachesModule::new().unwrap(), CancelToken::new());
        let mut out = CancelsAtFirstFinding { cancel: cancel.clone(), recorded: Recorded::default() };
        for found in mixed() {
            scan.with_context(&module, &cancel, false, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        let finished = scan.with_context(&module, &cancel, false, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out.recorded.identities(), ["library-caches/com.example.Editor"]);
        assert_eq!(out.recorded.left_out, []);
    }
}

mod positive {
    use fresher_core::{BundleId, ByteSize, DangerLevel, Module as _, ModuleGroup, Removal, Subject, TextArg, TextKey};
    use fresher_testing::FakeRunningApps;

    use super::{EDITOR, HELPER, Scan, cache, mixed, running};
    use crate::AppCachesModule;

    #[test]
    fn the_module_is_in_the_system_group_with_no_setting() {
        let module = AppCachesModule::new().unwrap();

        assert_eq!(module.descriptor().id().as_str(), "app-caches");
        assert_eq!(module.descriptor().group(), ModuleGroup::System);
        assert_eq!(module.descriptor().settings(), []);
    }

    #[test]
    fn the_only_category_is_the_library_caches_at_low_danger() {
        let module = AppCachesModule::new().unwrap();

        let categories = module.descriptor().categories();
        assert_eq!(categories.len(), 1);
        assert_eq!(categories[0].id().as_str(), "app-caches.library-caches");
        assert_eq!(categories[0].danger(), DangerLevel::Low);
    }

    #[test]
    fn the_four_left_out_reasons_are_declared_in_order() {
        let module = AppCachesModule::new().unwrap();

        let keys: Vec<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();
        assert_eq!(
            keys,
            [
                "app-caches.left-out.special",
                "app-caches.left-out.not-yours",
                "app-caches.left-out.no-system",
                "app-caches.left-out.app-open",
            ]
        );
    }

    #[test]
    fn the_one_interest_is_the_shipped_places_whole_claim() {
        let module = AppCachesModule::new().unwrap();

        assert_eq!(module.interests().len(), 1);
        assert!(module.interests()[0].matches_only_bundle_ids());
    }

    #[test]
    fn a_finding_names_the_app_and_the_size() {
        let recorded = Scan::new().run(&[cache(EDITOR, 1000), cache(HELPER, 2000)]);

        let (own, part) = (&recorded.findings[0], &recorded.findings[1]);
        assert_eq!(own.title().key().as_str(), "app-caches.title.own");
        assert_eq!(own.title().args().get("app"), Some(&TextArg::Text("Editor".into())));
        assert_eq!(own.title().args().get("folder"), None);
        assert_eq!(part.title().key().as_str(), "app-caches.title.part");
        assert_eq!(part.title().args().get("app"), Some(&TextArg::Text("Editor".into())));
        assert_eq!(part.title().args().get("folder"), Some(&TextArg::Text(HELPER.into())));
        for (finding, bytes) in [(own, 1000), (part, 2000)] {
            assert_eq!(finding.reason().key().as_str(), "app-caches.reason");
            assert_eq!(finding.reason().args().get("app"), Some(&TextArg::Text("Editor".into())));
            assert_eq!(finding.reason().args().get("size"), Some(&TextArg::Size(ByteSize::new(bytes))));
            assert_eq!(finding.sizes(), cache(EDITOR, bytes).sizes);
            assert_eq!(finding.category().as_str(), "app-caches.library-caches");
            assert_eq!(finding.danger(), DangerLevel::Low);
        }
    }

    #[test]
    fn a_finding_empties_the_folder() {
        let found = cache(EDITOR, 1000);

        let recorded = Scan::new().run(std::slice::from_ref(&found));

        let finding = &recorded.findings[0];
        assert_eq!(finding.key().identity(), "library-caches/com.example.Editor");
        assert_eq!(finding.subject(), &Subject::Path(found.path.clone()));
        assert_eq!(finding.removal(), &Removal::EmptyFolder(found.path));
    }

    #[test]
    fn a_finding_is_counted_in_its_app_s_line() {
        let recorded = Scan::new().run(&[cache(HELPER, 2000), cache("com.example.Viewer.Previews", 3000)]);

        let apps: Vec<Option<&str>> =
            recorded.findings.iter().map(|finding| finding.app().map(BundleId::as_str)).collect();
        assert_eq!(apps, [Some(EDITOR), Some("com.example.Viewer")]);
        assert!(recorded.findings.iter().all(|finding| finding.app_name().is_none()));
    }

    #[test]
    fn the_running_apps_are_asked_once_a_scan() {
        let scan = Scan::running(FakeRunningApps::new(vec![running("Mail", &["com.example.Mail"])]));

        let recorded = scan.run(&mixed());

        assert_eq!(recorded.findings.len(), 3);
        assert_eq!((scan.apps.calls(), scan.running.calls()), (1, 1));
    }
}

mod property {
    use fresher_testing::FakeRunningApps;
    use proptest::prelude::*;

    use super::{Scan, mixed, running};

    /// Some of `mixed()`, in any order.
    fn any_matches() -> impl Strategy<Value = Vec<fresher_core::Match>> {
        proptest::sample::subsequence(mixed(), 0..=mixed().len()).prop_shuffle()
    }

    proptest! {
        #[test]
        fn a_restored_context_decides_the_same(matches in any_matches()) {
            let scan = Scan::running(FakeRunningApps::new(vec![running("Viewer", &["com.example.Viewer"])]));

            prop_assert_eq!(scan.run_as(&matches, true), scan.run_as(&matches, false));
        }

        #[test]
        fn the_verdict_does_not_depend_on_the_order_of_matches(matches in any_matches()) {
            let scan = Scan::new();
            let mut sorted = matches.clone();
            sorted.sort_by(|first, second| first.path.cmp(&second.path));

            prop_assert_eq!(scan.run(&matches), scan.run(&sorted));
        }
    }
}
