//! The local-ai-models module on its fixture homes: what it must not claim or must keep first, then what it must
//! offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-local-ai-models` rewrites
//! `fixtures/golden/local-ai-models.json`; the diff is reviewed.

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

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};
    use fresher_testing::FakeRunningApps;

    use crate::left_out;
    use crate::support::{kept, kept_with, not_claimed};

    /// A space, locks, a version file, a repository without its sign, a name of four parts, a link, a loose file, a
    /// hidden publisher, Ollama's store and a model file in a project.
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
}

mod positive {
    use fresher_core::{DangerLevel, Removal, Subject, TextArg};
    use fresher_testing::MatchOrder;

    use crate::left_out;
    use crate::support::{fixtures, kept, not_claimed, offered, scan, scan_with};
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
