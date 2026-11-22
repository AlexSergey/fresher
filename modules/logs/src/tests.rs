//! The module's descriptor and interests, and the module through `on_match` and `finish` on a recording sink, with
//! every system fact faked.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, Claim, DangerLevel, EntryFacts, EntryKind, KnownFolder, LeftOutItem, Match, Module as _,
    ModuleContext, ModuleGroup, ModuleSettings, NewFinding, Pattern, SettingKind, SettingValue, SettingsSchema, Sizes,
    StoredSettings, Subject, SystemFacts, SystemPorts, TextKey, Timestamp, Unit,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::LogsModule;

const NOW: u64 = 1_767_225_600;
const DAY: u64 = 24 * 60 * 60;
const LOGS: &str = "/home/Library/Logs";
const APP_LOGS: &str = "logs.app-logs";
const CRASH_REPORTS: &str = "logs.crash-reports";

fn logs() -> LogsModule {
    LogsModule::new().expect("the module builds")
}

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

    /// Each finding as its category and identity.
    fn offered(&self) -> Vec<(&str, &str)> {
        self.findings.iter().map(|finding| (finding.category().as_str(), finding.key().identity())).collect()
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

/// A file of the user's at `below` (`/`-separated names below `Logs`), matched by the interest of its depth, of 100
/// bytes, modified and changed `days` ago.
fn file(below: &str, days: u64) -> Match {
    let bytes = ByteSize::new(100);
    let time = Some(Timestamp::from_secs(NOW - days * DAY));
    Match {
        interest: below.split('/').count() - 1,
        path: Path::new(LOGS).join(below),
        kind: EntryKind::File,
        sizes: Sizes::complete(bytes, bytes),
        modified: time,
        changed: time,
        accessed: time,
        inside: None,
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

/// The settings and the cache one run of the module scans with.
struct Run {
    /// The settings file's values, as JSON.
    stored: &'static str,
    restored: bool,
}

impl Run {
    const fn new() -> Self {
        Self { stored: "{}", restored: false }
    }

    /// Hands `matches` to a new module in their order, then calls `finish`. No app is installed or running: the
    /// module never asks.
    fn run(self, matches: &[Match]) -> Recorded {
        let module = logs();
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::new(Vec::new())),
            running: Arc::new(FakeRunningApps::failing("never asked")),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::new(fresher_testing::NoFiles),
        });
        let (clock, commands, cancel) = (FakeClock::at(NOW), FakeCommandRunner::default(), CancelToken::new());
        let settings = settings(&module, self.stored);
        let cx = ModuleContext::new(&clock, &commands, &system, &settings, &cancel).complete_walk();
        let cx = if self.restored { cx.restored() } else { cx };
        let mut out = Recorded::default();
        for found in matches {
            module.on_match(found, &cx, &mut out).unwrap();
        }
        module.finish(&cx, &mut out).unwrap();
        out
    }
}

fn settings(module: &LogsModule, stored: &str) -> ModuleSettings {
    let stored: StoredSettings = serde_json::from_str(stored).unwrap();
    SettingsSchema::new([module.descriptor()]).unwrap().resolve(&stored).module(module.descriptor().id())
}

fn path_of(finding: &NewFinding) -> Option<&Path> {
    match finding.subject() {
        Subject::Path(path) => Some(path.as_path()),
        Subject::External { .. } => None,
    }
}

mod negative {
    use super::*;

    #[test]
    fn a_match_outside_the_logs_is_not_claimed() {
        let outside = |path: &str| Match { path: path.into(), ..file("a.log", 30) };
        let matches = [
            outside("/home/Library/Logsx/a.log"),
            outside("/home/Documents/Logs/a.log"),
            outside(LOGS),
            file("JetBrains/IdeaIC2025.1/idea.log", 30),
            file("com.example.App/app.log", 30),
            file("Vendor/state.plist", 30),
            Match { kind: EntryKind::Folder, ..file("Vendor", 30) },
            Match { kind: EntryKind::Symlink, ..file("Vendor/link.log", 30) },
        ];

        assert_eq!(Run::new().run(&matches), Recorded::default());
    }

    #[test]
    fn nothing_left_out_is_also_offered() {
        let socket = Match { kind: EntryKind::Other, ..file("Vendor/agent.sock", 30) };
        let theirs = Match {
            facts: EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE },
            ..file("Vendor/theirs.log", 30)
        };
        let locked = Match {
            facts: EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE },
            ..file("DiagnosticReports/kept.ips", 30)
        };

        let recorded = Run::new().run(&[socket, theirs, locked, file("Vendor/fresh.log", 7)]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                (APP_LOGS, "logs.left-out.special", 100),
                (APP_LOGS, "logs.left-out.not-yours", 100),
                (CRASH_REPORTS, "logs.left-out.locked", 100),
                (APP_LOGS, "logs.left-out.recent", 100),
            ]
        );
    }

    #[test]
    fn a_restored_scan_reports_only_left_outs() {
        let restored = Run { restored: true, ..Run::new() };

        let recorded = restored.run(&[
            file("old log.txt", 30),
            file("DiagnosticReports/App.ips", 30),
            file("Vendor/app.lock", 30),
        ]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [(APP_LOGS, "logs.left-out.scan-again", 100), (CRASH_REPORTS, "logs.left-out.scan-again", 100)]
        );
    }

    #[test]
    fn the_module_declares_no_other_category() {
        let module = logs();

        let categories = module.descriptor().categories();

        assert_eq!(categories.len(), 2);
    }
}

mod positive {
    use fresher_core::{CategoryDescriptor, CategoryId, ModuleId, Removal, Text, TextArg};

    use super::*;

    #[test]
    fn an_app_log_is_offered_in_app_logs_with_its_folder() {
        let recorded = Run::new().run(&[file("Vendor/Updater/1.0/updater.log", 30)]);

        let path = Path::new(LOGS).join("Vendor/Updater/1.0/updater.log");
        let category = CategoryDescriptor::new(
            CategoryId::new(ModuleId::new("logs").unwrap(), APP_LOGS).unwrap(),
            DangerLevel::Safe,
        );
        let title = Text::new(TextKey::new("logs.title").unwrap()).with("name", TextArg::Text("updater.log".into()));
        let reason = Text::new(TextKey::new("logs.reason.folder").unwrap())
            .with("folder", TextArg::Text("Vendor/Updater/1.0".into()))
            .with("days", TextArg::Count(30));
        let bytes = ByteSize::new(100);
        let expected = NewFinding::new(
            &category,
            "app-logs/Vendor/Updater/1.0/updater.log",
            Subject::Path(path.clone()),
            Sizes::complete(bytes, bytes),
            title,
            reason,
            Removal::MoveToTrash(path),
        );
        assert_eq!(recorded.findings, [expected]);
        assert_eq!(recorded.findings[0].app(), None);
        assert_eq!(recorded.left_out, []);
    }

    #[test]
    fn a_top_level_log_reads_your_logs_folder() {
        let recorded = Run::new().run(&[file("Install Log.txt", 12)]);

        assert_eq!(recorded.offered(), [(APP_LOGS, "app-logs/Install Log.txt")]);
        let reason = recorded.findings[0].reason();
        assert_eq!(reason.key().as_str(), "logs.reason.top");
        assert_eq!(reason.args().get("days"), Some(&TextArg::Count(12)));
        assert_eq!(reason.args().get("folder"), None);
    }

    #[test]
    fn a_top_level_file_shaped_as_a_bundle_id_is_offered_as_a_top_level_log() {
        let recorded = Run::new().run(&[file("system.log", 30), file("com.example.App.log", 30)]);

        assert_eq!(recorded.offered(), [(APP_LOGS, "app-logs/system.log"), (APP_LOGS, "app-logs/com.example.App.log")]);
        for finding in &recorded.findings {
            assert_eq!(finding.reason().key().as_str(), "logs.reason.top");
        }
    }

    #[test]
    fn a_crash_report_is_offered_in_crash_reports() {
        let recorded = Run::new()
            .run(&[file("DiagnosticReports/App-2026-11-01.ips", 20), file("DiagnosticReports/Retired/x.ips", 9)]);

        assert_eq!(
            recorded.offered(),
            [(CRASH_REPORTS, "crash-reports/App-2026-11-01.ips"), (CRASH_REPORTS, "crash-reports/Retired/x.ips")]
        );
        let reason = recorded.findings[0].reason();
        assert_eq!(reason.key().as_str(), "logs.reason.report");
        assert_eq!(reason.args().get("days"), Some(&TextArg::Count(20)));
        assert_eq!(recorded.findings[0].title().args().get("name"), Some(&TextArg::Text("App-2026-11-01.ips".into())));
    }

    #[test]
    fn a_finding_moves_the_file_to_the_trash() {
        let recorded = Run::new().run(&[file("Vendor/a.log", 30)]);

        let path = Path::new(LOGS).join("Vendor/a.log");
        assert_eq!(path_of(&recorded.findings[0]), Some(path.as_path()));
        assert_eq!(recorded.findings[0].removal(), &Removal::MoveToTrash(path));
        assert_eq!(recorded.findings[0].danger(), DangerLevel::Safe);
    }

    #[test]
    fn a_left_out_is_in_its_place_s_category() {
        let recorded = Run::new().run(&[file("DiagnosticReports/new.ips", 1), file("Vendor/new.log", 1)]);

        assert_eq!(
            recorded.left_out(),
            [(CRASH_REPORTS, "logs.left-out.recent", 100), (APP_LOGS, "logs.left-out.recent", 100)]
        );
    }

    #[test]
    fn the_setting_moves_the_threshold() {
        let matches = [file("Vendor/ten-days.log", 10)];

        let default = Run::new().run(&matches);
        let longer = Run { stored: r#"{"logs.older-than-days": 14}"#, ..Run::new() }.run(&matches);

        assert_eq!(default.offered(), [(APP_LOGS, "app-logs/Vendor/ten-days.log")]);
        assert_eq!(longer.findings, []);
        assert_eq!(longer.left_out(), [(APP_LOGS, "logs.left-out.recent", 100)]);
    }

    #[test]
    fn the_group_is_system() {
        assert_eq!(logs().descriptor().group(), ModuleGroup::System);
    }

    #[test]
    fn the_categories_are_safe_in_the_order_app_logs_then_crash_reports() {
        let module = logs();

        let categories: Vec<_> = module
            .descriptor()
            .categories()
            .iter()
            .map(|category| (category.id().to_string(), category.danger()))
            .collect();

        assert_eq!(
            categories,
            [("logs.app-logs".to_string(), DangerLevel::Safe), ("logs.crash-reports".to_string(), DangerLevel::Safe),]
        );
    }

    #[test]
    fn the_one_setting_is_the_days_from_7_to_90_with_7_by_default() {
        let module = logs();

        let settings = module.descriptor().settings();

        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].key().as_str(), "logs.older-than-days");
        assert_eq!(settings[0].kind(), &SettingKind::Number { min: 7, max: 90, unit: Some(Unit::Days) });
        assert_eq!(settings[0].default(), &SettingValue::Number(7));
    }

    #[test]
    fn the_problems_are_the_five_left_out_reasons_in_order() {
        let module = logs();

        let problems: Vec<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        assert_eq!(
            problems,
            [
                "logs.left-out.scan-again",
                "logs.left-out.special",
                "logs.left-out.not-yours",
                "logs.left-out.locked",
                "logs.left-out.recent",
            ]
        );
    }

    #[test]
    fn the_interests_are_one_to_six_names_deep_below_logs_and_claim_items() {
        let module = logs();

        let interests = module.interests();

        assert_eq!(interests.len(), 6);
        for (index, interest) in interests.iter().enumerate() {
            let expected = vec!["*"; index + 1].join("/");
            assert_eq!(interest.pattern(), &Pattern::Under(KnownFolder::Logs, expected));
            assert_eq!(interest.claim(), Claim::Item);
            assert_eq!(interest.siblings(), &[] as &[String]);
            assert_eq!(interest.holds(), &[] as &[String]);
            assert!(!interest.matches_only_bundle_ids());
            assert_eq!(interest.name_suffixes(), &[] as &[String]);
        }
    }
}

mod property {
    use proptest::prelude::*;

    use super::*;

    /// The verdicts as sorted lists: the findings' identities and the left-outs' categories and reasons.
    fn answers(recorded: &Recorded) -> (Vec<String>, Vec<String>) {
        let mut identities: Vec<String> =
            recorded.findings.iter().map(|finding| finding.key().identity().to_owned()).collect();
        let mut kept: Vec<String> = recorded
            .left_out
            .iter()
            .map(|item| format!("{} {}", item.category.as_str(), item.reason.as_str()))
            .collect();
        identities.sort();
        kept.sort();
        (identities, kept)
    }

    fn matches() -> Vec<Match> {
        vec![
            file("a log.txt", 30),
            file("Vendor/b.log", 3),
            file("DiagnosticReports/c.ips", 40),
            file("Vendor/state.plist", 30),
            Match { kind: EntryKind::Other, ..file("Vendor/d.sock", 30) },
            file("JetBrains/x/idea.log", 30),
        ]
    }

    proptest! {
        #[test]
        fn the_verdicts_do_not_depend_on_the_order_of_matches(shuffled in Just(matches()).prop_shuffle()) {
            let expected = answers(&Run::new().run(&matches()));

            prop_assert_eq!(answers(&Run::new().run(&shuffled)), expected);
        }
    }
}
