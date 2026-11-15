//! The smoke end-to-end test: a fixture home is built, read back and compared with its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-e2e` rewrites the golden files; the diff is reviewed before commit.

// Helpers of an integration test are test code too; `cfg(test)` lets Clippy's in-test allowances apply to them.
#[cfg(test)]
mod support {
    use std::path::PathBuf;

    use fresher_testing::TreeReport;

    pub(crate) fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
    }

    pub(crate) fn check_golden(name: &str, report: &TreeReport) {
        let golden = fixtures().join("golden").join(format!("{name}.json"));
        let actual = serde_json::to_string_pretty(report).unwrap() + "\n";

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, actual).unwrap();
            return;
        }

        let expected = std::fs::read_to_string(&golden).unwrap();
        assert_eq!(actual, expected, "{name}: the report differs from {}", golden.display());
    }
}

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
