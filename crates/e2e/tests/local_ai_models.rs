//! The local-ai-models module end to end: a fixture sandbox's home with a Hugging Face model (snapshot links into its
//! blobs), a dataset, an LM Studio model, a Space and a project holding a link into the model's blobs, the real
//! scanner, the `local-ai-models` module, the real settings file, scan cache and journal, and the real `DiskCleaner`
//! reaching only inside the sandbox, with a fake Trash. The flows of the design's section 7; the cleanup flow's report
//! is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test local_ai_models` rewrites
//! `fixtures/golden/local-ai-models-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "local_ai_models_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::fs;

    use fresher_core::{DangerLevel, Event, ItemOutcome, PutBackOutcome, ScanKind, SkipReason};
    use fresher_testing::TreeReport;
    use serde_json::{Value, json};

    use crate::flow::{Flow, left_out, select};
    use crate::tree::contents;

    const MODEL: (&str, &str) =
        ("local-ai-models.hugging-face-models", "home/.cache/huggingface/hub/models--meta--tiny");
    const DATASET: (&str, &str) =
        ("local-ai-models.hugging-face-datasets", "home/.cache/huggingface/hub/datasets--o--words");
    const LM_STUDIO: (&str, &str) = ("local-ai-models.lm-studio", "home/.lmstudio/models/pub/model-a");
    /// The three models of the flow home, in the order `Flow::findings` sorts them.
    const OFFERED: [(&str, &str); 3] = [DATASET, MODEL, LM_STUDIO];
    const SPACE: &str = "home/.cache/huggingface/hub/spaces--o--demo";
    const PROJECT_FILE: &str = "home/Code/app/main.py";
    const PROJECT_LINK: &str = "home/Code/app/weights";
    const BLOB: &str = "home/.cache/huggingface/hub/models--meta--tiny/blobs/bb";

    fn offered(models: &[(&str, &str)]) -> Vec<(String, String)> {
        models.iter().map(|(category, path)| ((*category).to_owned(), (*path).to_owned())).collect()
    }

    fn categories() -> Vec<&'static str> {
        OFFERED.iter().map(|(category, _)| *category).collect()
    }

    fn tree(flow: &Flow) -> Value {
        serde_json::to_value(TreeReport::of(&flow.path("home")).unwrap()).unwrap()
    }

    fn scan_kind(events: &[Event]) -> Option<&ScanKind> {
        events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(kind),
            _ => None,
        })
    }

    /// A Space and the project's link are never offered, nor are they part of a model.
    pub(crate) fn a_space_and_the_project_s_link_are_never_offered() {
        let flow = Flow::new();

        let events = flow.scan();

        let found = flow.findings();
        assert_eq!(found, offered(&OFFERED));
        assert!(found.iter().all(|(_, path)| path != SPACE && !path.starts_with("home/Code")), "{found:?}");
        assert_eq!(left_out(&events), []);
    }

    /// A blob held open: the model is skipped in use, nothing of it moves, the others go.
    pub(crate) fn a_model_with_a_file_open_is_skipped_in_use() {
        let flow = Flow::new();
        flow.scan();
        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();
        flow.open_files.hold(flow.path(BLOB), "python");
        let before = contents(&flow.path(MODEL.1));

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let expected: Vec<(String, ItemOutcome)> = OFFERED
            .iter()
            .map(|(_, path)| {
                let outcome = if *path == MODEL.1 {
                    ItemOutcome::Skipped { reason: SkipReason::InUse { app: "python".into() } }
                } else {
                    ItemOutcome::Moved
                };
                ((*path).to_owned(), outcome)
            })
            .collect();
        assert_eq!(outcomes, expected);
        assert_eq!(contents(&flow.path(MODEL.1)), before, "nothing of the model moved");
        assert!(!flow.path(DATASET.1).exists() && !flow.path(LM_STUDIO.1).exists());
    }

    /// The project's link is still a link after the cleanup; its target went with the model.
    pub(crate) fn the_project_s_link_stays_a_link_after_the_cleanup() {
        let flow = Flow::new();
        flow.scan();
        let link = flow.path(PROJECT_LINK);
        let target = fs::read_link(&link).unwrap();
        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();

        let (_, cleaned) = flow.clean(&view);

        assert!(flow.outcomes(&view, &cleaned).iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved));
        assert!(fs::symlink_metadata(&link).unwrap().file_type().is_symlink(), "still a link");
        assert_eq!(fs::read_link(&link).unwrap(), target);
        assert!(!link.exists(), "its target went with the model");
    }

    /// Scan, review, confirm to the Trash, put back. The Space and the project stay.
    pub(crate) fn a_confirmed_cleanup_moves_the_model_folder_with_its_links_to_the_trash() -> Value {
        let flow = Flow::new();
        let events = flow.scan();

        assert_eq!(flow.findings(), offered(&OFFERED));
        assert_eq!(left_out(&events), []);
        let before: Vec<_> = OFFERED.iter().map(|(_, path)| contents(&flow.path(path))).collect();
        assert!(before.iter().all(|files| !files.is_empty()));
        let kept = [contents(&flow.path(SPACE)), contents(&flow.path(PROJECT_FILE))];

        let view = flow.engine.prepare_cleanup(&select(&categories())).unwrap();

        assert_eq!(view.summary.count, 3);
        assert_eq!(view.summary.highest, DangerLevel::Medium);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        assert!(OFFERED.iter().all(|(_, path)| !flow.path(path).exists()));
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), 3);
        assert_eq!(flow.journal().iter().filter(|record| record["type"] == "item").count(), 3);
        assert_eq!([contents(&flow.path(SPACE)), contents(&flow.path(PROJECT_FILE))], kept);
        assert_eq!(flow.findings(), []);
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let after: Vec<_> = OFFERED.iter().map(|(_, path)| contents(&flow.path(path))).collect();
        assert_eq!(after, before, "put back byte for byte");
        assert!(flow.path(PROJECT_LINK).exists(), "the link points at the model again");
        let report = json!({
            "findings": offered(&OFFERED),
            "leftOut": left_out(&events),
            "plan": view,
            "outcomes": outcomes,
            "treeAfterCleanup": tree_after_cleanup,
            "putBack": back,
            "journal": flow.journal(),
            "treeAfterPutBack": tree(&flow),
        });
        flow.relative(&report)
    }

    /// The models are offered per category with their sizes: the folder's files, its links counted as links.
    pub(crate) fn the_scan_offers_each_model_in_its_category_with_its_sizes() {
        let flow = Flow::new();
        flow.scan();

        let overview = flow.engine.overview().unwrap();

        let mut totals: Vec<(String, u64)> = overview
            .totals
            .iter()
            .map(|total| (total.category.as_str().to_owned(), total.sizes.logical.bytes()))
            .collect();
        totals.sort();
        // The dataset's blob and its file (5000 + 100 bytes).
        // The model's two blobs and its refs (3040 bytes); its links count as links, not as the blobs again.
        let expected = [(DATASET.0, 5100), (MODEL.0, 3040), (LM_STUDIO.0, 8192)];
        assert_eq!(totals, expected.map(|(category, bytes)| (category.to_owned(), bytes)));
    }

    /// A new engine on the same data folder restores the last scan: the same models as the walk.
    pub(crate) fn a_restored_scan_offers_the_same_models() {
        let mut flow = Flow::new();
        flow.scan();
        let found = flow.findings();
        flow.restart();

        let restored = flow.restore();

        assert!(matches!(scan_kind(&restored), Some(ScanKind::Restored { .. })), "{:?}", scan_kind(&restored));
        assert_eq!(flow.findings(), found);
        assert_eq!(found, offered(&OFFERED));
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn a_space_and_the_project_s_link_are_never_offered() {
        flows::a_space_and_the_project_s_link_are_never_offered();
    }

    #[test]
    fn a_model_with_a_file_open_is_skipped_in_use() {
        flows::a_model_with_a_file_open_is_skipped_in_use();
    }

    #[test]
    fn the_project_s_link_stays_a_link_after_the_cleanup() {
        flows::the_project_s_link_stays_a_link_after_the_cleanup();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn the_scan_offers_each_model_in_its_category_with_its_sizes() {
        flows::the_scan_offers_each_model_in_its_category_with_its_sizes();
    }

    #[test]
    fn a_confirmed_cleanup_moves_the_model_folder_with_its_links_to_the_trash() {
        check_golden(
            "local-ai-models-flow",
            &flows::a_confirmed_cleanup_moves_the_model_folder_with_its_links_to_the_trash(),
        );
    }

    #[test]
    fn a_restored_scan_offers_the_same_models() {
        flows::a_restored_scan_offers_the_same_models();
    }
}
