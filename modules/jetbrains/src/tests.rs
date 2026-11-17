//! The module through `on_match` and `finish`, on a recording sink.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, EntryFacts, EntryKind, InstalledApp, LeftOutItem, Match, Module, ModuleContext,
    ModuleSettings, NewFinding, RunningApp, SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts, Timestamp,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::JetBrainsModule;

const NOW: u64 = 1_767_225_600;
const DAY: u64 = 24 * 60 * 60;

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
}

impl fresher_core::FindingSink for Recorded {
    fn push(&mut self, finding: NewFinding) {
        self.findings.push(finding);
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        self.left_out.push(left_out);
    }
}

/// A match of the module's `interest` (0 settings, 1 caches, 2 logs) at `path`, of `bytes`, changed `days` ago.
fn matched(interest: usize, path: &str, kind: EntryKind, bytes: u64) -> Match {
    let bytes = ByteSize::new(bytes);
    Match {
        interest,
        path: path.into(),
        kind,
        sizes: Sizes::complete(bytes, bytes),
        modified: Some(Timestamp::from_secs(NOW - 40 * DAY)),
        siblings: Vec::new(),
        marker: None,
        facts: EntryFacts::default(),
    }
}

/// A folder of the module's `interest`, changed 40 days ago.
fn folder(interest: usize, path: &str) -> Match {
    matched(interest, path, EntryKind::Folder, 100)
}

/// A folder of the module's `interest`, changed `days` ago (`None`: a time the walk could not read).
fn aged(interest: usize, path: &str, days: Option<u64>) -> Match {
    Match { modified: days.map(|days| Timestamp::from_secs(NOW - days * DAY)), ..folder(interest, path) }
}

/// The system and settings a scan runs with.
struct Scan {
    installed: Arc<FakeInstalledApps>,
    running: Vec<RunningApp>,
    /// The settings file's values, as JSON.
    stored: &'static str,
    /// Whether the scan is restored from the cache.
    restored: bool,
    /// Whether the walk read every folder.
    walk_complete: bool,
}

impl Scan {
    fn new(installed: Vec<InstalledApp>, running: Vec<RunningApp>) -> Self {
        Self {
            installed: Arc::new(FakeInstalledApps::new(installed)),
            running,
            stored: "{}",
            restored: false,
            walk_complete: true,
        }
    }

    /// Hands `matches` to `module` in their order with `cancel`, the scan's token, without `finish`.
    fn walk(&self, module: &JetBrainsModule, matches: &[Match], cancel: &CancelToken) -> Recorded {
        let mut out = Recorded::default();
        self.with_context(module, cancel, |cx| {
            for found in matches {
                module.on_match(found, cx, &mut out).unwrap();
            }
        });
        out
    }

    /// `walk`, then `finish`.
    fn run_on(&self, module: &JetBrainsModule, matches: &[Match], cancel: &CancelToken) -> Recorded {
        let mut out = self.walk(module, matches, cancel);
        self.with_context(module, cancel, |cx| module.finish(cx, &mut out).unwrap());
        out
    }

    fn run(self, matches: &[Match]) -> Recorded {
        self.run_on(&JetBrainsModule::new().unwrap(), matches, &CancelToken::new())
    }

    fn with_context(&self, module: &JetBrainsModule, cancel: &CancelToken, call: impl FnOnce(&ModuleContext<'_>)) {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::clone(&self.installed) as Arc<dyn fresher_core::InstalledApps>,
            running: Arc::new(FakeRunningApps::new(self.running.clone())),
        });
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let settings = settings(module, self.stored);
        let cx = ModuleContext::new(&clock, &commands, &system, &settings, cancel);
        let cx = if self.walk_complete { cx.complete_walk() } else { cx };
        call(&if self.restored { cx.restored() } else { cx });
    }
}

fn settings(module: &JetBrainsModule, stored: &str) -> ModuleSettings {
    let stored: StoredSettings = serde_json::from_str(stored).unwrap();
    SettingsSchema::new([module.descriptor()]).unwrap().resolve(&stored).module(module.descriptor().id())
}

/// Hands `matches` to the module in their order, then calls `finish`, with these apps installed and running.
fn run(matches: &[Match], installed: Vec<InstalledApp>, running: Vec<RunningApp>) -> Recorded {
    Scan::new(installed, running).run(matches)
}

fn webstorm(version: &str) -> Vec<InstalledApp> {
    vec![fresher_testing::installed_app("com.jetbrains.WebStorm", "WebStorm", version).unwrap()]
}

/// The folders of `DataGrip` 2024.3 and 2023.2 in the three places, each changed `days` ago.
fn datagrip(days: [Option<u64>; 3]) -> Vec<Match> {
    let [settings, caches, logs] = days;
    vec![
        aged(0, "/home/Library/Application Support/JetBrains/DataGrip2024.3", settings),
        aged(1, "/home/Library/Caches/JetBrains/DataGrip2024.3", caches),
        aged(2, "/home/Library/Logs/JetBrains/DataGrip2023.2", logs),
    ]
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{CancelToken, EntryKind};
    use fresher_testing::{FakeInstalledApps, installed_app, running_app};

    use super::{Recorded, Scan, datagrip, folder, matched, run, webstorm};
    use crate::JetBrainsModule;

    const CACHES: &str = "/home/Library/Caches/JetBrains";

    #[test]
    fn a_link_or_a_file_named_like_a_version_folder_is_not_kept() {
        let matches = [
            matched(1, &format!("{CACHES}/WebStorm2024.1"), EntryKind::Symlink, 10),
            matched(2, "/home/Library/Logs/JetBrains/WebStorm2024.2", EntryKind::File, 10),
            matched(1, &format!("{CACHES}/DataGrip2024.1"), EntryKind::Symlink, 10),
        ];

        assert_eq!(run(&matches, webstorm("2025.3"), vec![]), Recorded::default());
    }

    #[test]
    fn a_folder_of_an_unknown_product_or_not_a_version_is_not_kept() {
        let matches = [
            folder(1, &format!("{CACHES}/FooStorm2024.1")),
            folder(1, &format!("{CACHES}/JetBrainsClient2025.1")),
            folder(1, &format!("{CACHES}/Toolbox")),
            folder(1, &format!("{CACHES}/WebStorm2024.3.bak")),
            folder(1, &format!("{CACHES}/webstorm2024.2")),
        ];

        assert_eq!(run(&matches, webstorm("2025.3"), vec![]), Recorded::default());
    }

    #[test]
    fn a_match_of_no_place_is_not_kept() {
        assert_eq!(
            run(&[folder(3, &format!("{CACHES}/WebStorm2024.1"))], webstorm("2025.3"), vec![]),
            Recorded::default()
        );
    }

    #[test]
    fn a_running_ide_keeps_its_folders_as_left_out_in_their_categories() {
        let matches = [
            folder(0, "/home/Library/Application Support/JetBrains/WebStorm2024.1"),
            folder(1, &format!("{CACHES}/WebStorm2024.1")),
        ];
        let running = vec![running_app("WebStorm", &["com.jetbrains.WebStorm"]).unwrap()];

        let recorded = run(&matches, webstorm("2025.3"), running);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("jetbrains.settings", "jetbrains.left-out.running", 100),
                ("jetbrains.caches", "jetbrains.left-out.running", 100),
            ]
        );
    }

    #[test]
    fn apps_that_cannot_be_listed_keep_every_folder_as_no_system() {
        let mut matches = datagrip([Some(400); 3]);
        matches.push(folder(1, &format!("{CACHES}/WebStorm2024.1")));
        let installed = Arc::new(FakeInstalledApps::failing("refused"));
        let scan = Scan { installed, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&matches);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("jetbrains.settings", "jetbrains.left-out.no-system", 100),
                ("jetbrains.caches", "jetbrains.left-out.no-system", 100),
                ("jetbrains.logs", "jetbrains.left-out.no-system", 100),
                ("jetbrains.caches", "jetbrains.left-out.no-system", 100),
            ]
        );
    }

    #[test]
    fn a_partial_list_of_apps_keeps_every_folder_as_apps_unread() {
        let mut matches = datagrip([Some(400); 3]);
        matches.push(folder(1, &format!("{CACHES}/WebStorm2024.1")));
        let installed = Arc::new(FakeInstalledApps::partial(webstorm("2025.3"), &["/Applications/Broken.app"]));
        let scan = Scan { installed, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&matches);

        assert_eq!(recorded.findings, []);
        assert!(recorded.left_out().iter().all(|&(_, reason, _)| reason == "jetbrains.left-out.apps-unread"));
        assert_eq!(recorded.left_out.len(), 4);
    }

    #[test]
    fn an_ide_the_toolbox_installed_is_not_taken_for_a_removed_one() {
        let mut toolbox = installed_app("com.jetbrains.datagrip", "DataGrip", "2024.3.2").unwrap();
        toolbox.path =
            "/home/Library/Application Support/JetBrains/Toolbox/apps/DataGrip/ch-0/243.1/DataGrip.app".into();

        let recorded = run(&datagrip([Some(400); 3]), vec![toolbox], vec![]);

        assert_eq!(
            recorded.left_out(),
            [
                ("jetbrains.settings", "jetbrains.left-out.current", 100),
                ("jetbrains.caches", "jetbrains.left-out.current", 100),
            ]
        );
        assert_eq!(recorded.findings.len(), 1);
        assert_eq!(recorded.findings[0].category().as_str(), "jetbrains.logs");
    }

    #[test]
    fn a_removed_ide_that_runs_or_an_unknown_vendor_app_running_keeps_its_folders() {
        for (running, expected) in [
            (running_app("DataGrip", &["com.jetbrains.datagrip"]).unwrap(), "jetbrains.left-out.running"),
            (running_app("DataGrip EAP", &["com.jetbrains.datagrip-EAP"]).unwrap(), "jetbrains.left-out.running"),
            (running_app("Gateway", &["com.jetbrains.gateway"]).unwrap(), "jetbrains.left-out.vendor-app-open"),
        ] {
            let recorded = run(&datagrip([Some(400); 3]), vec![], vec![running.clone()]);

            assert_eq!(recorded.findings, [], "{running:?}");
            assert!(recorded.left_out().iter().all(|&(_, reason, _)| reason == expected), "{running:?}");
            assert_eq!(recorded.left_out.len(), 3);
        }
    }

    #[test]
    fn one_folder_of_a_removed_ide_changed_29_days_ago_keeps_every_one_as_used_recently() {
        for days in [[Some(400), Some(29), Some(400)], [Some(400), None, Some(400)], [Some(0); 3]] {
            let recorded = run(&datagrip(days), vec![], vec![]);

            assert_eq!(recorded.findings, [], "{days:?}");
            assert_eq!(recorded.left_out(), [("jetbrains.uninstalled", "jetbrains.left-out.used-recently", 100); 3]);
        }
    }

    #[test]
    fn a_restored_scan_never_offers_a_removed_ides_data() {
        let scan = Scan { restored: true, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&datagrip([Some(400); 3]));

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("jetbrains.uninstalled", "jetbrains.left-out.used-recently", 100); 3]);
    }

    #[test]
    fn a_walk_that_could_not_read_every_folder_never_offers_a_removed_ides_data() {
        let scan = Scan { walk_complete: false, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&datagrip([Some(400); 3]));

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("jetbrains.uninstalled", "jetbrains.left-out.used-recently", 100); 3]);
    }

    #[test]
    fn an_unknown_vendor_app_installed_keeps_a_removed_ides_data() {
        let unknown = installed_app("com.jetbrains.NewIde", "NewIde", "2026.1").unwrap();

        let recorded = run(&datagrip([Some(400); 3]), vec![unknown], vec![]);

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("jetbrains.uninstalled", "jetbrains.left-out.unknown-ide", 100); 3]);
    }

    #[test]
    fn an_ide_spotlight_finds_elsewhere_keeps_its_data() {
        let mut far = installed_app("com.jetbrains.datagrip-EAP", "DataGrip", "2025.3").unwrap();
        far.path = "/Volumes/Drive/DataGrip.app".into();
        let installed = Arc::new(FakeInstalledApps::new(vec![]).with_elsewhere(vec![far]));
        let scan = Scan { installed, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&datagrip([Some(400); 3]));

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("jetbrains.uninstalled", "jetbrains.left-out.installed-elsewhere", 100); 3]);
    }

    #[test]
    fn a_spotlight_that_cannot_be_asked_keeps_a_removed_ides_data_as_spotlight_unread() {
        let installed = Arc::new(FakeInstalledApps::new(vec![]).failing_search("timed out"));
        let scan = Scan { installed, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&datagrip([Some(400); 3]));

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("jetbrains.uninstalled", "jetbrains.left-out.spotlight-unread", 100); 3]);
    }

    #[test]
    fn a_longer_setting_keeps_what_the_default_offers() {
        let scan = Scan { stored: r#"{"jetbrains.uninstalled-after-days": 60}"#, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&datagrip([Some(40); 3]));

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out.len(), 3);
    }

    #[test]
    fn the_folders_of_a_scan_that_never_finished_are_not_decided_by_the_next() {
        let module = JetBrainsModule::new().unwrap();
        let scan = Scan::new(vec![], vec![]);
        let cancelled = scan.walk(&module, &datagrip([Some(400); 3]), &CancelToken::new());

        let next = scan.run_on(&module, &[], &CancelToken::new());

        assert_eq!(cancelled, Recorded::default());
        assert_eq!(next, Recorded::default());
    }
}

mod positive {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use fresher_core::{
        BundleId, CategoryDescriptor, Claim, DangerLevel, Interest, KnownFolder, Module, ModuleGroup, Pattern, Removal,
        Subject, TextArg, TextKey,
    };

    use super::{Scan, datagrip, folder, run, webstorm};
    use crate::JetBrainsModule;

    fn text(pairs: &[(&str, &str)]) -> BTreeMap<String, TextArg> {
        pairs.iter().map(|&(name, value)| (name.to_owned(), TextArg::Text(value.into()))).collect()
    }

    #[test]
    fn an_older_folder_is_offered_in_its_places_category_for_the_ide() {
        let path = "/home/Library/Application Support/JetBrains/WebStorm2024.3";

        let recorded = run(&[folder(0, path)], webstorm("2025.3.1"), vec![]);

        assert_eq!(recorded.left_out, []);
        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = &recorded.findings[0];
        assert_eq!(finding.category().as_str(), "jetbrains.settings");
        assert_eq!(finding.danger(), DangerLevel::Medium);
        assert_eq!(finding.key().identity(), "settings/WebStorm2024.3");
        assert_eq!(finding.subject(), &Subject::Path(PathBuf::from(path)));
        assert_eq!(finding.removal(), &Removal::MoveToTrash(PathBuf::from(path)));
        assert_eq!(finding.sizes().allocated.bytes(), 100);
        assert_eq!(finding.app().map(BundleId::as_str), Some("com.jetbrains.WebStorm"));
        assert_eq!(finding.title().key().as_str(), "jetbrains.finding.title");
        assert_eq!(finding.title().args(), &text(&[("product", "WebStorm"), ("version", "2024.3")]));
        assert_eq!(finding.reason().key().as_str(), "jetbrains.finding.reason");
        assert_eq!(
            finding.reason().args(),
            &text(&[("product", "WebStorm"), ("version", "2024.3"), ("current", "2025.3")])
        );
    }

    #[test]
    fn every_folder_of_a_removed_ide_unchanged_for_30_days_is_offered_for_it() {
        let recorded = run(&datagrip([Some(30), Some(400), Some(31)]), vec![], vec![]);

        assert_eq!(recorded.left_out, []);
        let found: Vec<_> = recorded
            .findings
            .iter()
            .map(|finding| (finding.category().as_str(), finding.key().identity(), finding.title().key().as_str()))
            .collect();
        assert_eq!(
            found,
            [
                ("jetbrains.uninstalled", "settings/DataGrip2024.3", "jetbrains.finding.uninstalled-settings"),
                ("jetbrains.uninstalled", "caches/DataGrip2024.3", "jetbrains.finding.uninstalled-caches"),
                ("jetbrains.uninstalled", "logs/DataGrip2023.2", "jetbrains.finding.uninstalled-logs"),
            ]
        );
        let finding = &recorded.findings[1];
        let path = PathBuf::from("/home/Library/Caches/JetBrains/DataGrip2024.3");
        assert_eq!(finding.danger(), DangerLevel::Medium);
        assert_eq!(finding.subject(), &Subject::Path(path.clone()));
        assert_eq!(finding.removal(), &Removal::MoveToTrash(path));
        assert_eq!(finding.sizes().allocated.bytes(), 100);
        assert_eq!(finding.app().map(BundleId::as_str), Some("com.jetbrains.datagrip"));
        assert_eq!(finding.title().args(), &text(&[("product", "DataGrip"), ("version", "2024.3")]));
        assert_eq!(finding.reason().key().as_str(), "jetbrains.finding.uninstalled-reason");
        let mut reason = text(&[("product", "DataGrip"), ("version", "2024.3")]);
        reason.insert("days".into(), TextArg::Count(30));
        assert_eq!(finding.reason().args(), &reason);
    }

    #[test]
    fn a_shorter_setting_offers_what_the_default_keeps() {
        let scan = Scan { stored: r#"{"jetbrains.uninstalled-after-days": 7}"#, ..Scan::new(vec![], vec![]) };

        let recorded = scan.run(&datagrip([Some(10); 3]));

        assert_eq!(recorded.left_out, []);
        assert_eq!(recorded.findings.len(), 3);
        assert_eq!(recorded.findings[0].reason().args().get("days"), Some(&TextArg::Count(7)));
    }

    #[test]
    fn the_same_matches_in_two_orders_give_the_same_findings_and_left_outs() {
        let mut matches = vec![
            folder(0, "/home/Library/Application Support/JetBrains/WebStorm2024.3"),
            folder(1, "/home/Library/Caches/JetBrains/WebStorm2025.3"),
            folder(1, "/home/Library/Caches/JetBrains/WebStorm2026.1"),
            folder(2, "/home/Library/Logs/JetBrains/WebStorm2023.1"),
            folder(2, "/home/Library/Logs/JetBrains/PhpStorm2024.1"),
            super::aged(1, "/home/Library/Caches/JetBrains/GoLand2024.1", Some(5)),
        ];
        matches.extend(datagrip([Some(400); 3]));
        let mut reversed = matches.clone();
        reversed.reverse();

        let mut forward = run(&matches, webstorm("2025.3"), vec![]);
        let mut backward = run(&reversed, webstorm("2025.3"), vec![]);

        for recorded in [&mut forward, &mut backward] {
            recorded.findings.sort_by(|a, b| a.key().identity().cmp(b.key().identity()));
            recorded.left_out.sort_by_key(|item| (item.category.as_str(), item.reason.as_str().to_owned()));
        }
        assert_eq!(forward, backward);
        assert_eq!(forward.findings.len(), 6);
        assert_eq!(forward.left_out.len(), 3);
    }

    #[test]
    fn the_categories_keep_their_danger_levels_in_the_development_group() {
        let module = JetBrainsModule::new().unwrap();
        let descriptor = module.descriptor();

        let categories: Vec<_> = descriptor
            .categories()
            .iter()
            .map(|category: &CategoryDescriptor| (category.id().as_str(), category.danger()))
            .collect();
        assert_eq!(
            categories,
            [
                ("jetbrains.caches", DangerLevel::Safe),
                ("jetbrains.logs", DangerLevel::Safe),
                ("jetbrains.settings", DangerLevel::Medium),
                ("jetbrains.uninstalled", DangerLevel::Medium),
            ]
        );
        assert_eq!(descriptor.group(), ModuleGroup::Development);
        let settings: Vec<_> = descriptor.settings().iter().map(|setting| setting.key().as_str()).collect();
        assert_eq!(settings, ["jetbrains.uninstalled-after-days"]);
    }

    #[test]
    fn the_eleven_left_out_reasons_are_declared() {
        let module = JetBrainsModule::new().unwrap();

        let problems: Vec<_> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        assert_eq!(
            problems,
            [
                "jetbrains.left-out.current",
                "jetbrains.left-out.newer",
                "jetbrains.left-out.running",
                "jetbrains.left-out.unknown-version",
                "jetbrains.left-out.no-system",
                "jetbrains.left-out.used-recently",
                "jetbrains.left-out.unknown-ide",
                "jetbrains.left-out.installed-elsewhere",
                "jetbrains.left-out.vendor-app-open",
                "jetbrains.left-out.apps-unread",
                "jetbrains.left-out.spotlight-unread",
            ]
        );
    }

    #[test]
    fn the_interests_are_every_version_folder_of_the_three_places_claimed_whole() {
        let module = JetBrainsModule::new().unwrap();

        let interests: Vec<_> = module
            .interests()
            .iter()
            .map(|interest: &Interest| (interest.pattern().clone(), interest.claim()))
            .collect();

        assert_eq!(
            interests,
            [KnownFolder::AppSupport, KnownFolder::AppCaches, KnownFolder::Logs]
                .map(|folder| (Pattern::Under(folder, "JetBrains/*".into()), Claim::Whole))
        );
        assert!(module.interests().iter().all(|interest| interest.siblings().is_empty()));
    }

    #[test]
    fn the_shipped_products_are_loaded() {
        let module = JetBrainsModule::new().unwrap();

        assert!(module.products.by_folder("WebStorm").is_some());
    }
}
