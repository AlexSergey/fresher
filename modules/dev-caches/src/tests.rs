//! The module as built, and the module through `on_match` and `finish`, on a recording sink, with every system fact
//! faked and plain `Match` values in a home at `/home`.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    ByteSize, CancelToken, EntryFacts, EntryKind, Inside, LeftOutItem, MarkerCheck, Match, Module as _, ModuleContext,
    NewFinding, SettingsSchema, Sizes, StoredSettings, SystemFacts, SystemPorts,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::DevCachesModule;
use crate::home::HASH;
use crate::tools::{Role, Tools};

const NOW: u64 = 1_767_225_600;
const XCODE: &str = "com.apple.dt.Xcode";

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

/// The index of the interest of the row `id`.
fn interest(id: &str) -> usize {
    let tools = Tools::shipped().unwrap();
    let row = tools.caches().iter().position(|row| row.id == id).unwrap();
    tools.interests().unwrap().roles.iter().position(|role| *role == Role::Cache(row)).unwrap()
}

/// The index of the interest of the sign place of the row `id`.
fn holder_interest(id: &str) -> usize {
    let tools = Tools::shipped().unwrap();
    let row = tools.caches().iter().position(|row| row.id == id).unwrap();
    let map = tools.interests().unwrap();
    let holder = map.holder_of(row).unwrap();
    map.roles.iter().position(|role| *role == Role::SignHolder(holder)).unwrap()
}

/// The folder of the row `id` in the home `/home`, a derived data project named `Demo-<hash>`.
fn place(id: &str) -> PathBuf {
    let tools = Tools::shipped().unwrap();
    crate::home::place(tools.caches().iter().find(|row| row.id == id).unwrap())
}

/// A folder of the user's of `bytes` bytes, read in full, matched by `interest` at `path`, holding `holds`.
fn folder(interest: usize, path: PathBuf, holds: &[&str], bytes: u64) -> Match {
    let marker = holds.contains(&"CACHEDIR.TAG").then_some(MarkerCheck::Valid);
    let bytes = ByteSize::new(bytes);
    Match {
        interest,
        path,
        kind: EntryKind::Folder,
        sizes: Sizes::complete(bytes, bytes),
        modified: None,
        changed: None,
        accessed: None,
        inside: Some(Inside::EMPTY),
        siblings: Vec::new(),
        holds: holds.iter().map(|name| (*name).to_owned()).collect(),
        marker,
        facts: EntryFacts::REMOVABLE,
    }
}

/// The match of the row `id`'s folder holding `holds`.
fn cache(id: &str, holds: &[&str]) -> Match {
    folder(interest(id), place(id), holds, 100)
}

/// The match of the sign place of the row `id`, holding `holds`.
fn holder(id: &str, at: &str, holds: &[&str]) -> Match {
    folder(holder_interest(id), Path::new("/home").join(at), holds, 0)
}

/// Every row's folder with its sign, and the two sign holders.
fn every_cache() -> Vec<Match> {
    vec![
        cache("npm", &["index-v5", "content-v2"]),
        cache("yarn", &["v6"]),
        cache("homebrew", &[]),
        holder("homebrew", "Library/Caches/Homebrew", &["api"]),
        cache("cargo-cache", &[]),
        cache("cargo-src", &[]),
        holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"]),
        cache("cocoapods", &["Pods"]),
        cache("gradle", &["CACHEDIR.TAG"]),
        cache("pip", &["http-v2"]),
        cache("go-build", &["README", "trim.txt"]),
        cache("xcode-derived-data", &["info.plist"]),
    ]
}

/// The system a scan runs with.
struct Scan {
    running: Arc<FakeRunningApps>,
}

impl Scan {
    fn new() -> Self {
        Self { running: Arc::new(FakeRunningApps::default()) }
    }

    fn running(running: FakeRunningApps) -> Self {
        Self { running: Arc::new(running) }
    }

    /// Hands `matches` to a new module in their order, then calls `finish`.
    fn run(&self, matches: &[Match]) -> Recorded {
        let module = DevCachesModule::new().unwrap();
        let cancel = CancelToken::new();
        let mut out = Recorded::default();
        for found in matches {
            self.on_match(&module, &cancel, found, &mut out);
        }
        self.finish(&module, &cancel, &mut out);
        out
    }

    fn on_match(&self, module: &DevCachesModule, cancel: &CancelToken, found: &Match, out: &mut Recorded) {
        self.with_context(module, cancel, |cx| module.on_match(found, &cx, out).unwrap());
    }

    fn finish(&self, module: &DevCachesModule, cancel: &CancelToken, out: &mut Recorded) {
        self.with_context(module, cancel, |cx| module.finish(&cx.complete_walk(), out).unwrap());
    }

    fn with_context(&self, module: &DevCachesModule, cancel: &CancelToken, call: impl FnOnce(ModuleContext<'_>)) {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::new(Vec::new())),
            running: Arc::<FakeRunningApps>::clone(&self.running),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::new(fresher_testing::NoFiles),
        });
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings::default())
            .module(module.descriptor().id());
        call(ModuleContext::new(&clock, &commands, &system, &settings, cancel));
    }
}

fn running(id: &str) -> fresher_core::RunningApp {
    fresher_testing::running_app("App", &[id]).unwrap()
}

mod negative {
    use fresher_core::{CancelToken, EntryFacts, EntryKind, Inside, MarkerCheck, Match};
    use fresher_testing::FakeRunningApps;

    use super::{Recorded, Scan, XCODE, cache, holder, place, running};
    use crate::DevCachesModule;

    #[test]
    fn a_match_of_no_interest_is_not_claimed() {
        let stray = Match { interest: 99, ..cache("npm", &["index-v5", "content-v2"]) };

        assert_eq!(Scan::new().run(&[stray]), Recorded::default());
    }

    #[test]
    fn a_sign_holder_alone_offers_nothing() {
        let recorded = Scan::new().run(&[holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"])]);

        assert_eq!(recorded, Recorded::default());
    }

    #[test]
    fn a_cache_whose_sign_holder_is_missing_a_link_or_unsigned_is_not_claimed() {
        let link = Match { kind: EntryKind::Symlink, ..holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"]) };
        let invalid =
            Match { marker: Some(MarkerCheck::Invalid), ..holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"]) };
        let no_api = holder("homebrew", "Library/Caches/Homebrew", &["Cask"]);

        for holders in [vec![], vec![link], vec![invalid], vec![no_api]] {
            let mut matches = vec![cache("cargo-cache", &[]), cache("homebrew", &[])];
            matches.extend(holders);

            assert_eq!(Scan::new().run(&matches), Recorded::default());
        }
    }

    #[test]
    fn a_cache_whose_own_sign_is_missing_is_not_claimed() {
        let matches = [
            cache("npm", &["content-v2"]),
            cache("gradle", &[]),
            Match { marker: Some(MarkerCheck::Invalid), ..cache("gradle", &["CACHEDIR.TAG"]) },
            cache("go-build", &["README"]),
            Match {
                path: place("xcode-derived-data").with_file_name("Demo-short"),
                ..cache("xcode-derived-data", &["info.plist"])
            },
        ];

        assert_eq!(Scan::new().run(&matches), Recorded::default());
    }

    #[test]
    fn every_reason_is_reported_with_its_key_the_rows_category_and_sizes() {
        let socket = Match {
            inside: Some(Inside { special: true, ..Inside::EMPTY }),
            ..cache("npm", &["index-v5", "content-v2"])
        };
        let theirs =
            Match { inside: Some(Inside { foreign: true, ..Inside::EMPTY }), ..cache("gradle", &["CACHEDIR.TAG"]) };
        let locked =
            Match { facts: EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE }, ..cache("cargo-src", &[]) };
        let scan = Scan::running(FakeRunningApps::new(vec![running(XCODE)]));

        let recorded = scan.run(&[
            socket,
            theirs,
            locked,
            holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"]),
            cache("xcode-derived-data", &["info.plist"]),
        ]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("dev-caches.cargo", "dev-caches.left-out.not-yours", 100),
                ("dev-caches.gradle", "dev-caches.left-out.not-yours", 100),
                ("dev-caches.npm", "dev-caches.left-out.special", 100),
                ("dev-caches.xcode-derived-data", "dev-caches.left-out.app-open", 100),
            ]
        );
        assert_eq!(scan.running.calls(), 1);
    }

    #[test]
    fn derived_data_is_left_out_when_the_running_apps_cannot_be_listed() {
        let scan = Scan::running(FakeRunningApps::failing("no process list"));

        let recorded = scan.run(&[cache("xcode-derived-data", &["info.plist"])]);

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [("dev-caches.xcode-derived-data", "dev-caches.left-out.no-system", 100)]);
    }

    #[test]
    fn the_running_apps_are_not_read_when_no_candidate_names_an_app() {
        let scan = Scan::new();

        let recorded = scan.run(&[cache("npm", &["index-v5", "content-v2"]), cache("cargo-src", &[])]);

        assert_eq!(recorded.identities(), ["npm/_cacache"]);
        assert_eq!(scan.running.calls(), 0);
    }

    #[test]
    fn a_scan_cancelled_before_finish_leaves_nothing_to_the_next() {
        let module = DevCachesModule::new().unwrap();
        let scan = Scan::new();
        let (cancelled, next) = (CancelToken::new(), CancelToken::new());
        let mut out = Recorded::default();
        scan.on_match(&module, &cancelled, &cache("npm", &["index-v5", "content-v2"]), &mut out);
        scan.on_match(&module, &cancelled, &holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"]), &mut out);

        scan.on_match(&module, &next, &cache("cargo-src", &[]), &mut out);
        scan.finish(&module, &next, &mut out);

        assert_eq!(out, Recorded::default());
    }
}

mod positive {
    use fresher_core::{DangerLevel, Module as _, ModuleGroup, Removal, Subject, TextArg, TextKey};
    use fresher_testing::FakeRunningApps;

    use super::{HASH, Scan, XCODE, cache, every_cache, holder, place, running};
    use crate::DevCachesModule;

    #[test]
    fn the_descriptor_names_the_module_its_group_and_its_nine_categories_in_order() {
        let module = DevCachesModule::new().unwrap();

        let descriptor = module.descriptor();
        assert_eq!(descriptor.id().as_str(), "dev-caches");
        assert_eq!(descriptor.group(), ModuleGroup::Development);
        let categories: Vec<_> =
            descriptor.categories().iter().map(|category| (category.id().as_str(), category.danger())).collect();
        assert_eq!(
            categories,
            [
                ("dev-caches.npm", DangerLevel::Low),
                ("dev-caches.yarn", DangerLevel::Low),
                ("dev-caches.homebrew", DangerLevel::Low),
                ("dev-caches.cargo", DangerLevel::Low),
                ("dev-caches.cocoapods", DangerLevel::Low),
                ("dev-caches.gradle", DangerLevel::Low),
                ("dev-caches.pip", DangerLevel::Low),
                ("dev-caches.go-build", DangerLevel::Safe),
                ("dev-caches.xcode-derived-data", DangerLevel::Safe),
            ]
        );
    }

    #[test]
    fn the_module_has_no_setting_and_declares_its_four_left_out_reasons_in_order() {
        let module = DevCachesModule::new().unwrap();

        let descriptor = module.descriptor();
        assert_eq!(descriptor.settings(), []);
        let problems: Vec<&str> = descriptor.problems().iter().map(TextKey::as_str).collect();
        assert_eq!(
            problems,
            [
                "dev-caches.left-out.not-yours",
                "dev-caches.left-out.special",
                "dev-caches.left-out.app-open",
                "dev-caches.left-out.no-system"
            ]
        );
    }

    #[test]
    fn the_module_asks_for_the_tables_twelve_interests() {
        let module = DevCachesModule::new().unwrap();

        let shipped = crate::tools::Tools::shipped().unwrap().interests().unwrap();

        assert_eq!(module.interests(), shipped.interests.as_slice());
        assert_eq!(module.interests().len(), 12);
    }

    #[test]
    fn every_rows_cache_with_its_sign_is_offered_sorted_by_path() {
        let recorded = Scan::new().run(&every_cache());

        assert_eq!(recorded.left_out, []);
        let offered: Vec<_> = recorded
            .findings
            .iter()
            .map(|finding| (finding.key().identity(), finding.category().as_str(), finding.danger()))
            .collect();
        let project = format!("xcode-derived-data/Demo-{HASH}");
        assert_eq!(
            offered,
            [
                ("cargo-cache/cache", "dev-caches.cargo", DangerLevel::Low),
                ("cargo-src/src", "dev-caches.cargo", DangerLevel::Low),
                ("gradle/caches", "dev-caches.gradle", DangerLevel::Low),
                ("npm/_cacache", "dev-caches.npm", DangerLevel::Low),
                ("cocoapods/CocoaPods", "dev-caches.cocoapods", DangerLevel::Low),
                ("homebrew/downloads", "dev-caches.homebrew", DangerLevel::Low),
                ("yarn/Yarn", "dev-caches.yarn", DangerLevel::Low),
                ("go-build/go-build", "dev-caches.go-build", DangerLevel::Safe),
                ("pip/pip", "dev-caches.pip", DangerLevel::Low),
                (project.as_str(), "dev-caches.xcode-derived-data", DangerLevel::Safe),
            ]
        );
    }

    #[test]
    fn a_finding_moves_its_folder_to_the_trash_with_its_rows_title_and_reason() {
        let recorded = Scan::new().run(&[cache("go-build", &["README", "trim.txt"])]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = recorded.findings.first().unwrap();
        let path = place("go-build");
        assert_eq!(finding.subject(), &Subject::Path(path.clone()));
        assert_eq!(finding.removal(), &Removal::MoveToTrash(path));
        assert_eq!(finding.sizes().allocated.bytes(), 100);
        assert_eq!(finding.title().key().as_str(), "dev-caches.title.go-build");
        assert!(finding.title().args().is_empty());
        assert_eq!(finding.reason().key().as_str(), "dev-caches.finding.reason-build");
        assert_eq!(finding.reason().args().get("tool"), Some(&TextArg::Text("Go".into())));
        assert_eq!(finding.app(), None);
    }

    #[test]
    fn a_download_cache_gives_the_download_reason() {
        let recorded =
            Scan::new().run(&[cache("cargo-src", &[]), holder("cargo-src", ".cargo/registry", &["CACHEDIR.TAG"])]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = recorded.findings.first().unwrap();
        assert_eq!(finding.title().key().as_str(), "dev-caches.title.cargo-src");
        assert_eq!(finding.reason().key().as_str(), "dev-caches.finding.reason-download");
        assert_eq!(finding.reason().args().get("tool"), Some(&TextArg::Text("Cargo".into())));
    }

    #[test]
    fn a_derived_data_project_names_its_project_and_belongs_to_xcode() {
        let scan = Scan::running(FakeRunningApps::new(vec![running("com.example.Editor")]));
        let my_app = fresher_core::Match {
            path: place("xcode-derived-data").with_file_name(format!("My-App-{HASH}")),
            ..cache("xcode-derived-data", &["info.plist"])
        };

        let recorded = scan.run(&[my_app]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = recorded.findings.first().unwrap();
        assert_eq!(finding.key().identity(), format!("xcode-derived-data/My-App-{HASH}"));
        assert_eq!(finding.title().key().as_str(), "dev-caches.title.xcode-derived-data");
        assert_eq!(finding.title().args().get("project"), Some(&TextArg::Text("My-App".into())));
        assert_eq!(finding.reason().args().get("tool"), Some(&TextArg::Text("Xcode".into())));
        assert_eq!(finding.app().map(fresher_core::BundleId::as_str), Some(XCODE));
    }

    #[test]
    fn the_same_matches_in_two_orders_give_the_same_answers() {
        let mut matches = every_cache();
        matches.push(fresher_core::Match {
            inside: Some(fresher_core::Inside { foreign: true, ..fresher_core::Inside::EMPTY }),
            path: place("xcode-derived-data").with_file_name(format!("Other-{HASH}")),
            ..cache("xcode-derived-data", &["info.plist"])
        });
        let forward = Scan::new().run(&matches);

        matches.reverse();
        let backward = Scan::new().run(&matches);

        assert_eq!(forward, backward);
        assert_eq!(forward.findings.len(), 10);
        assert_eq!(forward.left_out.len(), 1);
    }

    #[test]
    fn a_cargo_cache_whose_sign_holder_arrives_after_it_is_offered() {
        let matches = [cache("cargo-cache", &[]), holder("cargo-cache", ".cargo/registry", &["CACHEDIR.TAG"])];

        let recorded = Scan::new().run(&matches);

        assert_eq!(recorded.identities(), ["cargo-cache/cache"]);
    }

    #[test]
    fn a_scan_offers_again_after_an_earlier_one_was_cancelled() {
        let module = DevCachesModule::new().unwrap();
        let scan = Scan::new();
        let (cancelled, next) = (fresher_core::CancelToken::new(), fresher_core::CancelToken::new());
        let mut out = super::Recorded::default();
        scan.on_match(&module, &cancelled, &cache("yarn", &["v6"]), &mut out);

        scan.on_match(&module, &next, &cache("pip", &["http-v2"]), &mut out);
        scan.finish(&module, &next, &mut out);

        assert_eq!(out.identities(), ["pip/pip"]);
    }
}
