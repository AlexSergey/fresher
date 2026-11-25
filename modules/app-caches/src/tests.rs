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
use crate::gathered::Source;

const NOW: u64 = 1_767_225_600;
const EDITOR: &str = "com.example.Editor";
const HELPER: &str = "com.example.Editor.Helper";
const VIEWER: &str = "com.example.Viewer";
const CHROME: &str = "com.google.Chrome";
const REDIS_INSIGHT: &str = "org.RedisLabs.RedisInsight-V2";

/// The interest of the shipped places table's one row.
const CACHES: usize = 0;
/// The interest of a cache of a row of the shipped apps table: Chrome is row 0, Chromium row 1, Redis Insight row 3.
fn app_interest(row: usize, cache: usize) -> usize {
    let module = AppCachesModule::new().unwrap();
    module.sources.iter().position(|source| *source == Source::App { row, cache }).unwrap()
}

fn chrome_cache() -> usize {
    app_interest(0, 0)
}

fn chrome_code_cache() -> usize {
    app_interest(0, 1)
}

fn chromium_cache() -> usize {
    app_interest(1, 0)
}

fn redis_insight_gpu_cache() -> usize {
    app_interest(3, 2)
}

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

/// Editor with its helper, a viewer, Google Chrome with its helper, and Redis Insight; Chromium is not installed.
fn installed() -> Vec<InstalledApp> {
    vec![
        app(EDITOR, "Editor", &[HELPER]),
        app(VIEWER, "Viewer", &[]),
        app(CHROME, "Google Chrome", &["com.google.Chrome.helper"]),
        app(REDIS_INSIGHT, "Redis Insight", &[]),
    ]
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

/// A folder of the user's at `place` in `/home/Library`, matched by `interest`, read in full, of `bytes` logical bytes.
fn table_cache(interest: usize, place: &str, bytes: u64) -> Match {
    Match { interest, path: Path::new("/home/Library").join(place), ..cache(EDITOR, bytes) }
}

/// Caches of the apps table: two profiles of Chrome, a socket in a third, Chromium (not installed), Redis Insight.
fn browsers() -> Vec<Match> {
    vec![
        table_cache(chrome_cache(), "Caches/Google/Chrome/Default/Cache", 4000),
        table_cache(chrome_code_cache(), "Caches/Google/Chrome/Profile 1/Code Cache", 5000),
        Match {
            inside: Some(Inside { special: true, ..Inside::EMPTY }),
            ..table_cache(chrome_code_cache(), "Caches/Google/Chrome/Guest Profile/Code Cache", 80)
        },
        table_cache(chromium_cache(), "Caches/Chromium/Default/Cache", 6000),
        table_cache(redis_insight_gpu_cache(), "Application Support/RedisInsight/GPUCache", 7000),
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

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(&self, matches: &[Match]) -> Recorded {
        let (module, cancel) = (AppCachesModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        for found in matches {
            self.with_context(&module, &cancel, |cx| module.on_match(found, &cx, &mut out).unwrap());
        }
        self.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out).unwrap());
        out
    }

    fn with_context<T>(
        &self,
        module: &AppCachesModule,
        cancel: &CancelToken,
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
        call(ModuleContext::new(&clock, &commands, &system, &settings, cancel))
    }
}

fn running(name: &str, ids: &[&str]) -> fresher_core::RunningApp {
    fresher_testing::running_app(name, ids).unwrap()
}

mod negative {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt as _;
    use std::path::Path;

    use fresher_core::{CancelToken, Match, Module as _, ModuleError, Removal, Subject};
    use fresher_testing::{FakeInstalledApps, FakeRunningApps};

    use super::{
        CHROME, EDITOR, HELPER, Recorded, Scan, browsers, cache, chrome_cache, mixed, redis_insight_gpu_cache, running,
        table_cache,
    };
    use crate::AppCachesModule;

    #[test]
    fn a_match_of_no_interest_is_not_claimed() {
        let past_the_last = AppCachesModule::new().unwrap().interests().len();
        for interest in [past_the_last, 99] {
            let stray = Match { interest, ..cache(EDITOR, 1000) };

            assert_eq!(Scan::new().run(&[stray]), Recorded::default(), "{interest}");
        }
    }

    #[test]
    fn a_profile_s_finding_never_names_another_path() {
        let matches = browsers();

        let recorded = Scan::new().run(&matches);

        assert_eq!(recorded.findings.len(), 3);
        for finding in &recorded.findings {
            let Subject::Path(path) = finding.subject() else { unreachable!("{finding:?}") };
            assert!(matches.iter().any(|found| &found.path == path), "{path:?}");
            assert_eq!(finding.removal(), &Removal::EmptyFolder(path.clone()));
        }
    }

    #[test]
    fn an_open_browser_s_caches_are_left_out_in_its_category() {
        let scan = Scan::running(FakeRunningApps::new(vec![running("Google Chrome", &[CHROME])]));

        let recorded = scan.run(&browsers());

        assert_eq!(recorded.identities(), ["redis-insight/GPUCache"]);
        assert_eq!(
            recorded.left_out(),
            [
                ("app-caches.browsers", "app-caches.left-out.app-open", 4000),
                ("app-caches.browsers", "app-caches.left-out.special", 80),
                ("app-caches.browsers", "app-caches.left-out.app-open", 5000),
            ]
        );
    }

    #[test]
    fn a_cache_whose_profile_or_name_is_not_in_utf8_gets_no_finding() {
        let profile = Path::new("Caches/Google/Chrome").join(OsStr::from_bytes(b"Profile \xff")).join("Cache");
        let name = Path::new("Application Support/RedisInsight").join(OsStr::from_bytes(b"GPUCache\xff"));
        let odd = [
            Match { path: Path::new("/home/Library").join(profile), ..table_cache(chrome_cache(), "", 4000) },
            Match { path: Path::new("/home/Library").join(name), ..table_cache(redis_insight_gpu_cache(), "", 10) },
        ];

        assert_eq!(Scan::new().run(&odd), Recorded::default());
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
            scan.with_context(&module, &cancel, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        cancel.cancel();
        let finished = scan.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out, Recorded::default());
    }

    #[test]
    fn a_cancelled_scan_stops_between_candidates() {
        let (scan, module, cancel) = (Scan::new(), AppCachesModule::new().unwrap(), CancelToken::new());
        let mut out = CancelsAtFirstFinding { cancel: cancel.clone(), recorded: Recorded::default() };
        for found in mixed() {
            scan.with_context(&module, &cancel, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        let finished = scan.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out.recorded.identities(), ["library-caches/com.example.Editor"]);
        assert_eq!(out.recorded.left_out, []);
    }
}

mod positive {
    use fresher_core::{BundleId, ByteSize, DangerLevel, Module as _, ModuleGroup, Removal, Subject, TextArg, TextKey};
    use fresher_testing::FakeRunningApps;

    use super::{
        CHROME, EDITOR, HELPER, REDIS_INSIGHT, Scan, browsers, cache, chrome_code_cache, mixed,
        redis_insight_gpu_cache, running, table_cache,
    };
    use crate::AppCachesModule;
    use crate::gathered::Source;

    #[test]
    fn a_browser_cache_is_titled_by_its_browser_profile_and_folder() {
        let found = table_cache(chrome_code_cache(), "Caches/Google/Chrome/Profile 1/Code Cache", 5000);

        let recorded = Scan::new().run(&[found]);

        let finding = &recorded.findings[0];
        assert_eq!(finding.title().key().as_str(), "app-caches.title.profile");
        assert_eq!(finding.title().args().get("app"), Some(&TextArg::Text("Google Chrome".into())));
        assert_eq!(finding.title().args().get("profile"), Some(&TextArg::Text("Profile 1".into())));
        assert_eq!(finding.title().args().get("folder"), Some(&TextArg::Text("Code Cache".into())));
        assert_eq!(finding.reason().key().as_str(), "app-caches.reason");
        assert_eq!(finding.reason().args().get("app"), Some(&TextArg::Text("Google Chrome".into())));
        assert_eq!(finding.reason().args().get("size"), Some(&TextArg::Size(ByteSize::new(5000))));
        assert_eq!(finding.category().as_str(), "app-caches.browsers");
        assert_eq!(finding.danger(), DangerLevel::Low);
    }

    #[test]
    fn a_web_engine_cache_is_titled_by_its_app_and_folder() {
        let found = table_cache(redis_insight_gpu_cache(), "Application Support/RedisInsight/GPUCache", 7000);

        let recorded = Scan::new().run(&[found]);

        let finding = &recorded.findings[0];
        assert_eq!(finding.title().key().as_str(), "app-caches.title.part");
        assert_eq!(finding.title().args().get("app"), Some(&TextArg::Text("Redis Insight".into())));
        assert_eq!(finding.title().args().get("folder"), Some(&TextArg::Text("GPUCache".into())));
        assert_eq!(finding.title().args().get("profile"), None);
        assert_eq!(finding.reason().args().get("size"), Some(&TextArg::Size(ByteSize::new(7000))));
        assert_eq!(finding.category().as_str(), "app-caches.web-engine");
        assert_eq!(finding.danger(), DangerLevel::Low);
    }

    #[test]
    fn a_table_finding_empties_its_folder_and_is_counted_in_the_app_s_line() {
        let matches = browsers();

        let recorded = Scan::new().run(&matches);

        for finding in &recorded.findings {
            let Subject::Path(path) = finding.subject() else { unreachable!("{finding:?}") };
            assert_eq!(finding.removal(), &Removal::EmptyFolder(path.clone()));
            let found = matches.iter().find(|found| &found.path == path).unwrap();
            assert_eq!(finding.sizes(), found.sizes);
            assert!(finding.app_name().is_none());
        }
        let apps: Vec<Option<&str>> =
            recorded.findings.iter().map(|finding| finding.app().map(BundleId::as_str)).collect();
        assert_eq!(apps, [Some(REDIS_INSIGHT), Some(CHROME), Some(CHROME)]);
    }

    #[test]
    fn table_identities_name_the_row_the_profile_and_the_cache() {
        let recorded = Scan::new().run(&browsers());

        assert_eq!(
            recorded.identities(),
            ["redis-insight/GPUCache", "chrome/Default/Cache", "chrome/Profile 1/Code Cache"]
        );
    }

    #[test]
    fn the_library_caches_row_reports_as_before() {
        let scan = Scan::running(FakeRunningApps::new(vec![running("Viewer", &["com.example.Viewer"])]));
        let both: Vec<_> = mixed().into_iter().chain(browsers()).collect();

        let alone = scan.run(&mixed());
        let together = scan.run(&both);

        let library = |category: &str| category == "app-caches.library-caches";
        let findings: Vec<_> =
            together.findings.iter().filter(|finding| library(finding.category().as_str())).cloned().collect();
        let left_out: Vec<_> = together.left_out().into_iter().filter(|(category, _, _)| library(category)).collect();
        assert_eq!(findings, alone.findings);
        assert_eq!(left_out, alone.left_out());
    }

    #[test]
    fn the_module_is_in_the_system_group_with_no_setting() {
        let module = AppCachesModule::new().unwrap();

        assert_eq!(module.descriptor().id().as_str(), "app-caches");
        assert_eq!(module.descriptor().group(), ModuleGroup::System);
        assert_eq!(module.descriptor().settings(), []);
    }

    #[test]
    fn the_three_categories_are_listed_in_order_at_low_danger() {
        let module = AppCachesModule::new().unwrap();

        let categories: Vec<(&str, DangerLevel)> = module
            .descriptor()
            .categories()
            .iter()
            .map(|category| (category.id().as_str(), category.danger()))
            .collect();
        assert_eq!(
            categories,
            [
                ("app-caches.library-caches", DangerLevel::Low),
                ("app-caches.browsers", DangerLevel::Low),
                ("app-caches.web-engine", DangerLevel::Low),
            ]
        );
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
    fn the_interests_are_the_shipped_place_then_one_per_cache_of_the_apps_table() {
        let module = AppCachesModule::new().unwrap();

        assert_eq!(module.interests().len(), 12);
        assert!(module.interests()[0].matches_only_bundle_ids());
        assert!(module.interests()[1..].iter().all(|interest| !interest.matches_only_bundle_ids()));
        assert_eq!(module.sources[0], Source::Place { row: 0 });
        assert_eq!(module.sources[1], Source::App { row: 0, cache: 0 });
        assert_eq!(module.sources[11], Source::App { row: 3, cache: 4 });
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
    use proptest::prelude::*;

    use super::{Scan, browsers, mixed};

    /// Some of `mixed()` and `browsers()`, in any order.
    fn any_matches() -> impl Strategy<Value = Vec<fresher_core::Match>> {
        let all: Vec<_> = mixed().into_iter().chain(browsers()).collect();
        let count = all.len();
        proptest::sample::subsequence(all, 0..=count).prop_shuffle()
    }

    proptest! {
        #[test]
        fn the_verdict_does_not_depend_on_the_order_of_matches(matches in any_matches()) {
            let scan = Scan::new();
            let mut sorted = matches.clone();
            sorted.sort_by(|first, second| first.path.cmp(&second.path));

            prop_assert_eq!(scan.run(&matches), scan.run(&sorted));
        }
    }
}
