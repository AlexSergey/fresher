//! The demo module checked through the module kit: the category contract and the use of its texts.

#[cfg(test)]
mod helpers {
    use std::path::Path;

    use fresher_module_demo::DemoModule;
    use fresher_testing::{FixtureTree, ModuleRun, Ports, run_module};

    pub(crate) fn demo() -> DemoModule {
        DemoModule::new().unwrap()
    }

    /// One run of the demo module on the empty fixture home, every setting at its default.
    pub(crate) fn run_on_an_empty_home(module: &DemoModule) -> ModuleRun {
        let home = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/homes/empty.toml");
        let tree = FixtureTree::from_toml(&home).unwrap();
        run_module(module, &tree, &Ports::at(0)).unwrap()
    }
}

mod negative {
    use fresher_testing::left_out_kinds;

    use crate::helpers::demo;

    #[test]
    fn what_only_passes_is_never_kept_for_good() {
        assert_eq!(left_out_kinds(&demo()).for_now, ["open"]);
    }
}

mod positive {
    use fresher_core::{ByteSize, Removal};
    use fresher_testing::{
        Catalogue, ContractProblem, assert_module_contract, assert_texts_used, check_removals_matched,
    };

    use crate::helpers::{demo, run_on_an_empty_home};

    #[test]
    fn the_demo_module_passes_the_contract() {
        assert_module_contract(&demo(), &Catalogue::DevOnly);
    }

    /// `demo.error.failed` is shown only when the demo cannot build its example findings, which no run provokes; it
    /// is a declared problem, so it counts as used.
    #[test]
    fn every_demo_key_is_used() {
        let module = demo();
        let run = run_on_an_empty_home(&module);

        assert_texts_used(&module, &[run]);
    }

    #[test]
    fn a_scan_leaves_out_one_made_up_cache() {
        let run = run_on_an_empty_home(&demo());

        let left_out: Vec<_> = run
            .left_out
            .iter()
            .map(|item| (item.category.as_str(), item.reason.as_str(), item.sizes.logical))
            .collect();

        assert_eq!(left_out, [("demo.caches", "demo.left-out.open", ByteSize::new(157_286_400))]);
    }

    /// The demo's paths are made up and nothing matches them, so the kit flags every path removal: the engine shows
    /// those items as needing an administrator, and only the tool removals as removable.
    #[test]
    fn the_made_up_paths_are_flagged_and_the_tool_removals_are_not() {
        let module = demo();
        let run = run_on_an_empty_home(&module);
        let paths = run
            .findings
            .iter()
            .filter(|finding| !matches!(finding.removal(), Removal::ToolCommand(_)))
            .map(|finding| ContractProblem::UnmatchedRemoval { identity: finding.key().identity().into() })
            .collect::<Vec<_>>();

        let problems = check_removals_matched(&[run]);

        assert_ne!(problems, []);
        assert_eq!(problems, paths);
    }

    #[test]
    fn only_what_is_in_use_or_never_a_candidate_is_kept_for_good() {
        assert_eq!(fresher_testing::left_out_kinds(&demo()).kept, Vec::<String>::new());
    }
}
