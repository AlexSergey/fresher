//! The module through `on_match` and `finish`, on a recording sink, with plain `Match` values in a home at `/home`,
//! the small files faked by path and the projects set by hand.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{
    Artifact, ByteSize, CancelToken, EntryFacts, EntryKind, Inside, LastChange, LeftOutItem, Match, Module as _,
    ModuleContext, NewFinding, Project, ProjectKind, ReadError, SettingsSchema, Sizes, SmallFile, SmallFiles,
    StoredSettings, SystemFacts, SystemPorts, Timestamp,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::UnusedToolchainsModule;

const NOW: u64 = 1_767_225_600;
const CHANGED: u64 = 1_760_000_000;
const ACCESSED: u64 = 1_765_000_000;

/// The interests of the shipped table, in its order.
const TOOLCHAINS: usize = 0;
const RUSTUP_HOME: usize = 1;
const NODE_VERSIONS: usize = 2;
const ALIAS: usize = 3;
const LTS_ALIAS: usize = 4;
const RUST_TOOLCHAIN_TOML: usize = 5;
const TOOL_VERSIONS: usize = 7;
const NVMRC: usize = 8;

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

/// Small files by path, counting the reads of each; a path it does not know is gone.
#[derive(Debug, Default)]
struct FakeSmallFiles {
    files: BTreeMap<PathBuf, SmallFile>,
    reads: Mutex<BTreeMap<PathBuf, usize>>,
}

impl FakeSmallFiles {
    fn with(files: &[(&str, SmallFile)]) -> Self {
        Self { files: files.iter().map(|(at, file)| (path(at), file.clone())).collect(), ..Self::default() }
    }

    /// How many times the file at `/home/<place>` was read.
    fn reads(&self, place: &str) -> usize {
        self.reads.lock().unwrap().get(&path(place)).copied().unwrap_or(0)
    }

    fn all_reads(&self) -> BTreeMap<PathBuf, usize> {
        self.reads.lock().unwrap().clone()
    }
}

impl SmallFiles for FakeSmallFiles {
    fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError> {
        let at = folder.join(name);
        *self.reads.lock().unwrap().entry(at.clone()).or_default() += 1;
        match self.files.get(&at) {
            None | Some(SmallFile::Missing) => Err(ReadError::Vanished),
            Some(SmallFile::Text(text)) => Ok(text.as_bytes().to_vec()),
            Some(SmallFile::NotAFile) => Err(ReadError::Replaced),
            Some(SmallFile::TooLong) => Ok(vec![b'1'; limit]),
            Some(SmallFile::NotText) => Ok(vec![0xff, 0xfe]),
            Some(SmallFile::Unreadable) => Err(ReadError::Failed),
        }
    }
}

fn path(place: &str) -> PathBuf {
    Path::new("/home").join(place)
}

fn text(text: &str) -> SmallFile {
    SmallFile::Text(text.to_owned())
}

/// A version folder of the user's of `bytes` logical bytes at `/home/<place>`, read in full, matched by `interest`.
fn folder(interest: usize, place: &str, bytes: u64) -> Match {
    Match {
        interest,
        path: path(place),
        kind: EntryKind::Folder,
        sizes: Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes + 4096)),
        modified: None,
        changed: None,
        accessed: None,
        inside: Some(Inside {
            newest_change: Some(Timestamp::from_secs(CHANGED)),
            newest_file_access: Some(Timestamp::from_secs(ACCESSED)),
            ..Inside::EMPTY
        }),
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

fn toolchain(name: &str) -> Match {
    folder(TOOLCHAINS, &format!(".rustup/toolchains/{name}"), 1000)
}

fn node(name: &str) -> Match {
    folder(NODE_VERSIONS, &format!(".nvm/versions/node/{name}"), 2000)
}

/// `~/.rustup`, holding its `settings.toml`.
fn rustup_home() -> Match {
    Match { inside: None, holds: vec!["settings.toml".to_owned()], ..folder(RUSTUP_HOME, ".rustup", 0) }
}

/// The alias file `~/.nvm/alias/<key>` (`lts/<name>` for an LTS alias).
fn alias(key: &str) -> Match {
    let interest = if key.starts_with("lts/") { LTS_ALIAS } else { ALIAS };
    Match { kind: EntryKind::File, inside: None, ..folder(interest, &format!(".nvm/alias/{key}"), 10) }
}

/// The folder `/home/<place>` holding the pin of `interest`.
fn pin(interest: usize, place: &str) -> Match {
    Match { inside: None, ..folder(interest, place, 0) }
}

/// `found` with a socket inside.
fn holding_a_socket(found: Match) -> Match {
    Match { inside: found.inside.map(|inside| Inside { special: true, ..inside }), ..found }
}

/// The pointer and pin files of `mixed`.
fn mixed_files() -> FakeSmallFiles {
    FakeSmallFiles::with(&[
        (
            ".rustup/settings.toml",
            text(
                "default_toolchain = \"stable-aarch64-apple-darwin\"\n\n[overrides]\n\
                 \"/home/dev/old\" = \"1.75.0-aarch64-apple-darwin\"\n",
            ),
        ),
        (".nvm/alias/default", text("20\n")),
        (".nvm/alias/lts/hydrogen", text("v18.17.0\n")),
        ("dev/app/rust-toolchain.toml", text("[toolchain]\nchannel = \"1.79.0\"\n")),
        ("dev/app/.tool-versions", text("rust 1.80.0\nnodejs 16.20.2 # the old one\n")),
        ("dev/web/.nvmrc", text("lts/hydrogen\n")),
    ])
}

/// Versions of both managers, each kept by one pointer or pin, offered, or not claimed, with their pointers and pins.
fn mixed() -> Vec<Match> {
    vec![
        toolchain("stable-aarch64-apple-darwin"),
        toolchain("1.70.0-aarch64-apple-darwin"),
        holding_a_socket(toolchain("1.72.0-aarch64-apple-darwin")),
        toolchain("1.75.0-aarch64-apple-darwin"),
        toolchain("1.79.0-aarch64-apple-darwin"),
        toolchain("1.80.0-aarch64-apple-darwin"),
        toolchain("nightly-2024-01-01-aarch64-apple-darwin"),
        toolchain("my-toolchain"),
        Match { kind: EntryKind::Symlink, inside: None, ..toolchain("linked") },
        node("v14.21.3"),
        node("v16.20.2"),
        node("v18.17.0"),
        node("v19.0.0"),
        node("v20.11.0"),
        node("v22.11.0"),
        rustup_home(),
        alias("default"),
        Match { kind: EntryKind::Folder, ..alias("lts") },
        alias("lts/hydrogen"),
        pin(RUST_TOOLCHAIN_TOML, "dev/app"),
        pin(TOOL_VERSIONS, "dev/app"),
        pin(NVMRC, "dev/web"),
    ]
}

/// What `mixed` offers.
const MIXED_OFFERED: [&str; 4] = [
    "nvm/v14.21.3",
    "nvm/v19.0.0",
    "rustup/1.70.0-aarch64-apple-darwin",
    "rustup/nightly-2024-01-01-aarch64-apple-darwin",
];

/// The system a scan runs with: its small files, its projects (`None`: not known yet) and the Node switch.
struct World {
    files: Arc<FakeSmallFiles>,
    projects: Option<Vec<Project>>,
    node_on: bool,
}

impl World {
    fn new(files: FakeSmallFiles) -> Self {
        Self { files: Arc::new(files), projects: Some(Vec::new()), node_on: true }
    }

    fn run(&self, matches: &[Match]) -> Recorded {
        let mut out = Recorded::default();
        let (module, cancel) = (UnusedToolchainsModule::new().unwrap(), CancelToken::new());
        self.feed(&module, &cancel, matches, &mut out);
        self.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out)).unwrap();
        out
    }

    /// Hands `matches` to `module` in their order.
    fn feed(
        &self,
        module: &UnusedToolchainsModule,
        cancel: &CancelToken,
        matches: &[Match],
        out: &mut dyn fresher_core::FindingSink,
    ) {
        for found in matches {
            self.with_context(module, cancel, |cx| module.on_match(found, &cx, out)).unwrap();
        }
    }

    fn with_context<T>(
        &self,
        module: &UnusedToolchainsModule,
        cancel: &CancelToken,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::new(Vec::new())),
            running: Arc::new(FakeRunningApps::default()),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::<FakeSmallFiles>::clone(&self.files),
        });
        if let Some(projects) = &self.projects {
            system.set_projects(projects.clone());
        }
        let (clock, commands) = (FakeClock::at(NOW), FakeCommandRunner::default());
        let values = [("unused-toolchains.node-versions".to_owned(), serde_json::Value::Bool(self.node_on))]
            .into_iter()
            .collect();
        let settings = SettingsSchema::new([module.descriptor()])
            .unwrap()
            .resolve(&StoredSettings { values })
            .module(module.descriptor().id());
        call(ModuleContext::new(&clock, &commands, &system, &settings, cancel))
    }
}

/// A Node project at `/home/<root>` whose `node_modules` the walk saw.
fn node_project(root: &str) -> Project {
    let artifact =
        Artifact { path: path(root).join("node_modules"), sizes: Sizes::ZERO, last_change: LastChange::Unknown };
    Project {
        root: path(root),
        kinds: vec![ProjectKind::Node],
        artifacts: vec![artifact],
        caches: Vec::new(),
        last_activity: LastChange::Unknown,
    }
}

mod negative {
    use fresher_core::{CancelToken, EntryKind, LeftOutItem, Match, Module as _, NewFinding, SmallFile};

    use super::{
        FakeSmallFiles, MIXED_OFFERED, NVMRC, Recorded, TOOL_VERSIONS, World, alias, mixed, mixed_files, node,
        node_project, pin, rustup_home, text, toolchain,
    };
    use crate::UnusedToolchainsModule;

    #[test]
    fn nothing_is_claimed_before_the_projects_are_known() {
        let world = World { projects: None, ..World::new(mixed_files()) };

        let recorded = world.run(&mixed());

        assert_eq!(recorded, Recorded::default());
        assert!(world.files.all_reads().is_empty());
    }

    #[test]
    fn a_pin_inside_node_modules_is_not_read() {
        let files = FakeSmallFiles::with(&[
            ("dev/app/node_modules/dep/.nvmrc", text("18\n")),
            ("dev/app/node_modules/.nvmrc", SmallFile::Unreadable),
            ("dev/app/node_modules-kit/.nvmrc", text("16\n")),
        ]);
        let world = World { projects: Some(vec![node_project("dev/app")]), ..World::new(files) };
        let matches = [
            node("v16.20.2"),
            node("v18.17.0"),
            node("v22.11.0"),
            pin(NVMRC, "dev/app/node_modules/dep"),
            pin(NVMRC, "dev/app/node_modules"),
            pin(NVMRC, "dev/app/node_modules-kit"),
        ];

        let recorded = world.run(&matches);

        assert_eq!(recorded.identities(), ["nvm/v18.17.0"]);
        assert_eq!(recorded.left_out, []);
        assert_eq!(world.files.reads("dev/app/node_modules/dep/.nvmrc"), 0);
        assert_eq!(world.files.reads("dev/app/node_modules/.nvmrc"), 0);
        assert_eq!(world.files.reads("dev/app/node_modules-kit/.nvmrc"), 1);
    }

    #[test]
    fn a_linked_pin_blocks_its_manager() {
        let files = FakeSmallFiles::with(&[
            (".rustup/settings.toml", text("default_toolchain = \"stable\"\n")),
            ("dev/app/.nvmrc", SmallFile::NotAFile),
        ]);
        let world = World::new(files);
        let matches = [
            toolchain("stable-aarch64-apple-darwin"),
            toolchain("1.70.0-aarch64-apple-darwin"),
            rustup_home(),
            node("v16.20.2"),
            node("v22.11.0"),
            pin(NVMRC, "dev/app"),
        ];

        let recorded = world.run(&matches);

        assert_eq!(recorded.identities(), ["rustup/1.70.0-aarch64-apple-darwin"]);
        assert_eq!(recorded.left_out(), [("unused-toolchains.node", "unused-toolchains.left-out.pin-unknown", 2000)]);
    }

    #[test]
    fn every_unreadable_answer_is_unknown_and_a_gone_pin_is_not_counted() {
        for file in [SmallFile::NotAFile, SmallFile::TooLong, SmallFile::NotText, SmallFile::Unreadable] {
            let files = FakeSmallFiles::with(&[("dev/app/.tool-versions", file.clone())]);
            let recorded = World::new(files).run(&[node("v16.20.2"), node("v22.11.0"), pin(TOOL_VERSIONS, "dev/app")]);

            assert_eq!(
                recorded.left_out(),
                [("unused-toolchains.node", "unused-toolchains.left-out.pin-unknown", 2000)],
                "{file:?}"
            );
        }
        let gone = FakeSmallFiles::with(&[("dev/app/.tool-versions", SmallFile::Missing)]);
        let recorded = World::new(gone).run(&[node("v16.20.2"), node("v22.11.0"), pin(TOOL_VERSIONS, "dev/app")]);
        assert_eq!(recorded.identities(), ["nvm/v16.20.2"]);
    }

    #[test]
    fn an_unreadable_or_missing_rustup_default_keeps_every_toolchain() {
        let left_out = [("unused-toolchains.rust", "unused-toolchains.left-out.default-unknown", 1000)];
        let old = toolchain("1.70.0-aarch64-apple-darwin");
        for (files, home) in [
            (FakeSmallFiles::with(&[(".rustup/settings.toml", SmallFile::Unreadable)]), rustup_home()),
            (FakeSmallFiles::default(), rustup_home()),
            (FakeSmallFiles::with(&[(".rustup/settings.toml", text("version = \"12\"\n"))]), rustup_home()),
            (
                FakeSmallFiles::with(&[(".rustup/settings.toml", text("default_toolchain = \"1.60.0\"\n"))]),
                rustup_home(),
            ),
            (FakeSmallFiles::with(&[(".rustup/settings.toml", text("default_toolchain = \"stable\"\n"))]), {
                Match { holds: Vec::new(), ..rustup_home() }
            }),
        ] {
            let world = World::new(files);

            let recorded = world.run(&[old.clone(), home.clone()]);

            assert_eq!(recorded.left_out(), left_out, "{home:?}");
            assert_eq!(world.files.reads(".rustup/settings.toml"), usize::from(!home.holds.is_empty()));
        }
    }

    #[test]
    fn the_switch_off_reads_no_nvm_file() {
        let world = World { node_on: false, ..World::new(mixed_files()) };

        let recorded = world.run(&mixed());

        assert_eq!(
            recorded.identities(),
            ["rustup/1.70.0-aarch64-apple-darwin", "rustup/nightly-2024-01-01-aarch64-apple-darwin"]
        );
        assert_eq!(recorded.left_out().len(), 1);
        for place in [".nvm/alias/default", ".nvm/alias/lts/hydrogen", "dev/web/.nvmrc"] {
            assert_eq!(world.files.reads(place), 0, "{place}");
        }
        assert_eq!(world.files.reads("dev/app/.tool-versions"), 1);
    }

    #[test]
    fn nothing_left_out_is_also_offered() {
        let world = World::new(mixed_files());
        let pointers: Vec<Match> = mixed().into_iter().filter(|found| found.inside.is_none()).collect();
        for found in mixed().into_iter().filter(|found| found.inside.is_some()) {
            let mut matches = pointers.clone();
            matches.push(found.clone());

            let recorded = world.run(&matches);

            assert!(recorded.findings.len() + recorded.left_out.len() <= 1, "{found:?}: {recorded:?}");
        }

        let all = world.run(&mixed());

        assert_eq!(all.identities(), MIXED_OFFERED);
        assert_eq!(all.left_out(), [("unused-toolchains.rust", "unused-toolchains.left-out.special", 1000)]);
    }

    #[test]
    fn a_file_named_like_a_newer_version_does_not_take_the_newest_slot() {
        let world = World::new(mixed_files());
        let file = Match { kind: EntryKind::File, inside: None, ..node("v99.0.0") };

        let recorded = world.run(&[node("v22.11.0"), file]);

        assert_eq!(recorded.identities(), Vec::<&str>::new());
        assert_eq!(recorded.left_out(), []);
    }

    #[test]
    fn an_alias_folder_is_not_read_as_an_alias() {
        let world = World::new(FakeSmallFiles::with(&[(".nvm/alias/lts", SmallFile::NotAFile)]));
        let lts = Match { kind: EntryKind::Folder, ..alias("lts") };

        let recorded = world.run(&[node("v16.20.2"), node("v22.11.0"), lts]);

        assert_eq!(recorded.identities(), ["nvm/v16.20.2"]);
        assert_eq!(world.files.reads(".nvm/alias/lts"), 0);
    }

    /// Records like `Recorded`, and cancels the scan once the first finding is pushed.
    struct CancelsAtFirstFinding {
        cancel: CancelToken,
        recorded: Recorded,
    }

    impl fresher_core::FindingSink for CancelsAtFirstFinding {
        fn push(&mut self, finding: NewFinding) {
            self.recorded.findings.push(finding);
            self.cancel.cancel();
        }

        fn left_out(&mut self, left_out: LeftOutItem) {
            self.recorded.left_out.push(left_out);
        }
    }

    #[test]
    fn a_cancelled_scan_stops_between_candidates() {
        let (world, module, cancel) =
            (World::new(mixed_files()), UnusedToolchainsModule::new().unwrap(), CancelToken::new());
        let mut out = CancelsAtFirstFinding { cancel: cancel.clone(), recorded: Recorded::default() };
        world.feed(&module, &cancel, &mixed(), &mut out);

        let finished = world.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(fresher_core::ModuleError::Cancelled));
        assert_eq!(out.recorded.identities(), [MIXED_OFFERED[0]]);
        assert_eq!(out.recorded.left_out, []);
    }

    #[test]
    fn a_walk_not_complete_leaves_every_offer_out() {
        let (world, module, cancel) =
            (World::new(mixed_files()), UnusedToolchainsModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        world.feed(&module, &cancel, &mixed(), &mut out);

        world.with_context(&module, &cancel, |cx| module.finish(&cx, &mut out)).unwrap();

        assert_eq!(out.findings, []);
        let reasons: Vec<&str> = out.left_out().iter().map(|(_, reason, _)| *reason).collect();
        assert_eq!(reasons.len(), MIXED_OFFERED.len() + 1);
        assert_eq!(reasons.iter().filter(|reason| reason.ends_with("partial-scan")).count(), MIXED_OFFERED.len());
    }
}

mod positive {
    use fresher_core::{ByteSize, DangerLevel, Removal, Subject, TextArg, Timestamp};
    use proptest::prelude::*;

    use super::{
        ACCESSED, MIXED_OFFERED, NVMRC, TOOL_VERSIONS, World, alias, mixed, mixed_files, node, path, pin, toolchain,
    };

    #[test]
    fn each_file_is_read_once_per_scan() {
        let world = World::new(mixed_files());
        let mut matches = mixed();
        matches.push(pin(NVMRC, "dev/app"));
        matches.push(pin(TOOL_VERSIONS, "dev/app"));

        let recorded = world.run(&matches);

        assert_eq!(recorded.identities(), MIXED_OFFERED);
        let reads = world.files.all_reads();
        assert_eq!(reads.len(), 7, "{reads:?}");
        assert!(reads.values().all(|count| *count == 1), "{reads:?}");
    }

    #[test]
    fn two_orders_of_matches_give_the_same_answers() {
        let world = World::new(mixed_files());
        let mut matches = mixed();
        let forward = world.run(&matches);

        matches.reverse();
        let backward = world.run(&matches);

        assert_eq!(forward, backward);
        assert_eq!(forward.identities(), MIXED_OFFERED);
    }

    #[test]
    fn nvm_aliases_follow_their_chain_and_lts() {
        let files = super::FakeSmallFiles::with(&[
            (".nvm/alias/default", super::text("work\n")),
            (".nvm/alias/work", super::text("lts/iron\n")),
            (".nvm/alias/lts/iron", super::text("v20.11.0\n")),
            ("dev/web/.nvmrc", super::text("node\n")),
        ]);
        let matches = [
            node("v14.21.3"),
            node("v20.11.0"),
            node("v22.11.0"),
            alias("default"),
            alias("work"),
            alias("lts/iron"),
            pin(NVMRC, "dev/web"),
        ];

        let recorded = World::new(files).run(&matches);

        assert_eq!(recorded.identities(), ["nvm/v14.21.3"]);
        assert_eq!(recorded.left_out, []);
    }

    #[test]
    fn a_finding_names_the_version_the_date_and_the_command() {
        let files =
            super::FakeSmallFiles::with(&[(".rustup/settings.toml", super::text("default_toolchain = \"stable\""))]);
        let recorded = World::new(files).run(&[
            toolchain("stable-aarch64-apple-darwin"),
            toolchain("1.70.0-aarch64-apple-darwin"),
            super::rustup_home(),
            node("v18.17.0"),
            node("v22.11.0"),
        ]);

        assert_eq!(recorded.findings.len(), 2, "{recorded:?}");
        let (node_version, rust) = (&recorded.findings[0], &recorded.findings[1]);
        assert_eq!(rust.category().as_str(), "unused-toolchains.rust");
        assert_eq!(rust.danger(), DangerLevel::Low);
        assert_eq!(rust.title().key().as_str(), "unused-toolchains.title.rust");
        assert_eq!(rust.title().args()["name"], TextArg::Text("1.70.0-aarch64-apple-darwin".into()));
        assert_eq!(rust.reason().key().as_str(), "unused-toolchains.reason.rust");
        assert_eq!(rust.reason().args()["date"], TextArg::Date(Timestamp::from_secs(ACCESSED)));
        assert_eq!(rust.reason().args()["size"], TextArg::Size(ByteSize::new(1000)));
        assert_eq!(
            rust.reason().args()["command"],
            TextArg::Text("rustup toolchain install 1.70.0-aarch64-apple-darwin".into())
        );
        assert_eq!(node_version.category().as_str(), "unused-toolchains.node");
        assert_eq!(node_version.danger(), DangerLevel::Medium);
        assert_eq!(node_version.title().key().as_str(), "unused-toolchains.title.node");
        assert_eq!(node_version.title().args()["name"], TextArg::Text("v18.17.0".into()));
        assert_eq!(node_version.reason().key().as_str(), "unused-toolchains.reason.node");
        assert_eq!(node_version.reason().args()["size"], TextArg::Size(ByteSize::new(2000)));
        assert_eq!(node_version.reason().args()["command"], TextArg::Text("nvm install 18.17.0".into()));
    }

    #[test]
    fn a_finding_moves_the_version_folder_to_the_trash() {
        let recorded = World::new(mixed_files()).run(&mixed());

        let finding = recorded.findings.iter().find(|finding| finding.key().identity() == MIXED_OFFERED[2]).unwrap();
        let folder = path(".rustup/toolchains/1.70.0-aarch64-apple-darwin");
        assert_eq!(finding.subject(), &Subject::Path(folder.clone()));
        assert_eq!(finding.removal(), &Removal::MoveToTrash(folder));
        assert_eq!(finding.sizes().logical.bytes(), 1000);
        assert_eq!(finding.sizes().allocated.bytes(), 1000 + 4096);
    }

    proptest! {
        #[test]
        fn the_verdict_never_depends_on_the_order_of_matches(matches in Just(mixed()).prop_shuffle()) {
            let world = World::new(mixed_files());

            let shuffled = world.run(&matches);

            prop_assert_eq!(shuffled.identities(), MIXED_OFFERED);
            prop_assert_eq!(shuffled, world.run(&mixed()));
        }
    }

    /// The module as built: its descriptor and interests.
    mod descriptor {
        use fresher_core::{Capability, DangerLevel, GuardedFolder, Module as _, ModuleGroup, Need, TextKey};

        use crate::UnusedToolchainsModule;

        fn module() -> UnusedToolchainsModule {
            UnusedToolchainsModule::new().unwrap()
        }

        #[test]
        fn the_module_is_in_the_development_group() {
            assert_eq!(module().descriptor().group(), ModuleGroup::Development);
            assert_eq!(module().descriptor().id().as_str(), "unused-toolchains");
        }

        #[test]
        fn the_categories_are_in_order_with_their_dangers() {
            let module = module();

            let categories: Vec<(&str, DangerLevel)> =
                module.descriptor().categories().iter().map(|c| (c.id().as_str(), c.danger())).collect();

            assert_eq!(
                categories,
                [("unused-toolchains.rust", DangerLevel::Low), ("unused-toolchains.node", DangerLevel::Medium)]
            );
        }

        #[test]
        fn both_categories_require_the_three_guarded_folders() {
            let needs: Vec<Need> = [GuardedFolder::Desktop, GuardedFolder::Documents, GuardedFolder::Downloads]
                .into_iter()
                .map(|folder| Need::required(Capability::Folder(folder)))
                .collect();

            for category in module().descriptor().categories() {
                assert_eq!(category.needs(), needs, "{}", category.id());
            }
        }

        #[test]
        fn the_node_switch_is_off_by_default() {
            let module = module();

            let settings = module.descriptor().settings();

            assert_eq!(settings.len(), 1);
            assert_eq!(settings[0].key().as_str(), "unused-toolchains.node-versions");
            assert_eq!(settings[0].default(), &fresher_core::SettingValue::Switch(false));
        }

        #[test]
        fn the_five_left_out_reasons_are_declared_in_order() {
            let module = module();

            let problems: Vec<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

            assert_eq!(
                problems,
                [
                    "unused-toolchains.left-out.special",
                    "unused-toolchains.left-out.not-yours",
                    "unused-toolchains.left-out.partial-scan",
                    "unused-toolchains.left-out.default-unknown",
                    "unused-toolchains.left-out.pin-unknown",
                ]
            );
        }

        #[test]
        fn the_interests_are_the_tables() {
            let module = module();

            assert_eq!(module.interests().len(), module.toolchains.kinds().len());
            assert_eq!(module.interests().len(), 10);
        }
    }
}
