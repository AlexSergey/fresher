//! The `local-ai-models` module checked through the module kit: the category contract with its four catalogue files,
//! every text and left-out reason used by a run, and every part of every removal a matched path.

#[cfg(test)]
mod support;

use fresher_testing::{FakeRunningApps, MatchOrder, ModuleRun};

use crate::support::{OLLAMA_HOMES, kept, kept_with, not_claimed, offered, ollama, scan_with};

/// The run of every home with its default system, of the kept home with the open apps failing, of the offered homes
/// restored from the cache and handed in reverse, and of every Ollama home.
fn every_run() -> Vec<ModuleRun> {
    let again = |home, restored, order| scan_with(home, FakeRunningApps::default(), restored, order).1;
    let mut runs = vec![
        not_claimed().1,
        kept().1,
        kept_with(FakeRunningApps::failing("no process list")).1,
        offered().1,
        again("local-ai-models-offered", true, MatchOrder::Walked),
        again("local-ai-models-offered", false, MatchOrder::Reversed),
        again("local-ai-models-ollama-offered", true, MatchOrder::Walked),
        again("local-ai-models-ollama-offered", false, MatchOrder::Reversed),
    ];
    runs.extend(OLLAMA_HOMES.map(|name| ollama(name).1));
    runs
}

mod positive {
    use std::collections::BTreeSet;
    use std::path::Path;

    use fresher_core::{Module as _, TextKey};
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::every_run;
    use crate::support::local_ai_models;

    #[test]
    fn the_local_ai_models_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");
        assert_eq!(local_ai_models().descriptor().categories().len(), 4);

        assert_module_contract(&local_ai_models(), &Catalogue::Required(docs));
    }

    #[test]
    fn every_local_ai_models_text_is_used() {
        assert_texts_used(&local_ai_models(), &every_run());
    }

    #[test]
    fn every_declared_left_out_reason_is_given_by_a_fixture_run() {
        let module = local_ai_models();
        let declared: BTreeSet<&str> = module.descriptor().problems().iter().map(TextKey::as_str).collect();

        let runs = every_run();

        let given: BTreeSet<&str> =
            runs.iter().flat_map(|run| &run.left_out).map(|item| item.reason.as_str()).collect();
        assert_eq!(given, declared);
        assert_eq!(declared.len(), 7);
    }

    #[test]
    fn every_finding_reason_and_the_title_are_given_by_a_fixture_run() {
        let runs = every_run();

        let findings = || runs.iter().flat_map(|run| &run.findings);
        let reasons: BTreeSet<&str> = findings().map(|finding| finding.reason().key().as_str()).collect();
        let titles: BTreeSet<&str> = findings().map(|finding| finding.title().key().as_str()).collect();

        assert_eq!(
            reasons,
            BTreeSet::from([
                "local-ai-models.reason.hugging-face",
                "local-ai-models.reason.lm-studio",
                "local-ai-models.reason.ollama",
                "local-ai-models.reason.ollama-shared",
                "local-ai-models.reason.ollama-shared-more",
            ])
        );
        assert_eq!(titles, BTreeSet::from(["local-ai-models.title"]));
    }

    #[test]
    fn every_part_of_every_removal_names_a_matched_path() {
        assert_eq!(check_removals_matched(&every_run()), []);
    }
}
