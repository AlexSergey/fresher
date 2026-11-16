//! The trial module checked through the module kit: the category contract, the use of its texts and its removals.

#[cfg(test)]
mod support;

mod positive {
    use fresher_testing::{Catalogue, assert_module_contract, assert_texts_used, check_removals_matched};

    use crate::support::{failed_listing, sandbox, trial, unclaimed};

    #[test]
    fn the_trial_module_passes_the_contract() {
        assert_module_contract(&trial(), &Catalogue::DevOnly);
    }

    #[test]
    fn every_trial_key_is_used() {
        let runs = [unclaimed().1, sandbox().1, failed_listing()];

        assert_texts_used(&trial(), &runs);
    }

    #[test]
    fn every_path_removal_names_a_matched_path() {
        let runs = [unclaimed().1, sandbox().1];

        assert_eq!(check_removals_matched(&runs), []);
    }
}
