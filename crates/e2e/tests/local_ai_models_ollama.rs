//! The Ollama models of the local-ai-models module end to end: a fixture sandbox's store (models sharing files and
//! layers), the real scanner, the module, the real settings file, scan cache and journal, and the real `DiskCleaner`
//! reaching only inside the sandbox with a fake Trash. A model goes together: its manifests and the layers no other
//! model names move to the Trash as one item, or nothing of it moves. The cleanup flow's report is the golden file.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e --test local_ai_models_ollama` rewrites
//! `fixtures/golden/local-ai-models-ollama-flow.json`; the diff is reviewed.

#[cfg(test)]
mod support;

#[cfg(test)]
#[path = "support/tree.rs"]
mod tree;

#[cfg(test)]
#[path = "local_ai_models_ollama_home/mod.rs"]
mod flow;

#[cfg(test)]
mod flows {
    use std::fs;

    use fresher_core::{
        DangerLevel, Event, FindingOrder, FindingsQuery, ItemOutcome, PutBackOutcome, ScanKind, SkipReason, Subject,
    };
    use fresher_testing::TreeReport;
    use serde_json::{Value, json};

    use crate::flow::{Flow, left_out, select};
    use crate::tree::contents;

    const CATEGORY: &str = "local-ai-models.ollama";
    const STORE: &str = "home/.ollama/models";
    const LLAMA: &str = "home/.ollama/models/manifests/registry.ollama.ai/library/llama3/8b";
    const LLAMA_TAG: &str = "home/.ollama/models/manifests/registry.ollama.ai/library/llama3/latest";
    const MISTRAL: &str = "home/.ollama/models/manifests/registry.ollama.ai/library/mistral/7b";
    const BASE: &str = "home/.ollama/models/manifests/registry.ollama.ai/library/qwen/base";
    const CHAT: &str = "home/.ollama/models/manifests/registry.ollama.ai/me/qwen-chat/latest";

    /// The path of the layer whose digest ends in `tail` (the flow's digests are zeros and a tail).
    fn layer(tail: &str) -> String {
        format!("{STORE}/blobs/sha256-{tail:0>64}")
    }

    /// What each model frees: its manifests first (the subject, then the other tags), then its own layers.
    fn parts(model: &str) -> Vec<String> {
        let tails: &[&str] = match model {
            LLAMA => &["1", "2", "3"],
            MISTRAL => &["4", "5"],
            BASE => &["6"],
            CHAT => &["9", "a"],
            _ => &[],
        };
        let mut parts = vec![model.to_owned()];
        if model == LLAMA {
            parts.push(LLAMA_TAG.to_owned());
        }
        parts.extend(tails.iter().map(|tail| layer(tail)));
        parts
    }

    /// The models the scan offers, by their subject, in the order `Flow::findings` sorts them.
    const OFFERED: [&str; 4] = [BASE, LLAMA, MISTRAL, CHAT];

    fn offered() -> Vec<(String, String)> {
        let mut found: Vec<_> = OFFERED.iter().map(|path| (CATEGORY.to_owned(), (*path).to_owned())).collect();
        found.sort();
        found
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

    /// The bytes of every path of `paths`, by path.
    fn all_contents(flow: &Flow, paths: &[String]) -> Vec<std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>> {
        paths.iter().map(|path| contents(&flow.path(path))).collect()
    }

    /// The selection of the one finding whose subject is `model`.
    fn only(flow: &Flow, model: &str) -> fresher_core::CleanupSelection {
        let page = flow.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        let id = page
            .findings
            .iter()
            .find(|finding| matches!(finding.finding().subject(), Subject::Path(path) if path == &flow.path(model)))
            .map(|finding| serde_json::to_value(finding.id()).unwrap())
            .unwrap();
        serde_json::from_value(json!({ "findings": [id], "categories": [] })).unwrap()
    }

    fn all(flow: &Flow) -> fresher_core::CleanupPlanView {
        flow.engine.prepare_cleanup(&select(&[CATEGORY])).unwrap()
    }

    /// A layer held open (or mapped: the fakes hold both the same way): the model is skipped in use, none of its
    /// parts moves, the other models go.
    pub(crate) fn a_model_whose_layer_is_mapped_moves_nothing() {
        let flow = Flow::new();
        flow.scan();
        let view = all(&flow);
        flow.open_files.hold(flow.path(&layer("5")), "ollama");
        let before = all_contents(&flow, &parts(MISTRAL));

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let mut expected: Vec<(String, ItemOutcome)> = OFFERED
            .iter()
            .map(|path| {
                let outcome = if *path == MISTRAL {
                    ItemOutcome::Skipped { reason: SkipReason::InUse { app: "ollama".into() } }
                } else {
                    ItemOutcome::Moved
                };
                ((*path).to_owned(), outcome)
            })
            .collect();
        expected.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(outcomes, expected);
        assert_eq!(all_contents(&flow, &parts(MISTRAL)), before, "no part of the model moved");
        let others: usize = [LLAMA, BASE, CHAT].iter().map(|model| parts(model).len()).sum();
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), others, "the others went, and only they");
        let journal = flow.journal();
        let mistral =
            journal.iter().find(|record| record["type"] == "item" && record["outcome"]["kind"] == "skipped").unwrap();
        assert_eq!(mistral["trashed"], json!([]));
    }

    /// One model of a pair going: the layers its twin names and the twin's manifest stay, and the twin is still
    /// offered, with all its files.
    pub(crate) fn a_shared_layer_and_the_sharers_manifest_stay_after_the_cleanup() {
        let flow = Flow::new();
        flow.scan();
        let twin = all_contents(&flow, &parts(CHAT));
        let shared = [layer("7"), layer("8")];
        let shared_before = all_contents(&flow, &shared);
        let view = flow.engine.prepare_cleanup(&only(&flow, BASE)).unwrap();

        let (_, cleaned) = flow.clean(&view);

        assert_eq!(flow.outcomes(&view, &cleaned), [(BASE.to_owned(), ItemOutcome::Moved)]);
        assert!(parts(BASE).iter().all(|path| !flow.path(path).exists()), "its own files went");
        assert_eq!(all_contents(&flow, &shared), shared_before, "the shared layers stay");
        assert_eq!(all_contents(&flow, &parts(CHAT)), twin, "the twin keeps every file");
        flow.scan();
        assert!(flow.findings().contains(&(CATEGORY.to_owned(), CHAT.to_owned())), "the twin is still offered");
    }

    /// A layer rewritten as another file after the plan: the model is `Replaced`, none of its parts moves.
    pub(crate) fn a_layer_replaced_after_the_plan_moves_nothing() {
        let flow = Flow::new();
        flow.scan();
        let view = all(&flow);
        let blob = flow.path(&layer("2"));
        let size = fs::metadata(&blob).unwrap().len();
        let other = blob.with_extension("new");
        fs::write(&other, vec![7_u8; usize::try_from(size).unwrap()]).unwrap();
        fs::rename(&other, &blob).unwrap();
        let before = all_contents(&flow, &parts(LLAMA));

        let (_, cleaned) = flow.clean(&view);

        let outcomes = flow.outcomes(&view, &cleaned);
        let llama = outcomes.iter().find(|(path, _)| path == LLAMA).unwrap();
        assert_eq!(llama.1, ItemOutcome::Skipped { reason: SkipReason::Replaced });
        assert_eq!(all_contents(&flow, &parts(LLAMA)), before, "no part of the model moved");
        assert_eq!(outcomes.iter().filter(|(_, outcome)| *outcome == ItemOutcome::Moved).count(), 3);
        let others: usize = [MISTRAL, BASE, CHAT].iter().map(|model| parts(model).len()).sum();
        assert_eq!(fs::read_dir(flow.sandbox.trash()).unwrap().count(), others, "only the others went");
    }

    /// The scan offers each model with the size it frees: its manifests and its own layers, and says it in its reason.
    pub(crate) fn the_scan_offers_each_model_with_its_freed_size() {
        let flow = Flow::new();
        flow.scan();

        assert_eq!(flow.findings(), offered());
        let page = flow.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000)).unwrap();
        for finding in &page.findings {
            let subject = match finding.finding().subject() {
                Subject::Path(path) => flow.relative_path(path),
                Subject::External { id, .. } => id.clone(),
            };
            let finding = serde_json::to_value(finding.finding()).unwrap();
            let model = OFFERED.iter().find(|model| subject == **model).unwrap();
            let freed: u64 = parts(model).iter().map(|part| fs::metadata(flow.path(part)).unwrap().len()).sum();
            assert_eq!(finding["sizes"]["logical"], freed, "{model}");
            assert_eq!(finding["reason"]["args"]["size"]["value"], freed, "{model}");
            assert_eq!(finding["danger"], "medium");
        }
    }

    /// Scan, review, confirm to the Trash, put back. The shared layers stay throughout.
    pub(crate) fn a_confirmed_model_moves_its_manifests_and_unshared_layers_to_the_trash_together() -> Value {
        let flow = Flow::new();
        flow.scan();
        let kept = all_contents(&flow, &[layer("7"), layer("8")]);

        let view = all(&flow);

        assert_eq!(view.summary.count, 4);
        assert_eq!(view.summary.highest, DangerLevel::Medium);
        let (cleanup, cleaned) = flow.clean(&view);
        let outcomes = flow.outcomes(&view, &cleaned);
        assert!(outcomes.iter().all(|(_, outcome)| *outcome == ItemOutcome::Moved), "{outcomes:?}");
        for model in OFFERED {
            assert!(parts(model).iter().all(|part| !flow.path(part).exists()), "{model}");
        }
        assert_eq!(all_contents(&flow, &[layer("7"), layer("8")]), kept, "the shared layers stay");
        let journal = flow.journal();
        let begin = journal.iter().find(|record| record["type"] == "begin").unwrap();
        let begin_parts: usize =
            begin["items"].as_array().unwrap().iter().map(|item| item["more"].as_array().map_or(0, Vec::len) + 1).sum();
        assert_eq!(begin_parts, OFFERED.iter().map(|model| parts(model).len()).sum::<usize>(), "every part journalled");
        let planned = begin["items"].as_array().unwrap();
        let items: Vec<_> = journal.iter().filter(|record| record["type"] == "item").collect();
        assert_eq!(items.len(), OFFERED.len());
        for record in items {
            let item = &planned[usize::try_from(record["index"].as_u64().unwrap()).unwrap()];
            let trashed = record["trashed"].as_array().unwrap();
            assert_eq!(trashed[0]["original"], item["path"], "the subject moved first");
            let parts = 1 + item["more"].as_array().unwrap().len();
            assert_eq!(usize::try_from(record["went"].as_u64().unwrap()).unwrap(), parts);
            assert_eq!(trashed.len(), parts, "every part is in the item's record");
        }
        let tree_after_cleanup = tree(&flow);

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        let report = json!({
            "findings": offered(),
            "plan": view,
            "outcomes": outcomes,
            "treeAfterCleanup": tree_after_cleanup,
            "putBack": back,
            "journal": flow.journal(),
            "treeAfterPutBack": tree(&flow),
        });
        flow.relative(&report)
    }

    /// Put back restores every file byte for byte, and the journal says the subject came back last.
    pub(crate) fn put_back_restores_every_file_byte_for_byte() {
        let flow = Flow::new();
        flow.scan();
        let before = contents(&flow.path(STORE));
        let view = all(&flow);
        let (cleanup, _) = flow.clean(&view);
        assert!(!flow.path(LLAMA).exists());

        let back = flow.engine.put_back(cleanup, None).unwrap();

        assert!(back.items.iter().all(|item| item.outcome == PutBackOutcome::PutBack), "{back:?}");
        assert_eq!(contents(&flow.path(STORE)), before, "byte for byte");
        let journal = flow.journal();
        let begin = journal.iter().find(|record| record["type"] == "begin").unwrap();
        let planned = begin["items"].as_array().unwrap();
        let records: Vec<_> = journal.iter().filter(|record| record["type"] == "put-back").collect();
        assert_eq!(records.len(), OFFERED.len());
        for record in records {
            let item = &planned[usize::try_from(record["index"].as_u64().unwrap()).unwrap()];
            let count = 1 + item["more"].as_array().unwrap().len();
            let descending: Vec<Value> = (0..count).rev().map(|part| json!(part)).collect();
            assert_eq!(record["back"], json!(descending), "every part back, the subject (part 0) last");
        }
    }

    /// After one model went, the module's next scan offers the others unchanged and nothing of the removed one.
    pub(crate) fn ollama_lists_nothing_of_a_removed_model() {
        let flow = Flow::new();
        flow.scan();
        let before = flow.findings();
        let view = flow.engine.prepare_cleanup(&only(&flow, MISTRAL)).unwrap();
        let (_, cleaned) = flow.clean(&view);
        assert_eq!(flow.outcomes(&view, &cleaned), [(MISTRAL.to_owned(), ItemOutcome::Moved)]);

        flow.scan();

        let expected: Vec<_> = before.into_iter().filter(|(_, path)| path != MISTRAL).collect();
        assert_eq!(flow.findings(), expected);
    }

    /// A new engine on the same data folder restores the last scan: the same models as the walk. The module reads the
    /// manifests again on a restore, so a manifest gone between the restart and the restore is seen: the store is left
    /// out and nothing of it is offered.
    pub(crate) fn a_restored_scan_offers_the_same_models() {
        let mut flow = Flow::new();
        flow.scan();
        let found = flow.findings();
        flow.restart();

        let restored = flow.restore();

        assert!(matches!(scan_kind(&restored), Some(ScanKind::Restored { .. })), "{:?}", scan_kind(&restored));
        assert_eq!(flow.findings(), found);
        assert_eq!(found, offered());
        assert_eq!(left_out(&restored), []);
        flow.restart();
        fs::rename(flow.path(MISTRAL), flow.path("home/mistral-moved-away")).unwrap();

        let changed = flow.restore();

        assert!(matches!(scan_kind(&changed), Some(ScanKind::Restored { .. })), "{:?}", scan_kind(&changed));
        assert_eq!(flow.findings(), []);
        let keys: Vec<_> = left_out(&changed).into_iter().map(|(_, key, _)| key).collect();
        assert_eq!(keys, ["local-ai-models.left-out.ollama-unreadable"]);
    }
}

mod negative {
    use crate::flows;

    #[test]
    fn a_model_whose_layer_is_mapped_moves_nothing() {
        flows::a_model_whose_layer_is_mapped_moves_nothing();
    }

    #[test]
    fn a_shared_layer_and_the_sharers_manifest_stay_after_the_cleanup() {
        flows::a_shared_layer_and_the_sharers_manifest_stay_after_the_cleanup();
    }

    #[test]
    fn a_layer_replaced_after_the_plan_moves_nothing() {
        flows::a_layer_replaced_after_the_plan_moves_nothing();
    }
}

mod positive {
    use crate::flows;
    use crate::support::check_golden;

    #[test]
    fn the_scan_offers_each_model_with_its_freed_size() {
        flows::the_scan_offers_each_model_with_its_freed_size();
    }

    #[test]
    fn a_confirmed_model_moves_its_manifests_and_unshared_layers_to_the_trash_together() {
        check_golden(
            "local-ai-models-ollama-flow",
            &flows::a_confirmed_model_moves_its_manifests_and_unshared_layers_to_the_trash_together(),
        );
    }

    #[test]
    fn put_back_restores_every_file_byte_for_byte() {
        flows::put_back_restores_every_file_byte_for_byte();
    }

    #[test]
    fn ollama_lists_nothing_of_a_removed_model() {
        flows::ollama_lists_nothing_of_a_removed_model();
    }

    #[test]
    fn a_restored_scan_offers_the_same_models() {
        flows::a_restored_scan_offers_the_same_models();
    }
}
