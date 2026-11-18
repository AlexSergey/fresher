//! The module through `on_match` and `finish`, on a recording sink, with every system fact faked.

use std::path::Path;
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, EntryFacts, EntryKind, InstalledApp, LeftOutItem, Match, Module as _, ModuleContext,
    ModuleSettings, NewFinding, RunningApp, SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts, Timestamp,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::TempModule;

const NOW: u64 = 1_767_225_600;
const DAY: u64 = 24 * 60 * 60;
const USER_TEMP: &str = "/home/private/var/folders/xx/fixture/T";
const SHARED_TEMP: &str = "/home/private/tmp";

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

    /// Each finding as its identity, path and app.
    fn offered(&self) -> Vec<(&str, Option<&Path>, Option<&str>)> {
        self.findings
            .iter()
            .map(|finding| {
                let path = match finding.subject() {
                    fresher_core::Subject::Path(path) => Some(path.as_path()),
                    fresher_core::Subject::External { .. } => None,
                };
                (finding.key().identity(), path, finding.app().map(fresher_core::BundleId::as_str))
            })
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

/// A file of the user's at `path`, matched by `interest` (0 the user's temporary folder, 1 the shared one), of 100
/// bytes, changed and read `days` ago.
fn file(interest: usize, path: &str, days: u64) -> Match {
    let bytes = ByteSize::new(100);
    let time = Some(Timestamp::from_secs(NOW - days * DAY));
    Match {
        interest,
        path: path.into(),
        kind: EntryKind::File,
        sizes: Sizes::complete(bytes, bytes),
        modified: time,
        changed: time,
        accessed: time,
        inside: None,
        siblings: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

fn user(name: &str, days: u64) -> Match {
    file(0, &format!("{USER_TEMP}/{name}"), days)
}

/// The system and settings a scan runs with.
struct Scan {
    installed: Vec<InstalledApp>,
    running: FakeRunningApps,
    /// The settings file's values, as JSON.
    stored: &'static str,
    restored: bool,
}

impl Scan {
    fn new() -> Self {
        Self { installed: Vec::new(), running: FakeRunningApps::default(), stored: "{}", restored: false }
    }

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(self, matches: &[Match]) -> Recorded {
        let module = TempModule::new().unwrap();
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::new(self.installed)),
            running: Arc::new(self.running),
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

fn settings(module: &TempModule, stored: &str) -> ModuleSettings {
    let stored: StoredSettings = serde_json::from_str(stored).unwrap();
    SettingsSchema::new([module.descriptor()]).unwrap().resolve(&stored).module(module.descriptor().id())
}

fn running(name: &str, id: &str) -> RunningApp {
    fresher_testing::running_app(name, &[id]).unwrap()
}

mod negative {
    use fresher_core::{EntryKind, Inside};
    use fresher_testing::FakeRunningApps;

    use super::{Recorded, Scan, file, running, user};

    const CATEGORY: &str = "temp.old-files";

    #[test]
    fn a_match_of_no_place_or_with_no_name_is_not_claimed() {
        let matches = [file(2, "/home/private/var/folders/xx/fixture/T/old.tmp", 30), file(0, "/", 30)];

        assert_eq!(Scan::new().run(&matches), Recorded::default());
    }

    #[test]
    fn a_restored_scan_offers_nothing() {
        let scan = Scan { restored: true, ..Scan::new() };

        let recorded = scan.run(&[user("old.tmp", 30), user("app.lock", 30)]);

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [(CATEGORY, "temp.left-out.scan-again", 100); 2]);
    }

    #[test]
    fn every_reason_is_reported_with_its_key_and_sizes() {
        let socket = fresher_core::Match { kind: EntryKind::Other, ..user("dev.sock", 30) };
        let theirs = fresher_core::Match {
            facts: fresher_core::EntryFacts { owned_by_user: false, ..fresher_core::EntryFacts::REMOVABLE },
            ..file(1, "/home/private/tmp/theirs", 30)
        };
        let locked = fresher_core::Match {
            facts: fresher_core::EntryFacts { user_immutable: true, ..fresher_core::EntryFacts::REMOVABLE },
            ..user("kept.txt", 30)
        };
        let scan = Scan { running: FakeRunningApps::new(vec![running("Editor", "com.example.Editor")]), ..Scan::new() };

        let recorded = scan.run(&[
            user("app.lock", 30),
            socket,
            theirs,
            locked,
            user("com.example.Editor.tmp", 30),
            user("fresh.tmp", 1),
        ]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                (CATEGORY, "temp.left-out.lock", 100),
                (CATEGORY, "temp.left-out.special", 100),
                (CATEGORY, "temp.left-out.system", 100),
                (CATEGORY, "temp.left-out.locked", 100),
                (CATEGORY, "temp.left-out.app-open", 100),
                (CATEGORY, "temp.left-out.recent", 100),
            ]
        );
    }

    #[test]
    fn a_hidden_or_non_utf8_name_after_a_running_app_is_kept() {
        use std::os::unix::ffi::OsStrExt as _;

        let scan = Scan { running: FakeRunningApps::new(vec![running("Chrome", "com.google.Chrome")]), ..Scan::new() };
        let raw = std::path::Path::new(super::USER_TEMP).join(std::ffi::OsStr::from_bytes(b"com.google.Chrome.\xff"));

        let recorded =
            scan.run(&[user(".com.google.Chrome.XXXXXX", 30), fresher_core::Match { path: raw, ..user("x", 30) }]);

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [(CATEGORY, "temp.left-out.app-open", 100); 2]);
    }

    #[test]
    fn a_failed_running_list_keeps_everything() {
        let scan = Scan { running: FakeRunningApps::failing("no list"), ..Scan::new() };

        let recorded = scan.run(&[user("old.tmp", 30)]);

        assert_eq!(recorded.left_out(), [(CATEGORY, "temp.left-out.no-system", 100)]);
    }

    #[test]
    fn a_folder_not_read_in_full_is_neither_offered_nor_left_out() {
        let inside = Inside { complete: false, ..Inside::EMPTY };
        let folder =
            fresher_core::Match { kind: EntryKind::Folder, accessed: None, inside: Some(inside), ..user("x", 30) };

        assert_eq!(Scan::new().run(&[folder]), Recorded::default());
    }

    #[test]
    fn a_longer_setting_keeps_a_ten_day_item() {
        let scan = Scan { stored: r#"{"temp.older-than-days": 14}"#, ..Scan::new() };

        let recorded = scan.run(&[user("ten-days", 10)]);

        assert_eq!(recorded.left_out(), [(CATEGORY, "temp.left-out.recent", 100)]);
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{
        ByteSize, CategoryDescriptor, CategoryId, DangerLevel, EntryKind, Inside, ModuleId, NewFinding, Removal, Sizes,
        Subject, Text, TextArg, TextKey, Timestamp,
    };
    use fresher_testing::installed_app;

    use fresher_core::{Claim, KnownFolder, Module as _, ModuleGroup, Pattern};

    use super::{DAY, NOW, Recorded, SHARED_TEMP, Scan, USER_TEMP, file, user};
    use crate::TempModule;

    #[test]
    fn two_names_differing_only_in_bytes_that_are_not_utf8_get_two_identities() {
        use std::os::unix::ffi::OsStrExt as _;

        let at = |name: &[u8]| {
            let path = Path::new(USER_TEMP).join(std::ffi::OsStr::from_bytes(name));
            fresher_core::Match { path, ..user("x", 30) }
        };

        let recorded = Scan::new().run(&[at(b"tmp\xff"), at(b"tmp\xfe")]);

        let identities: Vec<_> = recorded.offered().into_iter().map(|(identity, _, _)| identity).collect();
        assert_eq!(identities, ["user-temp/tmp\\xFF", "user-temp/tmp\\xFE"]);
    }

    #[test]
    fn the_module_is_in_the_system_group_with_one_safe_category() {
        let module = TempModule::new().unwrap();
        let descriptor = module.descriptor();

        assert_eq!(descriptor.group(), ModuleGroup::System);
        assert_eq!(descriptor.categories().len(), 1);
        assert_eq!(descriptor.problems().len(), 8);
    }

    #[test]
    fn it_claims_every_top_level_item_of_the_two_places_whole() {
        let module = TempModule::new().unwrap();

        let patterns: Vec<_> =
            module.interests().iter().map(|interest| (interest.pattern(), interest.claim())).collect();

        assert_eq!(
            patterns,
            [
                (&Pattern::Under(KnownFolder::UserTemp, "*".into()), Claim::Whole),
                (&Pattern::Under(KnownFolder::SharedTemp, "*".into()), Claim::Whole),
            ]
        );
    }

    #[test]
    fn an_old_file_is_offered_to_the_trash_with_its_name_and_age() {
        let recorded = Scan::new().run(&[user("tmp.IKKNWYIRnq", 30)]);

        let path = Path::new(USER_TEMP).join("tmp.IKKNWYIRnq");
        let category = CategoryDescriptor::new(
            CategoryId::new(ModuleId::new("temp").unwrap(), "temp.old-files").unwrap(),
            DangerLevel::Safe,
        );
        let title =
            Text::new(TextKey::new("temp.finding.title").unwrap()).with("name", TextArg::Text("tmp.IKKNWYIRnq".into()));
        let reason = Text::new(TextKey::new("temp.finding.reason-user").unwrap()).with("days", TextArg::Count(30));
        let bytes = ByteSize::new(100);
        let expected = NewFinding::new(
            &category,
            "user-temp/tmp.IKKNWYIRnq",
            Subject::Path(path.clone()),
            Sizes::complete(bytes, bytes),
            title,
            reason,
            Removal::MoveToTrash(path),
        );
        assert_eq!(recorded.findings, [expected]);
        assert_eq!(recorded.left_out, []);
    }

    #[test]
    fn an_old_item_of_the_shared_place_has_its_identity_and_reason() {
        let recorded = Scan::new().run(&[file(1, &format!("{SHARED_TEMP}/old-build.log"), 10)]);

        assert_eq!(
            recorded.offered(),
            [("shared-temp/old-build.log", Some(Path::new(SHARED_TEMP).join("old-build.log").as_path()), None)]
        );
        assert_eq!(recorded.findings[0].reason().key().as_str(), "temp.finding.reason-shared");
        assert_eq!(recorded.findings[0].reason().args().get("days"), Some(&TextArg::Count(10)));
    }

    #[test]
    fn an_old_folder_and_an_old_link_are_offered_whole_and_as_themselves() {
        let inside = Inside { newest_change: Some(Timestamp::from_secs(NOW - 20 * DAY)), ..Inside::EMPTY };
        let folder = fresher_core::Match {
            kind: EntryKind::Folder,
            accessed: None,
            inside: Some(inside),
            ..user("node-compile-cache", 30)
        };
        let link = fresher_core::Match { kind: EntryKind::Symlink, ..user("docs-link", 30) };

        let recorded = Scan::new().run(&[folder, link]);

        let identities: Vec<_> = recorded.offered().into_iter().map(|(identity, _, _)| identity).collect();
        assert_eq!(identities, ["user-temp/node-compile-cache", "user-temp/docs-link"]);
        assert_eq!(recorded.findings[0].reason().args().get("days"), Some(&TextArg::Count(20)));
    }

    #[test]
    fn an_installed_apps_old_item_counts_for_the_app() {
        let viewer = installed_app("com.example.Viewer", "Viewer", "1.0").unwrap();
        let scan = Scan { installed: vec![viewer], ..Scan::new() };

        let recorded = scan.run(&[
            user("com.example.Viewer", 15),
            user(".com.example.Viewer.XXXXXX", 15),
            user("com.example.Viewerx", 15),
        ]);

        let apps: Vec<_> = recorded.offered().into_iter().map(|(_, _, app)| app).collect();
        assert_eq!(apps, [Some("com.example.Viewer"), Some("com.example.Viewer"), None]);
    }

    #[test]
    fn the_answer_does_not_depend_on_the_walks_order() {
        let matches = [user("a", 30), user("app.lock", 30), user("b", 1)];
        let mut reversed = matches.clone();
        reversed.reverse();

        let answers = |recorded: &Recorded| {
            let mut identities: Vec<String> =
                recorded.findings.iter().map(|finding| finding.key().identity().to_owned()).collect();
            let mut reasons: Vec<String> =
                recorded.left_out.iter().map(|item| item.reason.as_str().to_owned()).collect();
            identities.sort();
            reasons.sort();
            (identities, reasons)
        };

        let forward = answers(&Scan::new().run(&matches));
        let backward = answers(&Scan::new().run(&reversed));

        assert_eq!(forward, backward);
        assert_eq!(forward.0, ["user-temp/a"]);
        assert_eq!(forward.1, ["temp.left-out.lock", "temp.left-out.recent"]);
    }
}
