//! The unused-toolchains module on its fixture homes: what it must not claim or must keep first, then what it must
//! offer, and its golden report.
//!
//! `UPDATE_GOLDEN=1 cargo nextest run -p fresher-module-unused-toolchains` rewrites
//! `fixtures/golden/unused-toolchains.json`; the diff is reviewed.

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

/// The identity of each finding of the run, sorted.
fn identities(run: &ModuleRun) -> Vec<&str> {
    let mut identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
    identities.sort_unstable();
    identities
}

mod negative {
    use fresher_core::{ModuleOutcome, NewFinding};

    use crate::left_out;
    use crate::support::{kept, kept_unknown, not_claimed, switched_off};

    #[test]
    fn nothing_in_the_not_claimed_home_is_offered_or_left_out() {
        let (_tree, run) = not_claimed();

        assert_eq!(run.findings, Vec::<NewFinding>::new());
        assert_eq!(left_out(&run), []);
        assert_eq!(run.outcome, ModuleOutcome::Completed);
    }

    #[test]
    fn the_kept_homes_offer_nothing_and_leave_out_each_with_its_reason() {
        let (_tree, blocked) = kept();
        let (_tree, unknown) = kept_unknown();

        assert_eq!(blocked.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&blocked),
            [
                ("unused-toolchains.node", "unused-toolchains.left-out.not-yours"),
                ("unused-toolchains.node", "unused-toolchains.left-out.special"),
                ("unused-toolchains.rust", "unused-toolchains.left-out.not-yours"),
                ("unused-toolchains.rust", "unused-toolchains.left-out.special"),
            ]
        );
        assert_eq!(unknown.findings, Vec::<NewFinding>::new());
        assert_eq!(
            left_out(&unknown),
            [
                ("unused-toolchains.node", "unused-toolchains.left-out.pin-unknown"),
                ("unused-toolchains.node", "unused-toolchains.left-out.pin-unknown"),
                ("unused-toolchains.rust", "unused-toolchains.left-out.default-unknown"),
                ("unused-toolchains.rust", "unused-toolchains.left-out.default-unknown"),
            ]
        );
    }

    #[test]
    fn the_switch_off_claims_no_node_version() {
        let (_tree, run) = switched_off();

        assert!(run.findings.iter().all(|finding| finding.category().as_str() == "unused-toolchains.rust"));
        assert_eq!(left_out(&run), []);
    }
}

mod positive {
    use fresher_core::{DangerLevel, Removal, Subject, TextArg};
    use fresher_testing::MatchOrder;

    use crate::support::{Variant, fixtures, kept, not_claimed, offered, scan_as, switched_off};
    use crate::{identities, left_out};

    /// Per finding: its identity, its category, its danger, its title and its folder in the home.
    const EXPECTED: [(&str, &str, DangerLevel, &str, &str); 4] = [
        ("nvm/v12.22.12", "unused-toolchains.node", DangerLevel::Medium, "v12.22.12", ".nvm/versions/node/v12.22.12"),
        ("nvm/v14.21.3", "unused-toolchains.node", DangerLevel::Medium, "v14.21.3", ".nvm/versions/node/v14.21.3"),
        (
            "rustup/1.70.0-aarch64-apple-darwin",
            "unused-toolchains.rust",
            DangerLevel::Low,
            "1.70.0-aarch64-apple-darwin",
            ".rustup/toolchains/1.70.0-aarch64-apple-darwin",
        ),
        (
            "rustup/nightly-2023-06-01-aarch64-apple-darwin",
            "unused-toolchains.rust",
            DangerLevel::Low,
            "nightly-2023-06-01-aarch64-apple-darwin",
            ".rustup/toolchains/nightly-2023-06-01-aarch64-apple-darwin",
        ),
    ];

    #[test]
    fn the_offered_home_offers_each_unused_version_in_its_category() {
        let (tree, run) = offered();

        assert_eq!(identities(&run), EXPECTED.map(|(identity, ..)| identity));
        assert_eq!(left_out(&run), []);
        for (identity, category, danger, name, place) in EXPECTED {
            let found = run.findings.iter().find(|finding| finding.key().identity() == identity).unwrap();
            assert_eq!((found.category().as_str(), found.danger()), (category, danger), "{identity}");
            assert_eq!(found.title().args().get("name"), Some(&TextArg::Text(name.into())), "{identity}");
            let folder = tree.root().join(place);
            assert_eq!(found.subject(), &Subject::Path(folder.clone()), "{identity}");
            assert_eq!(found.removal(), &Removal::MoveToTrash(folder), "{identity}");
        }
    }

    #[test]
    fn a_pin_inside_node_modules_keeps_nothing() {
        let (_tree, run) = offered();

        assert!(identities(&run).contains(&"nvm/v14.21.3"));
    }

    #[test]
    fn the_switch_off_offers_the_rust_toolchains_only() {
        let (_tree, run) = switched_off();

        assert_eq!(
            identities(&run),
            ["rustup/1.70.0-aarch64-apple-darwin", "rustup/nightly-2023-06-01-aarch64-apple-darwin"]
        );
    }

    #[test]
    fn the_answers_do_not_depend_on_the_order_of_the_matches() {
        for name in ["unused-toolchains-not-claimed", "unused-toolchains-kept", "unused-toolchains-offered"] {
            let (_tree, walked) = scan_as(name, Variant::DEFAULT);
            let (_tree, reversed) = scan_as(name, Variant { order: MatchOrder::Reversed, ..Variant::DEFAULT });

            assert_eq!(walked.golden().unwrap(), reversed.golden().unwrap(), "{name}");
        }
    }

    /// The report of every home, by home.
    #[test]
    fn the_golden_report_matches() {
        let runs = [
            ("not-claimed", not_claimed().1),
            ("kept", kept().1),
            ("kept-unknown", crate::support::kept_unknown().1),
            ("offered", offered().1),
            ("switch-off", switched_off().1),
        ];
        let mut report = serde_json::Map::new();
        for (name, run) in &runs {
            report.insert((*name).into(), serde_json::from_str(&run.golden().unwrap()).unwrap());
        }
        let report = serde_json::to_string_pretty(&report).unwrap() + "\n";
        let golden = fixtures().join("golden/unused-toolchains.json");

        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&golden, &report).unwrap();
        }
        assert_eq!(report, std::fs::read_to_string(&golden).unwrap());
    }
}
