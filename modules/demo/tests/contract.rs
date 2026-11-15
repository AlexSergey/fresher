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

mod positive {
    use fresher_testing::{Catalogue, ContractProblem, assert_module_contract, check_texts_used};

    use crate::helpers::{demo, run_on_an_empty_home};

    #[test]
    fn the_demo_module_passes_the_contract() {
        assert_module_contract(&demo(), &Catalogue::DevOnly);
    }

    /// `demo.error.failed` is shown only when the demo cannot build its example findings, which no run provokes, so
    /// it is the one text a run leaves unused; anything else the check finds fails the test.
    #[test]
    fn every_demo_key_is_used() {
        let module = demo();
        let run = run_on_an_empty_home(&module);

        let problems = check_texts_used(&module, &[run]);

        assert_eq!(problems, [ContractProblem::UnusedText { key: "demo.error.failed".into() }]);
    }
}
