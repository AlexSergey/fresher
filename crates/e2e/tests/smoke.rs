//! The smoke end-to-end test: a fixture home is built, read back and compared with its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e` rewrites the golden files; the diff is reviewed before commit.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support;

mod negative {
    use fresher_testing::{FixtureTree, TreeEntry, TreeReport};

    use crate::support::fixtures;

    #[test]
    fn an_empty_home_has_nothing_to_report() {
        let tree = FixtureTree::from_toml(&fixtures().join("homes/empty.toml")).unwrap();
        let report = TreeReport::of(tree.root()).unwrap();

        assert_eq!(report.entries, Vec::<TreeEntry>::new());
        assert_eq!(report.total_logical_size, 0);
    }
}

mod positive {
    use fresher_testing::{FixtureTree, TreeReport};

    use crate::support::{check_golden, fixtures};

    #[test]
    fn the_smoke_home_matches_its_golden_report() {
        let tree = FixtureTree::from_toml(&fixtures().join("homes/smoke.toml")).unwrap();
        let report = TreeReport::of(tree.root()).unwrap();

        check_golden("smoke", &report);
    }
}
