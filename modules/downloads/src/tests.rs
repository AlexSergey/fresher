//! The module as built, and the module through `on_match` and `finish`, on a recording sink, with every system fact
//! faked and plain `Match` values in a home at `/home`.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, EntryFacts, EntryKind, Inside, LeftOutItem, Match, Module as _, ModuleContext, NewFinding,
    PlaceSeen, Receipt, SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts, Timestamp,
};
use fresher_testing::{
    FakeClock, FakeCommandRunner, FakeInstallRecords, FakeInstalledApps, FakeKnownFolders, FakeRunningApps, changed_at,
    installed_app,
};

use crate::DownloadsModule;
use crate::gathered::{BUNDLES, FILES};

const DAY: u64 = 24 * 60 * 60;
const NOW: u64 = 1_767_225_600;
const GIGABYTE: u64 = 1_000_000_000;

/// Keeps everything the module reports, in order.
#[derive(Debug, Default, PartialEq, Eq)]
struct Recorded {
    findings: Vec<NewFinding>,
    left_out: Vec<LeftOutItem>,
}

impl Recorded {
    /// Each finding's identity.
    fn identities(&self) -> Vec<&str> {
        self.findings.iter().map(|finding| finding.key().identity()).collect()
    }

    /// Each left-out as its category and reason.
    fn left_out(&self) -> Vec<(&str, &str)> {
        self.left_out.iter().map(|item| (item.category.as_str(), item.reason.as_str())).collect()
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

/// A match of `interest` at `path` under the home, of `kind` and `bytes` bytes, unchanged for `days` days.
fn entry(interest: usize, path: &str, kind: EntryKind, bytes: u64, days: u64) -> Match {
    let time = Some(Timestamp::from_secs(NOW - days * DAY));
    let size = ByteSize::new(bytes);
    Match {
        interest,
        path: Path::new("/home").join(path),
        kind,
        sizes: Sizes::complete(size, size),
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

/// A file of Downloads of `bytes` bytes, unchanged for `days` days.
fn file(name: &str, bytes: u64, days: u64) -> Match {
    entry(FILES, &format!("Downloads/{name}"), EntryKind::File, bytes, days)
}

/// A bundle of Downloads holding `holds`, nothing in it changed for `days` days.
fn bundle(name: &str, holds: &str, days: u64) -> Match {
    let folder = entry(BUNDLES, &format!("Downloads/{name}"), EntryKind::Folder, 500, days);
    Match { holds: vec![holds.into()], ..folder }
}

/// The system a scan runs with: by default nothing installed or running, no receipts, macOS 15.1, Downloads read.
struct System {
    apps: Arc<FakeInstalledApps>,
    running: Arc<FakeRunningApps>,
    installs: Arc<FakeInstallRecords>,
}

impl System {
    fn new() -> Self {
        Self {
            apps: Arc::new(FakeInstalledApps::default()),
            running: Arc::new(FakeRunningApps::default()),
            installs: Arc::new(FakeInstallRecords::default().with_version(15, 1)),
        }
    }

    fn apps(self, apps: FakeInstalledApps) -> Self {
        Self { apps: Arc::new(apps), ..self }
    }

    fn installs(self, installs: FakeInstallRecords) -> Self {
        Self { installs: Arc::new(installs), ..self }
    }

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(&self, matches: &[Match]) -> Recorded {
        let (module, cancel) = (DownloadsModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        self.gather(&module, &cancel, matches, &mut out);
        self.with_context(&module, &cancel, |cx| module.finish(&cx, &mut out).unwrap());
        out
    }

    fn gather(&self, module: &DownloadsModule, cancel: &CancelToken, matches: &[Match], out: &mut Recorded) {
        for found in matches {
            self.with_context(module, cancel, |cx| module.on_match(found, &cx, out).unwrap());
        }
    }

    fn with_context<T>(
        &self,
        module: &DownloadsModule,
        cancel: &CancelToken,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::<FakeInstalledApps>::clone(&self.apps),
            running: Arc::<FakeRunningApps>::clone(&self.running),
            installs: Arc::<FakeInstallRecords>::clone(&self.installs),
        });
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings::default())
            .module(module.descriptor().id());
        let places = [PlaceSeen::Read, PlaceSeen::Read];
        call(ModuleContext::new(&clock, &commands, &system, &settings, cancel).complete_walk().with_places(&places))
    }
}

fn firefox() -> fresher_core::InstalledApp {
    changed_at(installed_app("org.mozilla.firefox", "Firefox", "133.0").unwrap(), NOW - 10 * DAY)
}

/// A Downloads with something of every category, and of the user's.
fn every_kind() -> Vec<Match> {
    vec![
        file("report.pdf", 2_000, 400),
        file("movie.mp4.crdownload", 700, 40),
        file("song.flac.part", 300, 40),
        file("song.flac", 0, 40),
        bundle("trip.zip.download", "Info.plist", 40),
        file("Firefox 133.0.dmg", 900, 20),
        file("Firefox 140.0.dmg", 900, 20),
        file("Tool-3.1.pkg", 800, 20),
        bundle("Install macOS Sonoma.app", "Contents", 400),
        file("video.mov", 2 * GIGABYTE, 400),
        file("notes-backup.dmg", 2 * GIGABYTE, 400),
    ]
}

fn every_kind_system() -> System {
    let receipts = vec![Receipt { package_file: "Tool-3.1.pkg".into(), installed: Timestamp::from_secs(NOW - DAY) }];
    System::new()
        .apps(FakeInstalledApps::new(vec![firefox()]))
        .installs(FakeInstallRecords::default().with_version(15, 1).with_receipts(receipts))
}

mod negative {
    use fresher_core::{CancelToken, EntryKind, Module as _, ModuleError};
    use fresher_testing::FakeInstalledApps;

    use super::{GIGABYTE, Recorded, System, bundle, entry, every_kind, file};
    use crate::DownloadsModule;
    use crate::gathered::FILES;

    #[test]
    fn nothing_is_asked_of_the_system_without_a_candidate() {
        let system = System::new();

        let recorded = system.run(&[
            file("report.pdf", 2_000, 400),
            file("notes.txt", 10, 3),
            file("old.zip", 900_000_000, 400),
            entry(FILES, "Downloads/photos", EntryKind::Folder, 2 * GIGABYTE, 400),
        ]);

        assert_eq!(recorded, Recorded::default());
        assert_eq!((system.apps.calls(), system.running.calls(), system.installs.calls()), (0, 0, 0));
    }

    #[test]
    fn a_cancelled_scan_stops_between_entries() {
        let system = System::new();
        let (module, cancel) = (DownloadsModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        system.gather(&module, &cancel, &every_kind(), &mut out);

        cancel.cancel();
        let finished = system.with_context(&module, &cancel, |cx| module.finish(&cx, &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out, Recorded::default());
    }

    #[test]
    fn a_link_a_folder_of_yours_or_a_file_inside_one_is_never_claimed() {
        let recorded = System::new().run(&[
            entry(FILES, "Downloads/movie.mp4.part", EntryKind::Symlink, 100, 400),
            entry(FILES, "Downloads/photos", EntryKind::Folder, 2 * GIGABYTE, 400),
            entry(FILES, "Downloads/photos/big.mov", EntryKind::File, 2 * GIGABYTE, 400),
            entry(FILES, "Documents/big.mov", EntryKind::File, 2 * GIGABYTE, 400),
            bundle("Example.app", "Contents", 400),
            bundle("trip.download", "Contents", 400),
        ]);

        assert_eq!(recorded, Recorded::default());
    }

    #[test]
    fn installers_are_kept_while_the_apps_cannot_be_listed_and_receipts_unread_prove_nothing() {
        let system = System::new()
            .apps(FakeInstalledApps::failing("no apps"))
            .installs(fresher_testing::FakeInstallRecords::default().failing_receipts("no receipts"));

        let recorded = system.run(&[file("Tool-3.1.pkg", 800, 20)]);

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("downloads.installers", "downloads.left-out.no-system")]);
    }
}

mod positive {
    use fresher_core::{
        BundleId, Claim, GuardedFolder, KnownFolder, Module as _, Need, Pattern, Removal, TextArg, Timestamp,
    };
    use fresher_testing::{FakeInstallRecords, FakeInstalledApps, changed_at, installed_app};

    use super::{DAY, NOW, System, every_kind, every_kind_system, file};
    use crate::DownloadsModule;

    #[test]
    fn every_category_needs_downloads() {
        let module = DownloadsModule::new().unwrap();

        let categories = module.descriptor().categories();
        assert_eq!(categories.len(), 5);
        let need = Need::required(fresher_core::Capability::Folder(GuardedFolder::Downloads));
        for category in categories {
            assert_eq!(category.needs(), [need], "{}", category.id());
        }
    }

    #[test]
    fn the_interests_are_one_item_star_and_one_whole_suffixed_star() {
        let module = DownloadsModule::new().unwrap();

        let interests = module.interests();
        assert_eq!(interests.len(), 2);
        let star = Pattern::Under(KnownFolder::Downloads, "*".into());
        assert_eq!(interests[0].pattern(), &star);
        assert_eq!(interests[0].claim(), Claim::Item);
        assert_eq!(interests[0].name_suffixes(), [] as [String; 0]);
        assert_eq!(interests[1].pattern(), &star);
        assert_eq!(interests[1].claim(), Claim::Whole);
        assert_eq!(interests[1].name_suffixes(), [".download", ".app"]);
        assert_eq!(interests[1].holds(), ["Info.plist", "Contents"]);
    }

    #[test]
    fn every_kind_is_offered_or_left_out_by_name() {
        let recorded = every_kind_system().run(&every_kind());

        assert_eq!(
            recorded.identities(),
            [
                "installers/Firefox 133.0.dmg",
                "macos-installers/Install macOS Sonoma.app",
                "installers/Tool-3.1.pkg",
                "unfinished/movie.mp4.crdownload",
                "unfinished/song.flac.part",
                "unfinished/song.flac",
                "safari-resumable/trip.zip.download",
                "large-untouched/video.mov",
            ]
        );
        assert_eq!(recorded.left_out(), [("downloads.installers", "downloads.left-out.not-older")]);
    }

    #[test]
    fn the_matches_in_two_orders_give_the_same_answers() {
        let mut reversed = every_kind();
        reversed.reverse();

        assert_eq!(every_kind_system().run(&every_kind()), every_kind_system().run(&reversed));
    }

    #[test]
    fn an_installer_proven_by_name_is_for_its_app_and_says_its_version() {
        let recorded = every_kind_system().run(&[file("Firefox 133.0.dmg", 900, 20)]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = &recorded.findings[0];
        assert_eq!(finding.app(), Some(&BundleId::new("org.mozilla.firefox").unwrap()));
        assert_eq!(finding.reason().key().as_str(), "downloads.reason.installer");
        assert_eq!(finding.reason().args()["app"], TextArg::Text("Firefox".into()));
        assert_eq!(finding.reason().args()["version"], TextArg::Text("133.0".into()));
        assert_eq!(finding.title().args()["name"], TextArg::Text("Firefox 133.0.dmg".into()));
        let path = std::path::PathBuf::from("/home/Downloads/Firefox 133.0.dmg");
        assert_eq!(finding.removal(), &Removal::MoveToTrash(path));
    }

    #[test]
    fn a_package_proven_by_its_receipt_has_no_app_and_says_when() {
        let recorded = every_kind_system().run(&[file("Tool-3.1.pkg", 800, 20)]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = &recorded.findings[0];
        assert_eq!(finding.app(), None);
        assert_eq!(finding.reason().key().as_str(), "downloads.reason.installer-receipt");
        assert_eq!(finding.reason().args()["date"], TextArg::Date(Timestamp::from_secs(NOW - DAY)));
    }

    #[test]
    fn unreadable_receipts_leave_the_name_rule_to_prove_a_package() {
        let tool = changed_at(installed_app("com.example.Tool", "Tool", "3.1").unwrap(), NOW - DAY);
        let system = System::new()
            .apps(FakeInstalledApps::new(vec![tool]))
            .installs(FakeInstallRecords::default().failing_receipts("no receipts"));

        let recorded = system.run(&[file("Tool-3.1.pkg", 800, 20), file("Tool-3.0.pkg", 800, 30)]);

        assert_eq!(recorded.identities(), ["installers/Tool-3.0.pkg", "installers/Tool-3.1.pkg"]);
        assert_eq!(system.installs.calls(), 1);
    }

    #[test]
    fn an_empty_placeholder_is_told_once_right_after_its_part() {
        let recorded = System::new().run(&[file("setup.dmg.part", 300, 40), file("setup.dmg", 0, 40)]);

        assert_eq!(recorded.identities(), ["unfinished/setup.dmg.part", "unfinished/setup.dmg"]);
        assert_eq!(recorded.findings[1].reason().key().as_str(), "downloads.reason.placeholder");
        assert_eq!(recorded.left_out, []);
    }
}
