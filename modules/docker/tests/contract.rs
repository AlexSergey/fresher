//! The `docker` module checked through the module kit: the contract of a module without a category, the use of its
//! texts over every run, and that it removes nothing.

#[cfg(test)]
mod support;

mod positive {
    use std::path::Path;

    use fresher_module_docker::DockerModule;
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::support::{every_scan, orbstack_read};

    #[test]
    fn the_docker_module_passes_the_contract() {
        let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/categories");
        let module = DockerModule::new().unwrap();

        assert_module_contract(&module, &Catalogue::Required(docs));
    }

    #[test]
    fn every_docker_text_is_used() {
        let runs: Vec<_> = every_scan().into_iter().map(|scan| scan.run).collect();

        assert_texts_used(&DockerModule::new().unwrap(), &runs);
    }

    #[test]
    fn the_only_program_run_is_the_runtime_s_read() {
        for scan in every_scan() {
            assert!(scan.calls.iter().all(|call| *call == orbstack_read()), "{:?}", scan.calls);
        }
    }

    #[test]
    fn the_module_has_no_removal() {
        let runs: Vec<_> = every_scan().into_iter().map(|scan| scan.run).collect();

        assert_eq!(check_removals_matched(&runs), []);
        assert!(runs.iter().all(|run| run.findings.is_empty()));
    }
}
