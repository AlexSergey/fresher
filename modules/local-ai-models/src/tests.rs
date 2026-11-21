//! The module as built, and the module through `on_match` and `finish`, on a recording sink, with every system fact
//! faked and plain `Match` values in a home at `/home`. Ollama's store: `ollama.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use fresher_core::{
    ByteSize, CancelToken, EntryFacts, EntryKind, Inside, LeftOutItem, Match, Module as _, ModuleContext, NewFinding,
    ReadError, SettingsSchema, Sizes, SmallFile, SmallFiles, StoredSettings, SystemFacts, SystemPorts, Timestamp,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders, FakeRunningApps};

use crate::LocalAiModelsModule;

const NOW: u64 = 1_767_225_600;
const CHANGED: u64 = 1_760_000_000;
const ACCESSED: u64 = 1_765_000_000;
const LM_STUDIO: &str = "ai.elementlabs.lmstudio";

/// The interest of each row of the shipped table, in its order.
const HUGGING_FACE: usize = 0;
const LM_STUDIO_NOW: usize = 1;
const LM_STUDIO_OLD: usize = 2;

/// The small files of a scan, by path, each read counted.
#[derive(Debug, Default)]
struct FakeSmallFiles {
    files: BTreeMap<PathBuf, SmallFile>,
    reads: Mutex<BTreeMap<PathBuf, usize>>,
}

impl FakeSmallFiles {
    fn with(files: impl IntoIterator<Item = (PathBuf, SmallFile)>) -> Self {
        Self { files: files.into_iter().collect(), ..Self::default() }
    }

    /// How many times each file was read.
    fn reads(&self) -> BTreeMap<PathBuf, usize> {
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

fn path(place: &str) -> PathBuf {
    Path::new("/home").join(place)
}

/// A folder of the user's of `bytes` logical bytes at `/home/<place>`, read in full, matched by `interest`, holding
/// `holds`.
fn folder(interest: usize, place: &str, holds: &[&str], bytes: u64) -> Match {
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
        holds: holds.iter().map(|name| (*name).to_owned()).collect(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

/// A Hugging Face repository folder named `name`, holding its blobs and snapshots.
fn repo(name: &str) -> Match {
    folder(HUGGING_FACE, &format!(".cache/huggingface/hub/{name}"), &["blobs", "snapshots"], 1000)
}

/// An LM Studio model folder `publisher/model` in its current place.
fn lm_model(publisher: &str, model: &str) -> Match {
    folder(LM_STUDIO_NOW, &format!(".lmstudio/models/{publisher}/{model}"), &[], 2000)
}

/// An LM Studio model folder `publisher/model` in its older place.
fn lm_old_model(publisher: &str, model: &str) -> Match {
    folder(LM_STUDIO_OLD, &format!(".cache/lm-studio/models/{publisher}/{model}"), &[], 3000)
}

/// A model of every row and kind, two models of one folder name, a folder holding a socket, and entries not claimed.
fn mixed() -> Vec<Match> {
    vec![
        repo("models--google-bert--bert-base-uncased"),
        repo("models--gpt2"),
        repo("datasets--squad"),
        repo("spaces--owner--demo"),
        folder(HUGGING_FACE, ".cache/huggingface/hub/.locks", &[], 10),
        holding_a_socket(repo("models--owner--socket")),
        lm_model("lmstudio-community", "Qwen2.5-7B-Instruct-GGUF"),
        lm_model("bartowski", "Qwen2.5-7B-Instruct-GGUF"),
        Match { kind: EntryKind::File, ..lm_model("bartowski", "loose.gguf") },
        lm_old_model("TheBloke", "Mistral-7B-GGUF"),
    ]
}

/// `found` with a socket inside.
fn holding_a_socket(found: Match) -> Match {
    Match { inside: found.inside.map(|inside| Inside { special: true, ..inside }), ..found }
}

fn running(id: &str) -> fresher_core::RunningApp {
    fresher_testing::running_app("App", &[id]).unwrap()
}

/// The system a scan runs with.
struct Scan {
    running: Arc<FakeRunningApps>,
    files: Arc<FakeSmallFiles>,
}

impl Scan {
    fn new() -> Self {
        Self::running(FakeRunningApps::default())
    }

    fn running(running: FakeRunningApps) -> Self {
        Self { running: Arc::new(running), files: Arc::new(FakeSmallFiles::default()) }
    }

    fn files(files: FakeSmallFiles) -> Self {
        Self { running: Arc::new(FakeRunningApps::default()), files: Arc::new(files) }
    }

    fn run(&self, matches: &[Match]) -> Recorded {
        self.run_as(matches, false)
    }

    /// Hands `matches` to a new module in their order, then calls `finish`; every context restored when `restored`.
    fn run_as(&self, matches: &[Match], restored: bool) -> Recorded {
        let (module, cancel) = (LocalAiModelsModule::new().unwrap(), CancelToken::new());
        let mut out = Recorded::default();
        for found in matches {
            self.with_context(&module, &cancel, restored, |cx| module.on_match(found, &cx, &mut out).unwrap());
        }
        self.with_context(&module, &cancel, restored, |cx| module.finish(&cx.complete_walk(), &mut out).unwrap());
        out
    }

    fn with_context<T>(
        &self,
        module: &LocalAiModelsModule,
        cancel: &CancelToken,
        restored: bool,
        call: impl FnOnce(ModuleContext<'_>) -> T,
    ) -> T {
        let system = SystemFacts::new(&SystemPorts {
            known_folders: Arc::new(FakeKnownFolders::home_layout(Path::new("/home"))),
            apps: Arc::new(FakeInstalledApps::new(Vec::new())),
            running: Arc::<FakeRunningApps>::clone(&self.running),
            installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
            files: Arc::<FakeSmallFiles>::clone(&self.files),
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

mod negative {
    use fresher_core::{CancelToken, EntryFacts, Match, Module as _, ModuleError};
    use fresher_testing::FakeRunningApps;

    use super::{LM_STUDIO, Recorded, Scan, folder, holding_a_socket, lm_model, mixed, repo, running};
    use crate::LocalAiModelsModule;

    #[test]
    fn a_match_of_no_interest_is_not_claimed() {
        let stray = Match { interest: 99, ..folder(0, ".ollama/models/blobs", &["blobs", "snapshots"], 100) };

        assert_eq!(Scan::new().run(&[stray]), Recorded::default());
    }

    #[test]
    fn nothing_left_out_is_also_offered() {
        let scan = Scan::running(FakeRunningApps::new(vec![running(LM_STUDIO)]));
        for found in mixed() {
            let recorded = scan.run(std::slice::from_ref(&found));

            assert!(recorded.findings.len() + recorded.left_out.len() <= 1, "{found:?}: {recorded:?}");
        }

        let all = scan.run(&mixed());

        assert_eq!(
            all.identities(),
            [
                "hugging-face/datasets--squad",
                "hugging-face/models--google-bert--bert-base-uncased",
                "hugging-face/models--gpt2",
            ]
        );
        assert_eq!(all.left_out.len(), 4);
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
        let (scan, module, cancel) = (Scan::new(), LocalAiModelsModule::new().unwrap(), CancelToken::new());
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
        let (scan, module, cancel) = (Scan::new(), LocalAiModelsModule::new().unwrap(), CancelToken::new());
        let mut out = CancelsAtFirstFinding { cancel: cancel.clone(), recorded: Recorded::default() };
        for found in mixed() {
            scan.with_context(&module, &cancel, false, |cx| module.on_match(&found, &cx, &mut out).unwrap());
        }

        let finished = scan.with_context(&module, &cancel, false, |cx| module.finish(&cx.complete_walk(), &mut out));

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(out.recorded.identities(), ["hugging-face/datasets--squad"]);
        assert_eq!(out.recorded.left_out, []);
    }

    #[test]
    fn every_reason_is_reported_with_its_key_the_category_of_its_name_and_its_sizes() {
        let socket = holding_a_socket(repo("datasets--squad"));
        let theirs =
            Match { facts: EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE }, ..repo("models--gpt2") };
        let scan = Scan::running(FakeRunningApps::new(vec![running(LM_STUDIO)]));

        let recorded = scan.run(&[socket, theirs, lm_model("a", "b")]);

        assert_eq!(recorded.findings, []);
        assert_eq!(
            recorded.left_out(),
            [
                ("local-ai-models.hugging-face-datasets", "local-ai-models.left-out.special", 1000),
                ("local-ai-models.hugging-face-models", "local-ai-models.left-out.not-yours", 1000),
                ("local-ai-models.lm-studio", "local-ai-models.left-out.app-open", 2000),
            ]
        );
        assert_eq!(scan.running.calls(), 1);
    }

    #[test]
    fn lm_studio_s_models_are_left_out_when_the_running_apps_cannot_be_listed() {
        let scan = Scan::running(FakeRunningApps::failing("no process list"));

        let recorded = scan.run(&[lm_model("a", "b"), repo("models--gpt2")]);

        assert_eq!(recorded.identities(), ["hugging-face/models--gpt2"]);
        assert_eq!(recorded.left_out(), [("local-ai-models.lm-studio", "local-ai-models.left-out.no-system", 2000)]);
    }

    #[test]
    fn the_running_apps_are_not_read_when_no_candidate_names_an_app() {
        let scan = Scan::new();

        let recorded = scan.run(&[repo("models--gpt2"), repo("datasets--squad")]);

        assert_eq!(recorded.findings.len(), 2);
        assert_eq!(scan.running.calls(), 0);
    }
}

mod positive {
    use fresher_core::{
        ByteSize, DangerLevel, Module as _, ModuleGroup, Removal, Subject, TextArg, TextKey, Timestamp,
    };
    use fresher_testing::FakeRunningApps;
    use proptest::prelude::*;

    use super::{ACCESSED, LM_STUDIO, Scan, lm_model, lm_old_model, mixed, path, repo, running};
    use crate::LocalAiModelsModule;

    fn module() -> LocalAiModelsModule {
        LocalAiModelsModule::new().expect("the module is built")
    }

    #[test]
    fn the_module_is_in_the_development_group() {
        assert_eq!(module().descriptor().group(), ModuleGroup::Development);
    }

    #[test]
    fn the_categories_come_in_order_with_their_dangers() {
        let module = module();

        let categories: Vec<(&str, DangerLevel)> = module
            .descriptor()
            .categories()
            .iter()
            .map(|category| (category.id().as_str(), category.danger()))
            .collect();

        assert_eq!(
            categories,
            [
                ("local-ai-models.hugging-face-models", DangerLevel::Low),
                ("local-ai-models.hugging-face-datasets", DangerLevel::Low),
                ("local-ai-models.lm-studio", DangerLevel::Medium),
                ("local-ai-models.ollama", DangerLevel::Medium),
            ]
        );
    }

    #[test]
    fn the_left_out_reasons_are_the_seven_in_order() {
        let module = module();

        let problems: Vec<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        assert_eq!(
            problems,
            [
                "local-ai-models.left-out.not-yours",
                "local-ai-models.left-out.special",
                "local-ai-models.left-out.app-open",
                "local-ai-models.left-out.no-system",
                "local-ai-models.left-out.ollama-unreadable",
                "local-ai-models.left-out.ollama-download",
                "local-ai-models.left-out.ollama-incomplete",
            ]
        );
    }

    #[test]
    fn the_module_has_one_interest_per_store_and_six_for_ollama() {
        assert_eq!(module().interests().len(), 3 + 6);
    }

    #[test]
    fn two_orders_of_matches_give_the_same_answers() {
        let scan = Scan::running(FakeRunningApps::new(vec![running("com.example.Editor")]));
        let mut matches = mixed();
        let forward = scan.run(&matches);

        matches.reverse();
        let backward = scan.run(&matches);

        assert_eq!(forward, backward);
        assert_eq!(
            forward.identities(),
            [
                "hugging-face/datasets--squad",
                "hugging-face/models--google-bert--bert-base-uncased",
                "hugging-face/models--gpt2",
                "lm-studio-old/TheBloke/Mistral-7B-GGUF",
                "lm-studio/bartowski/Qwen2.5-7B-Instruct-GGUF",
                "lm-studio/lmstudio-community/Qwen2.5-7B-Instruct-GGUF",
            ]
        );
        assert_eq!(
            forward.left_out(),
            [("local-ai-models.hugging-face-models", "local-ai-models.left-out.special", 1000)]
        );
    }

    #[test]
    fn a_finding_names_the_model_the_date_and_the_size() {
        let recorded = Scan::new().run(&[repo("models--google-bert--bert-base-uncased")]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = recorded.findings.first().unwrap();
        assert_eq!(finding.category().as_str(), "local-ai-models.hugging-face-models");
        assert_eq!(finding.danger(), DangerLevel::Low);
        assert_eq!(finding.title().key().as_str(), "local-ai-models.title");
        assert_eq!(finding.title().args()["name"], TextArg::Text("google-bert/bert-base-uncased".into()));
        assert_eq!(finding.reason().key().as_str(), "local-ai-models.reason.hugging-face");
        assert_eq!(finding.reason().args()["date"], TextArg::Date(Timestamp::from_secs(ACCESSED)));
        assert_eq!(finding.reason().args()["size"], TextArg::Size(ByteSize::new(1000)));

        let recorded = Scan::new().run(&[lm_old_model("TheBloke", "Mistral-7B-GGUF")]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = recorded.findings.first().unwrap();
        assert_eq!(finding.category().as_str(), "local-ai-models.lm-studio");
        assert_eq!(finding.danger(), DangerLevel::Medium);
        assert_eq!(finding.title().args()["name"], TextArg::Text("TheBloke/Mistral-7B-GGUF".into()));
        assert_eq!(finding.reason().key().as_str(), "local-ai-models.reason.lm-studio");
        assert_eq!(finding.reason().args()["size"], TextArg::Size(ByteSize::new(3000)));
    }

    #[test]
    fn a_finding_moves_the_model_folder_to_the_trash() {
        let recorded = Scan::new().run(&[lm_model("lmstudio-community", "Qwen2.5-7B-Instruct-GGUF")]);

        assert_eq!(recorded.findings.len(), 1, "{recorded:?}");
        let finding = recorded.findings.first().unwrap();
        let folder = path(".lmstudio/models/lmstudio-community/Qwen2.5-7B-Instruct-GGUF");
        assert_eq!(finding.key().identity(), "lm-studio/lmstudio-community/Qwen2.5-7B-Instruct-GGUF");
        assert_eq!(finding.subject(), &Subject::Path(folder.clone()));
        assert_eq!(finding.removal(), &Removal::MoveToTrash(folder));
        assert_eq!(finding.sizes().logical.bytes(), 2000);
        assert_eq!(finding.sizes().allocated.bytes(), 2000 + 4096);
    }

    proptest! {
        #[test]
        fn a_restored_context_decides_the_same(
            picked in proptest::sample::subsequence((0..mixed().len()).collect::<Vec<_>>(), 0..=mixed().len()),
            lm_studio_open: bool,
        ) {
            let apps = if lm_studio_open { vec![running(LM_STUDIO)] } else { Vec::new() };
            let scan = Scan::running(FakeRunningApps::new(apps));
            let all = mixed();
            let matches: Vec<_> = picked.iter().filter_map(|at| all.get(*at).cloned()).collect();

            prop_assert_eq!(scan.run_as(&matches, true), scan.run_as(&matches, false));
        }
    }
}

mod ollama;
