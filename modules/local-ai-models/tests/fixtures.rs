//! The local-ai-models module on its fixture homes: what it must not claim or must keep first, then what it must
//! offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-local-ai-models` rewrites
//! `fixtures/golden/local-ai-models.json` and `fixtures/golden/local-ai-models-ollama.json`; the diff is reviewed.

#[cfg(test)]
mod support;

use fresher_testing::ModuleRun;

/// Each left-out of the run as its category and reason, sorted.
fn left_out(run: &ModuleRun) -> Vec<(&str, &str)> {
    let mut reported: Vec<(&str, &str)> =
        run.left_out.iter().map(|item| (item.category.as_str(), item.reason.as_str())).collect();
    reported.sort_unstable();
    reported
}

/// The files of a fixture home as the disk holds them.
#[cfg(test)]
mod files {
    use std::path::PathBuf;

    use fresher_testing::FixtureTree;

    /// The logical bytes of the files at `places` in `tree`, as the disk holds them.
    pub(crate) fn bytes_of(tree: &FixtureTree, places: &[PathBuf]) -> u64 {
        places.iter().map(|place| std::fs::symlink_metadata(tree.root().join(place)).unwrap().len()).sum()
    }

    /// Every file of the Ollama store of `tree`, relative to its root.
    pub(crate) fn store_files(tree: &FixtureTree) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut folders = vec![tree.root().join(".ollama/models")];
        while let Some(folder) = folders.pop() {
            for entry in std::fs::read_dir(folder).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    folders.push(path);
                } else {
                    files.push(path.strip_prefix(tree.root()).unwrap().to_path_buf());
                }
            }
        }
        files
    }
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding, Removal};
    use fresher_testing::FakeRunningApps;

    use crate::files::{bytes_of, store_files};
    use crate::left_out;
    use crate::support::{kept, kept_with, not_claimed, ollama};

    /// A space, locks, a version file, a repository without its sign, a name of four parts, a link, a loose file, a
    /// hidden publisher, Ollama's folder without `manifests/` and a model file in a project.
    #[test]
    fn nothing_in_the_not_claimed_home_is_offered_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn the_kept_home_offers_nothing_and_leaves_out_each_with_its_reason() {
        let (_tree, run) = kept();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                ("local-ai-models.hugging-face-models", "local-ai-models.left-out.not-yours"),
                ("local-ai-models.hugging-face-models", "local-ai-models.left-out.special"),
                ("local-ai-models.lm-studio", "local-ai-models.left-out.app-open"),
            ]
        );
    }

    #[test]
    fn open_apps_that_cannot_be_listed_keep_the_lm_studio_model_and_change_nothing_else() {
        let (_tree, run) = kept_with(FakeRunningApps::failing("no process list"));

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&run),
            [
                ("local-ai-models.hugging-face-models", "local-ai-models.left-out.not-yours"),
                ("local-ai-models.hugging-face-models", "local-ai-models.left-out.special"),
                ("local-ai-models.lm-studio", "local-ai-models.left-out.no-system"),
            ]
        );
    }

    /// A file below a tag folder, hidden files in the manifests' tree, a layer no manifest names, blobs off the
    /// shape; and a store holding only `manifests/`.
    #[test]
    fn nothing_in_the_ollama_not_claimed_homes_is_offered_or_left_out() {
        for name in ["not-claimed", "no-blobs"] {
            let (_tree, run) = ollama(name);

            assert_eq!(run.findings, Vec::<NewFinding>::new(), "{name}");
            assert_eq!(left_out(&run), [], "{name}");
            assert_eq!(run.outcome, ModuleOutcome::Completed, "{name}");
        }
    }

    #[test]
    fn a_partial_download_a_manifest_not_read_or_a_hidden_folder_leaves_out_the_whole_store_with_its_sizes() {
        for (name, reason) in [
            ("kept", "local-ai-models.left-out.ollama-download"),
            ("unreadable", "local-ai-models.left-out.ollama-unreadable"),
            ("long", "local-ai-models.left-out.ollama-unreadable"),
            ("hidden", "local-ai-models.left-out.ollama-unreadable"),
        ] {
            let (tree, run) = ollama(name);

            assert_eq!(run.findings, Vec::<NewFinding>::new(), "{name}");
            assert_eq!(left_out(&run), [("local-ai-models.ollama", reason)], "{name}");
            // Every file of the store but a manifest under a hidden folder, which is not one the module counts.
            let counted: Vec<_> =
                store_files(&tree).into_iter().filter(|file| !file.to_string_lossy().contains("/.hidden/")).collect();
            assert_eq!(run.left_out[0].sizes.logical.bytes(), bytes_of(&tree, &counted), "{name}");
        }
    }

    #[test]
    fn an_incomplete_model_is_left_out_with_its_own_files_beside_the_offered_ones() {
        let (tree, run) = ollama("offered");

        assert_eq!(left_out(&run), [("local-ai-models.ollama", "local-ai-models.left-out.ollama-incomplete")]);
        let broken = [
            ".ollama/models/manifests/registry.ollama.ai/library/broken/1".into(),
            format!(".ollama/models/blobs/sha256-{:064x}", 13).into(),
            format!(".ollama/models/blobs/sha256-{:064x}", 14).into(),
        ];
        assert_eq!(run.left_out[0].sizes.logical.bytes(), bytes_of(&tree, &broken));
        assert!(!run.findings.iter().any(|finding| finding.key().identity().contains("broken")));
        // The layer `broken:1` shares with llama3 still counts as named: it leaves with no finding.
        let shared = tree.root().join(format!(".ollama/models/blobs/sha256-{:064x}", 3));
        for finding in &run.findings {
            let Removal::MoveTogether(parts) = finding.removal() else { unreachable!("{:?}", finding.removal()) };
            assert!(!parts.all().contains(&shared), "{}", finding.key().identity());
        }
    }
}

mod positive {
    use fresher_core::{DangerLevel, Removal, Subject, TextArg};
    use fresher_testing::MatchOrder;

    use crate::files::bytes_of;
    use crate::left_out;
    use crate::support::{OLLAMA_HOMES, fixtures, kept, not_claimed, offered, ollama, scan, scan_with};
    use fresher_testing::FakeRunningApps;

    /// Per finding: its identity, its category, its danger, its title and its folder in the home.
    const EXPECTED: [(&str, &str, DangerLevel, &str, &str); 7] = [
        (
            "hugging-face/datasets--o--words",
            "local-ai-models.hugging-face-datasets",
            DangerLevel::Low,
            "o/words",
            ".cache/huggingface/hub/datasets--o--words",
        ),
        (
            "hugging-face/models--gpt2",
            "local-ai-models.hugging-face-models",
            DangerLevel::Low,
            "gpt2",
            ".cache/huggingface/hub/models--gpt2",
        ),
        (
            "hugging-face/models--meta--tiny",
            "local-ai-models.hugging-face-models",
            DangerLevel::Low,
            "meta/tiny",
            ".cache/huggingface/hub/models--meta--tiny",
        ),
        (
            "lm-studio-old/pub/model-b",
            "local-ai-models.lm-studio",
            DangerLevel::Medium,
            "pub/model-b",
            ".cache/lm-studio/models/pub/model-b",
        ),
        (
            "lm-studio/other/dup",
            "local-ai-models.lm-studio",
            DangerLevel::Medium,
            "other/dup",
            ".lmstudio/models/other/dup",
        ),
        ("lm-studio/pub/dup", "local-ai-models.lm-studio", DangerLevel::Medium, "pub/dup", ".lmstudio/models/pub/dup"),
        (
            "lm-studio/pub/model-a",
            "local-ai-models.lm-studio",
            DangerLevel::Medium,
            "pub/model-a",
            ".lmstudio/models/pub/model-a",
        ),
    ];

    #[test]
    fn the_offered_home_offers_every_model_in_its_category() {
        let (tree, run) = offered();

        let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
        identities.sort_unstable();
        assert_eq!(identities, EXPECTED.map(|(identity, ..)| identity));
        assert_eq!(left_out(&run), []);
        for (identity, category, danger, title, place) in EXPECTED {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{identity}");
            assert_eq!(found.title().key().as_str(), "local-ai-models.title", "{identity}");
            assert_eq!(found.title().args().get("name"), Some(&TextArg::Text(title.into())), "{identity}");
            assert_eq!(found.app(), None, "{identity}");
            let folder = tree.root().join(place);
            assert_eq!(found.subject(), &Subject::Path(folder.clone()), "{identity}");
            assert_eq!(found.removal(), &Removal::MoveToTrash(folder), "{identity}");
        }
    }

    /// Two blobs of 1000 and 2000 bytes and `refs/main` of 40; the two links inside the folder point at the blobs and
    /// add nothing, so the blobs are counted once.
    #[test]
    fn a_model_s_size_counts_its_blobs_once_and_its_links_as_links() {
        let (_tree, run) = offered();

        let tiny =
            run.findings.iter().find(|finding| finding.key().identity() == "hugging-face/models--meta--tiny").unwrap();

        assert_eq!(tiny.sizes().logical.bytes(), 1000 + 2000 + 40);
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        for name in ["local-ai-models-not-claimed", "local-ai-models-kept", "local-ai-models-offered"] {
            let (_tree, walked) = scan(name);
            let (_tree, reversed) = scan_with(name, FakeRunningApps::default(), false, MatchOrder::Reversed);

            assert_eq!(walked.golden().unwrap(), reversed.golden().unwrap(), "{name}");
        }
    }

    #[test]
    fn a_scan_restored_from_the_cache_offers_the_same() {
        let (_tree, walked) = offered();
        let (_tree, restored) =
            scan_with("local-ai-models-offered", FakeRunningApps::default(), true, MatchOrder::Walked);

        assert_eq!(restored.golden().unwrap(), walked.golden().unwrap());
    }

    /// An Ollama finding: its identity, its title, its reason and the other model it names, its parts in the home.
    type OllamaFinding = (&'static str, &'static str, &'static str, Option<&'static str>, &'static [&'static str]);

    const OLLAMA_EXPECTED: [OllamaFinding; 6] = [
        (
            "ollama/hf.co/org/tiny-gguf/q4",
            "hf.co/org/tiny-gguf:q4",
            "local-ai-models.reason.ollama",
            None,
            &["manifests/hf.co/org/tiny-gguf/q4", "11", "12"],
        ),
        (
            "ollama/registry.ollama.ai/library/llama3/8b",
            "llama3:8b, llama3:latest",
            "local-ai-models.reason.ollama-shared",
            Some("broken:1"),
            &[
                "manifests/registry.ollama.ai/library/llama3/8b",
                "manifests/registry.ollama.ai/library/llama3/latest",
                "1",
                "2",
            ],
        ),
        (
            "ollama/registry.ollama.ai/library/mistral/7b",
            "mistral:7b",
            "local-ai-models.reason.ollama",
            None,
            &["manifests/registry.ollama.ai/library/mistral/7b", "4", "5"],
        ),
        (
            "ollama/registry.ollama.ai/library/qwen/base",
            "qwen:base",
            "local-ai-models.reason.ollama-shared",
            Some("me/qwen-chat:latest"),
            &["manifests/registry.ollama.ai/library/qwen/base", "6"],
        ),
        (
            "ollama/registry.ollama.ai/me/qwen-chat/latest",
            "me/qwen-chat:latest",
            "local-ai-models.reason.ollama-shared-more",
            Some("qwen:base"),
            &["manifests/registry.ollama.ai/me/qwen-chat/latest", "9"],
        ),
        (
            "ollama/registry.ollama.ai/me/qwen-tools/latest",
            "me/qwen-tools:latest",
            "local-ai-models.reason.ollama-shared",
            Some("me/qwen-chat:latest"),
            &["manifests/registry.ollama.ai/me/qwen-tools/latest", "15", "16"],
        ),
    ];

    /// The place in the home of a part of `OLLAMA_EXPECTED`: a manifest's path in the store, or a layer's number.
    fn ollama_place(part: &str) -> std::path::PathBuf {
        match part.parse::<u64>() {
            Ok(number) => format!(".ollama/models/blobs/sha256-{number:064x}").into(),
            Err(_) => std::path::Path::new(".ollama/models").join(part),
        }
    }

    #[test]
    fn the_ollama_home_offers_each_model_once_with_its_manifests_and_unshared_layers_together() {
        let (tree, run) = ollama("offered");

        let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
        identities.sort_unstable();
        assert_eq!(identities, OLLAMA_EXPECTED.map(|(identity, ..)| identity));
        for (identity, title, reason, other, parts) in OLLAMA_EXPECTED {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            let places: Vec<std::path::PathBuf> = parts.iter().map(|part| ollama_place(part)).collect();
            let paths: Vec<std::path::PathBuf> = places.iter().map(|place| tree.root().join(place)).collect();
            assert_eq!((found.category().as_str(), found.danger()), ("local-ai-models.ollama", DangerLevel::Medium));
            assert_eq!(found.title().args().get("name"), Some(&TextArg::Text(title.into())), "{identity}");
            assert_eq!(found.reason().key().as_str(), reason, "{identity}");
            assert_eq!(found.reason().args().get("other"), other.map(|other| TextArg::Text(other.into())).as_ref());
            assert_eq!(found.subject(), &Subject::Path(paths[0].clone()), "{identity}");
            let Removal::MoveTogether(together) = found.removal() else {
                unreachable!("{identity}: {:?}", found.removal())
            };
            assert_eq!(together.all(), paths.as_slice(), "{identity}");
            assert_eq!(found.sizes().logical.bytes(), bytes_of(&tree, &places), "{identity}");
        }
        let chat = run.findings.iter().find(|finding| finding.key().identity().contains("qwen-chat")).unwrap();
        assert_eq!(chat.reason().args().get("count"), Some(&TextArg::Count(1)));
    }

    #[test]
    fn the_ollama_answers_do_not_depend_on_the_order_of_the_matches_or_the_cache() {
        for name in OLLAMA_HOMES {
            let home = format!("local-ai-models-ollama-{name}");
            let (_tree, walked) = ollama(name);
            let (_tree, reversed) = scan_with(&home, FakeRunningApps::default(), false, MatchOrder::Reversed);
            let (_tree, restored) = scan_with(&home, FakeRunningApps::default(), true, MatchOrder::Walked);

            assert_eq!(walked.golden().unwrap(), reversed.golden().unwrap(), "{name}");
            assert_eq!(walked.golden().unwrap(), restored.golden().unwrap(), "{name}");
        }
    }

    /// The report of every Ollama home, by home.
    #[test]
    fn the_ollama_golden_report_matches() {
        let mut report = serde_json::Map::new();
        for name in OLLAMA_HOMES {
            report.insert(name.into(), serde_json::from_str(&ollama(name).1.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/local-ai-models-ollama.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }

    /// The report of every home with its default system (the kept home with LM Studio open), by home.
    #[test]
    fn the_golden_report_matches() {
        let runs = [("not-claimed", not_claimed().1), ("kept", kept().1), ("offered", offered().1)];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/local-ai-models.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
