//! The module as built, and the module through `on_match` and `finish`, on a recording sink, with every system fact
//! faked and plain `Match` values in a home at `/home`.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, EntryFacts, EntryKind, Inside, LeftOutItem, Match, Module as _, ModuleContext, NewFinding,
    SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::XcodeModule;

const NOW: u64 = 1_767_225_600;
const DEVELOPER: &str = "/home/Library/Developer";

/// The interests of the shipped table, in its order.
const IOS: usize = 0;
const WATCH_OS: usize = 1;
const PREVIEWS: usize = 4;
const SIMULATOR_CACHES: usize = 5;
const ARCHIVES: usize = 6;

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

/// A folder of the user's at `path` under `~/Library/Developer`, matched by `interest`, holding `holds`, read in full,
/// of `bytes` logical bytes.
fn folder(interest: usize, path: &str, holds: &[&str], bytes: u64) -> Match {
    Match {
        interest,
        path: Path::new(DEVELOPER).join(path),
        kind: EntryKind::Folder,
        sizes: Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes + 4096)),
        modified: None,
        changed: None,
        accessed: None,
        inside: Some(Inside::EMPTY),
        siblings: Vec::new(),
        holds: holds.iter().map(|name| (*name).to_owned()).collect(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

/// The iOS device support folder `name`, with its sign.
fn ios(name: &str, bytes: u64) -> Match {
    folder(IOS, &format!("Xcode/iOS DeviceSupport/{name}"), &["Symbols"], bytes)
}

/// The watchOS device support folder `name`, with its sign.
fn watch(name: &str, bytes: u64) -> Match {
    folder(WATCH_OS, &format!("Xcode/watchOS DeviceSupport/{name}"), &["Symbols"], bytes)
}

fn previews(bytes: u64) -> Match {
    folder(PREVIEWS, "Xcode/UserData/Previews", &["Simulator Devices"], bytes)
}

fn simulator_caches(bytes: u64) -> Match {
    folder(SIMULATOR_CACHES, "CoreSimulator/Caches", &["dyld"], bytes)
}

/// The archive `name` in the dated folder `date`, with its sign.
fn archive(date: &str, name: &str, bytes: u64) -> Match {
    folder(ARCHIVES, &format!("Xcode/Archives/{date}/{name}"), &["Info.plist"], bytes)
}

/// A folder of every verdict, and entries not claimed.
fn mixed() -> Vec<Match> {
    vec![
        ios("17.2 (21C62)", 1000),
        ios("16.4.1 (20E252)", 2000),
        ios("iPhone15,2 15.0 (19A346)", 3000),
        watch("10.0 (21R355)", 4000),
        previews(5000),
        simulator_caches(6000),
        archive("2026-03-01", "Shop 01-03-2026, 10.15.xcarchive", 7000),
        Match { inside: Some(Inside { special: true, ..Inside::EMPTY }), ..ios("14.0 (18A373)", 10) },
        Match { facts: EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE }, ..watch("9.0 (20R362)", 20) },
        ios("Notes", 30),
        folder(IOS, "Xcode/iOS DeviceSupport/13.0 (17A577)", &[], 40),
        archive("March", "Shop.xcarchive", 50),
        Match { kind: EntryKind::Symlink, ..ios("12.0 (16A366)", 60) },
        ios("11.0 (15A372)", 0),
    ]
}

/// The system a scan runs with.
struct Scan {
    running: Arc<FakeRunningApps>,
    commands: FakeCommandRunner,
}

impl Scan {
    fn new() -> Self {
        Self::running(FakeRunningApps::default())
    }

    fn running(running: FakeRunningApps) -> Self {
        Self { running: Arc::new(running), commands: FakeCommandRunner::default() }
    }

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(&self, matches: &[Match]) -> Recorded {
        let (module, cancel) = (XcodeModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        for found in matches {
            self.with_context(&module, &cancel, |cx| module.on_match(found, &cx, &mut out).unwrap());
        }
        self.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out).unwrap());
        out
    }

    fn with_context<T>(
        &self,
        module: &XcodeModule,
        cancel: &CancelToken,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::failing("not asked by this module")),
            running: Arc::<FakeRunningApps>::clone(&self.running),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::new(fresher_testing::NoFiles),
        });
        let clock = FakeClock::at(NOW);
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings::default())
            .module(module.descriptor().id());
        call(ModuleContext::new(&clock, &self.commands, &system, &settings, cancel))
    }
}

fn running(name: &str, ids: &[&str]) -> fresher_core::RunningApp {
    fresher_testing::running_app(name, ids).unwrap()
}

mod negative {
    use fresher_core::{CancelToken, Match, Module as _, ModuleError};
    use fresher_testing::FakeRunningApps;

    use super::{Recorded, Scan, ios, mixed, previews, running};
    use crate::XcodeModule;

    #[test]
    fn the_module_has_no_setting_beyond_its_switch() {
        let module = XcodeModule::new().unwrap();

        assert_eq!(module.descriptor().settings(), []);
    }

    #[test]
    fn a_match_of_no_interest_is_not_claimed() {
        let stray = Match { interest: 99, ..previews(1000) };

        assert_eq!(Scan::new().run(&[stray]), Recorded::default());
    }

    #[test]
    fn nothing_left_out_is_also_offered() {
        let scan = Scan::new();
        for found in mixed() {
            let recorded = scan.run(std::slice::from_ref(&found));

            assert!(recorded.findings.len() + recorded.left_out.len() <= 1, "{found:?}: {recorded:?}");
        }

        let all = scan.run(&mixed());

        assert_eq!(
            all.identities(),
            [
                "simulator-caches",
                "archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive",
                "previews",
                "ios-device-support/16.4.1 (20E252)",
                "ios-device-support/17.2 (21C62)",
                "ios-device-support/iPhone15,2 15.0 (19A346)",
                "watchos-device-support/10.0 (21R355)",
            ]
        );
        assert_eq!(
            all.left_out(),
            [
                ("xcode.device-support", "xcode.left-out.special", 10),
                ("xcode.device-support", "xcode.left-out.not-yours", 20),
            ]
        );
    }

    #[test]
    fn the_running_apps_are_not_asked_when_no_folder_reaches_the_question() {
        let scan = Scan::new();

        let recorded = scan.run(&[ios("Notes", 10), ios("17.2 (21C62)", 0)]);

        assert_eq!(recorded, Recorded::default());
        assert_eq!(scan.running.calls(), 0);
    }

    #[test]
    fn every_folder_is_left_out_while_xcode_or_the_simulator_runs() {
        for id in ["com.apple.dt.Xcode", "com.apple.iphonesimulator"] {
            let scan = Scan::running(FakeRunningApps::new(vec![running("Xcode", &[id])]));

            let recorded = scan.run(&mixed());

            assert_eq!(recorded.findings, [], "{id}");
            assert_eq!(recorded.left_out.len(), 9, "{id}");
            let open = recorded.left_out.iter().filter(|item| item.reason.as_str() == "xcode.left-out.app-open");
            assert_eq!(open.count(), 7, "{id}");
            assert_eq!(scan.running.calls(), 1, "{id}");
        }
    }

    #[test]
    fn every_folder_is_left_out_no_system_when_the_running_apps_cannot_be_listed() {
        let scan = Scan::running(FakeRunningApps::failing("no process list"));

        let recorded = scan.run(&[ios("17.2 (21C62)", 1000), previews(2000)]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("xcode.previews", "xcode.left-out.no-system", 2000),
                ("xcode.device-support", "xcode.left-out.no-system", 1000),
            ]
        );
        assert_eq!(scan.running.calls(), 1);
    }

    #[test]
    fn no_command_is_run() {
        let scan = Scan::new();

        let recorded = scan.run(&mixed());

        assert_eq!(recorded.findings.len(), 7);
        assert_eq!(scan.commands.calls(), Vec::<String>::new());
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
        let (scan, module, cancel) = (Scan::new(), XcodeModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        for found in mixed() {
            scan.with_context(&module, &cancel, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        cancel.cancel();
        let finished = scan.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out, Recorded::default());
        assert_eq!(scan.running.calls(), 0);
    }

    #[test]
    fn a_cancelled_scan_stops_between_candidates() {
        let (scan, module, cancel) = (Scan::new(), XcodeModule::new().unwrap(), CancelToken::new());
        let mut out = CancelsAtFirstFinding { cancel: cancel.clone(), recorded: Recorded::default() };
        for found in mixed() {
            scan.with_context(&module, &cancel, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        let finished = scan.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out.recorded.identities(), ["simulator-caches"]);
        assert_eq!(out.recorded.left_out, []);
    }
}

mod positive {
    use fresher_core::{BundleId, ByteSize, DangerLevel, Module as _, ModuleGroup, Removal, Subject, TextArg, TextKey};

    use super::{Scan, archive, ios, mixed, previews, simulator_caches, watch};
    use crate::XcodeModule;

    fn text(value: &str) -> TextArg {
        TextArg::Text(value.into())
    }

    #[test]
    fn the_module_is_in_the_development_group() {
        let module = XcodeModule::new().unwrap();

        assert_eq!(module.descriptor().id().as_str(), "xcode");
        assert_eq!(module.descriptor().group(), ModuleGroup::Development);
    }

    #[test]
    fn the_four_categories_have_their_dangers() {
        let module = XcodeModule::new().unwrap();

        let categories: Vec<(&str, DangerLevel)> =
            module.descriptor().categories().iter().map(|c| (c.id().as_str(), c.danger())).collect();
        assert_eq!(
            categories,
            [
                ("xcode.device-support", DangerLevel::Low),
                ("xcode.previews", DangerLevel::Safe),
                ("xcode.simulator-caches", DangerLevel::Safe),
                ("xcode.archives", DangerLevel::High),
            ]
        );
    }

    #[test]
    fn the_four_left_out_reasons_are_declared_in_order() {
        let module = XcodeModule::new().unwrap();

        let keys: Vec<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();
        assert_eq!(
            keys,
            [
                "xcode.left-out.special",
                "xcode.left-out.not-yours",
                "xcode.left-out.no-system",
                "xcode.left-out.app-open"
            ]
        );
    }

    #[test]
    fn the_seven_interests_are_the_shipped_places() {
        let module = XcodeModule::new().unwrap();

        assert_eq!(module.interests().len(), 7);
    }

    #[test]
    fn a_device_support_finding_names_its_platform_version_and_build() {
        let recorded = Scan::new().run(&[ios("iPhone15,2 17.2 (21C62)", 1000)]);

        let finding = &recorded.findings[0];
        assert_eq!(finding.category().as_str(), "xcode.device-support");
        assert_eq!(finding.danger(), DangerLevel::Low);
        assert_eq!(finding.title().key().as_str(), "xcode.title.device-support");
        assert_eq!(finding.title().args().get("platform"), Some(&text("iOS")));
        assert_eq!(finding.title().args().get("version"), Some(&text("17.2")));
        assert_eq!(finding.title().args().get("build"), Some(&text("21C62")));
        assert_eq!(finding.reason().args().get("platform"), Some(&text("iOS")));
        assert_eq!(finding.reason().args().get("version"), Some(&text("17.2")));
        assert_eq!(finding.reason().args().get("size"), Some(&TextArg::Size(ByteSize::new(1000))));
        assert_eq!(finding.key().identity(), "ios-device-support/iPhone15,2 17.2 (21C62)");
        assert_eq!(finding.sizes(), ios("x", 1000).sizes);
    }

    #[test]
    fn a_recent_version_has_its_own_reason_and_the_same_danger() {
        let recorded =
            Scan::new().run(&[ios("17.10 (A1)", 1), ios("17.9 (A2)", 2), ios("16.4.1 (A3)", 3), watch("10.0 (B1)", 4)]);

        let reasons: Vec<(&str, &str, DangerLevel)> = recorded
            .findings
            .iter()
            .map(|finding| (finding.key().identity(), finding.reason().key().as_str(), finding.danger()))
            .collect();
        assert_eq!(
            reasons,
            [
                ("ios-device-support/16.4.1 (A3)", "xcode.reason.device-support", DangerLevel::Low),
                ("ios-device-support/17.10 (A1)", "xcode.reason.device-support-recent", DangerLevel::Low),
                ("ios-device-support/17.9 (A2)", "xcode.reason.device-support-recent", DangerLevel::Low),
                ("watchos-device-support/10.0 (B1)", "xcode.reason.device-support-recent", DangerLevel::Low),
            ]
        );
    }

    #[test]
    fn the_caches_are_emptied_and_the_rest_moved_to_the_trash() {
        let (caches, archived, shown, support) =
            (simulator_caches(2), archive("2026-03-01", "Shop.xcarchive", 4), previews(1), ios("17.2 (21C62)", 3));

        let recorded = Scan::new().run(&[shown.clone(), caches.clone(), support.clone(), archived.clone()]);

        let removals: Vec<(&str, &Subject, &Removal)> = recorded
            .findings
            .iter()
            .map(|finding| (finding.key().identity(), finding.subject(), finding.removal()))
            .collect();
        let subject = |found: &fresher_core::Match| Subject::Path(found.path.clone());
        assert_eq!(
            removals,
            [
                ("simulator-caches", &subject(&caches), &Removal::EmptyFolder(caches.path.clone())),
                (
                    "archives/2026-03-01/Shop.xcarchive",
                    &subject(&archived),
                    &Removal::MoveToTrash(archived.path.clone())
                ),
                ("previews", &subject(&shown), &Removal::EmptyFolder(shown.path.clone())),
                ("ios-device-support/17.2 (21C62)", &subject(&support), &Removal::MoveToTrash(support.path.clone())),
            ]
        );
        let fixed: Vec<(&str, DangerLevel, &str, &str, Option<&TextArg>)> =
            [&recorded.findings[0], &recorded.findings[2]]
                .into_iter()
                .map(|finding| {
                    (
                        finding.category().as_str(),
                        finding.danger(),
                        finding.title().key().as_str(),
                        finding.reason().key().as_str(),
                        finding.reason().args().get("size"),
                    )
                })
                .collect();
        let (two, one) = (TextArg::Size(ByteSize::new(2)), TextArg::Size(ByteSize::new(1)));
        assert_eq!(
            fixed,
            [
                (
                    "xcode.simulator-caches",
                    DangerLevel::Safe,
                    "xcode.title.simulator-caches",
                    "xcode.reason.simulator-caches",
                    Some(&two)
                ),
                ("xcode.previews", DangerLevel::Safe, "xcode.title.previews", "xcode.reason.previews", Some(&one)),
            ]
        );
    }

    #[test]
    fn every_finding_is_counted_in_xcode_s_line() {
        let recorded = Scan::new().run(&mixed());

        assert_eq!(recorded.findings.len(), 7);
        for finding in &recorded.findings {
            assert_eq!(finding.app().map(BundleId::as_str), Some("com.apple.dt.Xcode"));
            assert_eq!(finding.app_name(), None);
        }
    }

    #[test]
    fn an_archive_finding_names_its_date() {
        let recorded = Scan::new().run(&[archive("2026-03-01", "Shop 01-03-2026, 10.15.xcarchive", 7000)]);

        let finding = &recorded.findings[0];
        assert_eq!(finding.category().as_str(), "xcode.archives");
        assert_eq!(finding.danger(), DangerLevel::High);
        assert_eq!(finding.title().key().as_str(), "xcode.title.archive");
        assert_eq!(finding.title().args().get("name"), Some(&text("Shop 01-03-2026, 10.15")));
        assert_eq!(finding.reason().key().as_str(), "xcode.reason.archive");
        assert_eq!(finding.reason().args().get("date"), Some(&text("2026-03-01")));
        assert_eq!(finding.reason().args().get("size"), Some(&TextArg::Size(ByteSize::new(7000))));
        assert_eq!(finding.key().identity(), "archives/2026-03-01/Shop 01-03-2026, 10.15.xcarchive");
    }
}

mod property {
    use proptest::prelude::*;

    use super::{Scan, mixed};

    /// Some of `mixed()`, in any order.
    fn any_matches() -> impl Strategy<Value = Vec<fresher_core::Match>> {
        proptest::sample::subsequence(mixed(), 0..=mixed().len()).prop_shuffle()
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
