//! Ollama's store through `on_match` and `finish`: plain matches of its six interests and manifests read through a
//! fake small-file reader.

use std::path::PathBuf;

use fresher_core::{ByteSize, EntryFacts, EntryKind, Match, Sizes, SmallFile, Timestamp};

use super::{ACCESSED, CHANGED, FakeSmallFiles, Recorded, Scan, path};
use crate::ollama::test_support::{digest, listed};

/// The interests of the Ollama row: after the three of the other rows.
const STORE: usize = 3;
const HOSTS: usize = 4;
const NAMESPACES: usize = 5;
const MODELS: usize = 6;
const MANIFESTS: usize = 7;
const BLOBS: usize = 8;

const OLLAMA: &str = "local-ai-models.ollama";

/// A match of `interest` at `/home/.ollama/models/<place>`, the user's, with its own times.
fn entry(interest: usize, place: &str, kind: EntryKind, bytes: u64) -> Match {
    Match {
        interest,
        path: in_store(place),
        kind,
        sizes: Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes.next_multiple_of(4096))),
        modified: Some(Timestamp::from_secs(CHANGED)),
        changed: None,
        accessed: Some(Timestamp::from_secs(ACCESSED)),
        inside: None,
        siblings: Vec::new(),
        holds: Vec::new(),
        marker: None,
        facts: EntryFacts::REMOVABLE,
    }
}

fn in_store(place: &str) -> PathBuf {
    let store = path(".ollama/models");
    if place.is_empty() { store } else { store.join(place) }
}

/// The place of the layer of the digest of `byte`.
fn blob(byte: u8) -> String {
    format!("blobs/{}", digest(byte).blob_name())
}

/// A manifest naming the config `config` and the layers `layers`, each with its listed size.
fn manifest_text(config: u8, layers: &[u8]) -> String {
    let layer = |byte: u8| {
        let name = digest(byte).blob_name();
        let hex = name.trim_start_matches("sha256-");
        format!(
            r#"{{"mediaType":"application/vnd.ollama.image.model","digest":"sha256:{hex}","size":{}}}"#,
            listed(byte)
        )
    };
    let layers: Vec<String> = layers.iter().map(|byte| layer(*byte)).collect();
    format!(r#"{{"schemaVersion":2,"config":{},"layers":[{}]}}"#, layer(config), layers.join(","))
}

/// The matches of an Ollama store and the files its manifests hold.
#[derive(Clone, Debug)]
struct Store {
    matches: Vec<Match>,
    files: Vec<(PathBuf, SmallFile)>,
}

impl Store {
    /// The store holding its layout, and no model.
    fn new() -> Self {
        let store = Match { holds: vec!["manifests".into(), "blobs".into()], ..entry(STORE, "", EntryKind::Folder, 0) };
        Self { matches: vec![store], files: Vec::new() }
    }

    /// With the manifest of `host/namespace/model:tag` naming `config` and `layers`, and its folders.
    fn model(mut self, name: [&str; 4], config: u8, layers: &[u8]) -> Self {
        let [host, namespace, model, tag] = name;
        let folders = [
            (HOSTS, format!("manifests/{host}")),
            (NAMESPACES, format!("manifests/{host}/{namespace}")),
            (MODELS, format!("manifests/{host}/{namespace}/{model}")),
        ];
        for (interest, place) in folders {
            if !self.matches.iter().any(|found| found.path == in_store(&place)) {
                self.matches.push(entry(interest, &place, EntryKind::Folder, 0));
            }
        }
        let place = format!("manifests/{host}/{namespace}/{model}/{tag}");
        let text = manifest_text(config, layers);
        self.matches.push(entry(MANIFESTS, &place, EntryKind::File, u64::try_from(text.len()).unwrap()));
        self.files.push((in_store(&place), SmallFile::Text(text)));
        self
    }

    /// With the model `model:tag` of Ollama's own library.
    fn library(self, model: &str, tag: &str, config: u8, layers: &[u8]) -> Self {
        self.model(["registry.ollama.ai", "library", model, tag], config, layers)
    }

    fn layers(mut self, bytes: &[u8]) -> Self {
        self.matches.extend(bytes.iter().map(|byte| entry(BLOBS, &blob(*byte), EntryKind::File, listed(*byte))));
        self
    }

    fn with(mut self, found: Match) -> Self {
        self.matches.push(found);
        self
    }

    /// With the file of the manifest at `place` holding `file` instead.
    fn file(mut self, place: &str, file: SmallFile) -> Self {
        let at = in_store(place);
        self.files.retain(|(path, _)| *path != at);
        self.files.push((at, file));
        self
    }

    fn scan(&self) -> Scan {
        Scan::files(FakeSmallFiles::with(self.files.clone()))
    }

    fn run(&self) -> Recorded {
        self.scan().run(&self.matches)
    }

    /// The logical bytes of every manifest, layer and partial download.
    fn bytes(&self) -> u64 {
        let counted = |found: &&Match| {
            let place = found.path.strip_prefix(in_store("")).unwrap().to_string_lossy();
            found.interest == MANIFESTS || (found.interest == BLOBS && place.starts_with("blobs/sha256-"))
        };
        self.matches.iter().filter(counted).map(|found| found.sizes.logical.bytes()).sum()
    }
}

/// `llama3:latest` and `llama3:8b` (the same files), `mistral:7b`, and `chat:1` made from `mistral:7b`.
fn mixed() -> Store {
    Store::new()
        .library("llama3", "latest", 1, &[2, 3])
        .library("llama3", "8b", 1, &[2, 3])
        .library("mistral", "7b", 4, &[5])
        .library("chat", "1", 6, &[5, 7])
        .layers(&[1, 2, 3, 4, 5, 6, 7])
}

mod negative {
    use fresher_core::{CancelToken, EntryFacts, EntryKind, Match, Module as _, ModuleError, SmallFile};

    use super::{BLOBS, HOSTS, MODELS, NAMESPACES, OLLAMA, STORE, Store, blob, entry, in_store, mixed};
    use crate::LocalAiModelsModule;
    use crate::tests::{Recorded, Scan, repo};

    #[test]
    fn a_store_without_its_layout_is_not_claimed_and_no_manifest_is_read() {
        let mut no_blobs = mixed();
        no_blobs.matches[0].holds = vec!["manifests".into()];
        let mut gone = mixed();
        gone.matches.retain(|found| found.interest != STORE);

        for store in [no_blobs, gone] {
            let scan = store.scan();

            assert_eq!(scan.run(&store.matches), Recorded::default());
            assert!(scan.files.reads().is_empty());
        }
    }

    #[test]
    fn a_partial_download_leaves_out_the_store_with_its_sizes() {
        let partial = format!("{}-partial", blob(9));
        let store = mixed().with(entry(BLOBS, &partial, EntryKind::File, 777));

        let recorded = store.run();

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [(OLLAMA, "local-ai-models.left-out.ollama-download", store.bytes())]);
        assert!(store.bytes() > 777);
    }

    #[test]
    fn a_manifest_not_read_leaves_out_the_store() {
        let place = "manifests/registry.ollama.ai/library/mistral/7b";
        for file in [
            SmallFile::Text("{] not json".into()),
            SmallFile::Text(r#"{"schemaVersion":1,"config":{},"layers":[]}"#.into()),
            SmallFile::TooLong,
            SmallFile::NotText,
            SmallFile::NotAFile,
            SmallFile::Missing,
            SmallFile::Unreadable,
        ] {
            let store = mixed().file(place, file.clone());

            let recorded = store.run();

            assert_eq!(recorded.findings, [], "{file:?}");
            assert_eq!(
                recorded.left_out(),
                [(OLLAMA, "local-ai-models.left-out.ollama-unreadable", store.bytes())],
                "{file:?}"
            );
        }
    }

    #[test]
    fn a_folder_of_the_manifests_tree_not_the_users_leaves_out_the_store() {
        let mut store = mixed();
        let namespace = in_store("manifests/registry.ollama.ai/library");
        let theirs = store.matches.iter_mut().find(|found| found.path == namespace).unwrap();
        theirs.facts = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };

        let recorded = store.run();

        assert_eq!(recorded.findings, []);
        assert_eq!(recorded.left_out(), [(OLLAMA, "local-ai-models.left-out.not-yours", store.bytes())]);
    }

    #[test]
    fn a_link_in_the_manifests_tree_leaves_out_the_store_and_no_manifest_is_read() {
        for (interest, place) in [
            (HOSTS, "manifests/elsewhere.example"),
            (NAMESPACES, "manifests/registry.ollama.ai/shared"),
            (MODELS, "manifests/registry.ollama.ai/library/linked"),
        ] {
            let store = mixed().with(entry(interest, place, EntryKind::Symlink, 40));
            let scan = store.scan();

            let recorded = scan.run(&store.matches);

            assert_eq!(recorded.findings, [], "{place}");
            assert_eq!(
                recorded.left_out(),
                [(OLLAMA, "local-ai-models.left-out.ollama-unreadable", store.bytes())],
                "{place}"
            );
            assert!(scan.files.reads().is_empty(), "{place}");
        }
    }

    #[test]
    fn a_hidden_folder_in_the_manifests_tree_leaves_out_the_store_and_no_manifest_is_read() {
        for (interest, place) in [
            (HOSTS, "manifests/.cache"),
            (NAMESPACES, "manifests/registry.ollama.ai/.hidden"),
            (MODELS, "manifests/registry.ollama.ai/library/.llama3"),
        ] {
            let store = mixed().with(entry(interest, place, EntryKind::Folder, 0));
            let scan = store.scan();

            let recorded = scan.run(&store.matches);

            assert_eq!(recorded.findings, [], "{place}");
            assert_eq!(
                recorded.left_out(),
                [(OLLAMA, "local-ai-models.left-out.ollama-unreadable", store.bytes())],
                "{place}"
            );
            assert!(scan.files.reads().is_empty(), "{place}");
        }
    }

    #[test]
    fn a_hidden_file_in_the_manifests_tree_changes_nothing() {
        let store = mixed().with(entry(HOSTS, "manifests/.DS_Store", EntryKind::File, 6148));

        assert_eq!(store.run(), mixed().run());
    }

    #[test]
    fn a_store_kept_before_reading_reads_no_manifest() {
        let mut theirs = mixed();
        theirs.matches[1].facts = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
        let partial = mixed().with(entry(BLOBS, &format!("{}-partial", blob(9)), EntryKind::File, 5));

        for store in [theirs, partial] {
            let scan = store.scan();

            assert_eq!(scan.run(&store.matches).left_out.len(), 1);
            assert!(scan.files.reads().is_empty());
        }
    }

    #[test]
    fn an_incomplete_model_is_left_out_beside_an_offered_one() {
        let store = Store::new().library("whole", "1", 1, &[2]).library("broken", "1", 3, &[4]).layers(&[1, 2, 3]);

        let recorded = store.run();

        assert_eq!(recorded.identities(), ["ollama/registry.ollama.ai/library/whole/1"]);
        let manifest = store.matches.iter().find(|found| found.path.ends_with("broken/1")).unwrap();
        let kept = manifest.sizes.logical.bytes() + super::listed(3);
        assert_eq!(recorded.left_out(), [(OLLAMA, "local-ai-models.left-out.ollama-incomplete", kept)]);
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

    /// Runs `matches` through a new module with `scan`, cancelling at the first finding.
    fn cancelled_at_first_finding(scan: &Scan, matches: &[Match]) -> (Result<(), ModuleError>, Recorded) {
        let (module, cancel) = (LocalAiModelsModule::new().unwrap(), CancelToken::new());
        let mut out = CancelsAtFirstFinding { cancel: cancel.clone(), recorded: Recorded::default() };
        for found in matches {
            scan.with_context(&module, &cancel, |cx| module.on_match(found, &cx, &mut out).unwrap());
        }
        let finished = scan.with_context(&module, &cancel, |cx| module.finish(&cx.complete_walk(), &mut out));
        (finished, out.recorded)
    }

    #[test]
    fn a_scan_cancelled_before_the_manifests_are_read_reads_none() {
        let store = mixed().with(repo("models--gpt2"));
        let scan = store.scan();

        let (finished, recorded) = cancelled_at_first_finding(&scan, &store.matches);

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(recorded.identities(), ["hugging-face/models--gpt2"]);
        assert!(scan.files.reads().is_empty());
    }

    #[test]
    fn a_scan_cancelled_between_models_tells_no_more() {
        let store = mixed();
        let scan = store.scan();

        let (finished, recorded) = cancelled_at_first_finding(&scan, &store.matches);

        assert_eq!(finished, Err(ModuleError::Cancelled));
        assert_eq!(recorded.identities(), ["ollama/registry.ollama.ai/library/chat/1"]);
    }
}

mod positive {
    use fresher_core::{ByteSize, DangerLevel, Removal, Subject, TextArg, Timestamp};

    use super::{ACCESSED, OLLAMA, Store, blob, in_store, mixed};
    use crate::ollama::test_support::listed;

    #[test]
    fn a_model_is_one_finding_of_its_manifests_and_its_unshared_layers() {
        let store = mixed();

        let recorded = store.run();

        assert_eq!(
            recorded.identities(),
            [
                "ollama/registry.ollama.ai/library/chat/1",
                "ollama/registry.ollama.ai/library/llama3/8b",
                "ollama/registry.ollama.ai/library/mistral/7b",
            ]
        );
        assert_eq!(recorded.left_out, []);
        assert!(recorded.findings.iter().all(|finding| finding.danger() == DangerLevel::Medium));
        let llama = &recorded.findings[1];
        let manifest = |tag: &str| in_store(&format!("manifests/registry.ollama.ai/library/llama3/{tag}"));
        assert_eq!(llama.category().as_str(), OLLAMA);
        assert_eq!(llama.danger(), DangerLevel::Medium);
        assert_eq!(llama.subject(), &Subject::Path(manifest("8b")));
        let Removal::MoveTogether(parts) = llama.removal() else { unreachable!("{:?}", llama.removal()) };
        assert_eq!(
            parts.all(),
            [manifest("8b"), manifest("latest"), in_store(&blob(1)), in_store(&blob(2)), in_store(&blob(3))]
        );
        let manifests: u64 = store
            .matches
            .iter()
            .filter(|found| found.path == manifest("8b") || found.path == manifest("latest"))
            .map(|found| found.sizes.logical.bytes())
            .sum();
        assert_eq!(llama.sizes().logical.bytes(), manifests + listed(1) + listed(2) + listed(3));
        assert_eq!(llama.title().key().as_str(), "local-ai-models.title");
        assert_eq!(llama.title().args()["name"], TextArg::Text("llama3:8b, llama3:latest".into()));
        assert_eq!(llama.reason().key().as_str(), "local-ai-models.reason.ollama");
        assert_eq!(llama.reason().args()["name"], TextArg::Text("llama3:8b".into()));
        assert_eq!(llama.reason().args()["date"], TextArg::Date(Timestamp::from_secs(ACCESSED)));
        assert_eq!(llama.reason().args()["size"], TextArg::Size(llama.sizes().logical));
    }

    #[test]
    fn a_model_sharing_layers_with_one_other_names_it_and_its_whole_size() {
        let recorded = mixed().run();

        let chat = &recorded.findings[0];

        assert_eq!(chat.reason().key().as_str(), "local-ai-models.reason.ollama-shared");
        assert_eq!(chat.reason().args()["other"], TextArg::Text("mistral:7b".into()));
        assert!(!chat.reason().args().contains_key("name"));
        assert_eq!(chat.reason().args()["listed"], TextArg::Size(ByteSize::new(listed(5) + listed(6) + listed(7))));
        let Removal::MoveTogether(parts) = chat.removal() else { unreachable!("{:?}", chat.removal()) };
        assert!(!parts.all().contains(&in_store(&blob(5))));
    }

    #[test]
    fn a_model_sharing_layers_with_several_names_the_first_and_counts_the_others() {
        let store = Store::new()
            .library("base", "1", 1, &[2])
            .model(["registry.ollama.ai", "me", "tuned", "1"], 3, &[2])
            .model(["hf.co", "org", "repo", "q4"], 4, &[2])
            .layers(&[1, 2, 3, 4]);

        let recorded = store.run();

        let titles: Vec<&TextArg> = recorded.findings.iter().map(|finding| &finding.title().args()["name"]).collect();
        assert_eq!(
            titles,
            [
                &TextArg::Text("hf.co/org/repo:q4".into()),
                &TextArg::Text("base:1".into()),
                &TextArg::Text("me/tuned:1".into()),
            ]
        );
        let base = &recorded.findings[1];
        assert_eq!(base.reason().key().as_str(), "local-ai-models.reason.ollama-shared-more");
        assert_eq!(base.reason().args()["other"], TextArg::Text("hf.co/org/repo:q4".into()));
        assert_eq!(base.reason().args()["count"], TextArg::Count(1));
    }

    #[test]
    fn each_manifest_is_read_once_and_nothing_else_is_read() {
        let store = mixed();
        let scan = store.scan();

        scan.run(&store.matches);

        let reads = scan.files.reads();
        let manifests: Vec<_> = store.files.iter().map(|(path, _)| path.clone()).collect();
        assert_eq!(reads.keys().cloned().collect::<Vec<_>>(), {
            let mut sorted = manifests;
            sorted.sort();
            sorted
        });
        assert!(reads.values().all(|count| *count == 1));
    }

    #[test]
    fn a_reversed_scan_decides_the_same() {
        let store = mixed();
        let walked = store.run();

        let mut reversed = store.matches.clone();
        reversed.reverse();

        assert_eq!(store.scan().run(&reversed), walked);
    }
}
