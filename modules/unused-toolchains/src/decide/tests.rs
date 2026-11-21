//! The decision on plain values: each row's keep before the offers, then the offers, the date, and the properties.

use std::path::Path;

use fresher_core::{EntryFacts, EntryKind, Inside, Sizes, Timestamp};

use crate::decide::{Scan, Version};
use crate::gathered::Candidate;
use crate::pointers::{Pointers, Resolved};
use crate::toolchains::{Row, Toolchains};

const TOOLCHAINS: &str = "/home/.rustup/toolchains";
const NODE: &str = "/home/.nvm/versions/node";
const CHANGED: u64 = 1_700_000_000;

/// A scan whose walk read everything, the row's switch on.
const ON: Scan = Scan { walk_complete: true, switch_on: true };

fn row(id: &str) -> Row {
    Toolchains::shipped().unwrap().rows().iter().find(|row| row.id == id).cloned().unwrap()
}

/// A version folder of the user's named `name` in `row`'s place, read in full, changed once, the first installed.
fn version(id: &str, name: &str) -> (Row, Candidate, Option<Version>) {
    let row = row(id);
    let place = if id == "rustup" { TOOLCHAINS } else { NODE };
    let found = Candidate {
        row: 0,
        path: Path::new(place).join(name),
        kind: EntryKind::Folder,
        sizes: Sizes::ZERO,
        facts: EntryFacts::REMOVABLE,
        inside: Some(Inside { newest_change: Some(Timestamp::from_secs(CHANGED)), ..Inside::EMPTY }),
    };
    let parsed = Version::read(row.shape, 0, name);
    (row, found, parsed)
}

fn toolchain(name: &str) -> (Row, Candidate, Option<Version>) {
    version("rustup", name)
}

fn node(name: &str) -> (Row, Candidate, Option<Version>) {
    version("nvm", name)
}

/// An old version of each row.
fn every_version() -> Vec<(Row, Candidate, Option<Version>)> {
    vec![toolchain("1.70.0-aarch64-apple-darwin"), node("v16.20.2")]
}

fn keeps(indices: &[usize]) -> Resolved {
    Resolved::Keeps(indices.iter().copied().collect())
}

/// Pointers that are all known and keep another version than the first.
fn elsewhere() -> Pointers {
    Pointers { default: keeps(&[1]), others: vec![keeps(&[2]), Resolved::Nothing] }
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, Inside};

    use super::{ON, elsewhere, every_version, keeps, node, toolchain};
    use crate::decide::{Reason, Scan, Verdict, decide};
    use crate::gathered::Candidate;
    use crate::pointers::{Pointers, Resolved};

    #[test]
    fn a_link_or_a_file_is_not_claimed() {
        for (row, found, version) in every_version() {
            for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
                let found = Candidate { kind, ..found.clone() };

                let verdict = decide(&found, &row, version.as_ref(), false, &elsewhere(), ON);

                assert_eq!(verdict, Verdict::NotClaimed, "{} as {kind:?}", row.id);
            }
        }
    }

    #[test]
    fn a_name_off_the_shape_is_not_claimed() {
        let rustup = ["my-toolchain", "1.83.0-beta.3", ".DS_Store", "stable", "1.70.0"];
        let nvm = ["18.17.0", "v18", "v18.17", "system", "v18.17.0-rc.1", ".DS_Store"];
        for (row, found, version) in rustup.map(toolchain).into_iter().chain(nvm.map(node)) {
            assert_eq!(version, None, "{}", found.path.display());

            assert_eq!(decide(&found, &row, None, false, &elsewhere(), ON), Verdict::NotClaimed);
        }
    }

    #[test]
    fn a_node_version_with_the_switch_off_is_not_claimed() {
        let off = Scan { switch_on: false, ..ON };
        let (row, found, version) = node("v16.20.2");

        assert_eq!(decide(&found, &row, version.as_ref(), false, &elsewhere(), off), Verdict::NotClaimed);
        let (rustup, found, version) = toolchain("1.70.0-aarch64-apple-darwin");
        assert_eq!(decide(&found, &rustup, version.as_ref(), false, &elsewhere(), off), Verdict::Offered);
    }

    #[test]
    fn a_moving_channel_and_the_newest_node_are_not_claimed() {
        for name in ["stable-aarch64-apple-darwin", "beta-x86_64-apple-darwin", "nightly-aarch64-apple-darwin"] {
            let (row, found, version) = toolchain(name);
            assert!(version.is_some(), "{name}");

            assert_eq!(decide(&found, &row, version.as_ref(), false, &elsewhere(), ON), Verdict::NotClaimed, "{name}");
        }
        let (row, found, version) = node("v22.11.0");
        assert_eq!(decide(&found, &row, version.as_ref(), true, &elsewhere(), ON), Verdict::NotClaimed);
    }

    #[test]
    fn a_folder_read_in_part_is_not_claimed() {
        let timeless = Inside { complete: true, ..Inside::EMPTY };
        for (row, found, version) in every_version() {
            let partial = found.inside.map(|inside| Inside { complete: false, ..inside });
            for inside in [None, partial, Some(timeless)] {
                let found = Candidate { inside, ..found.clone() };

                let verdict = decide(&found, &row, version.as_ref(), false, &elsewhere(), ON);

                assert_eq!(verdict, Verdict::NotClaimed, "{} with {inside:?}", row.id);
            }
        }
    }

    #[test]
    fn a_version_the_default_an_override_an_alias_or_a_pin_keeps_is_not_claimed() {
        let blocked = Scan { walk_complete: false, ..ON };
        let by_default = Pointers { default: keeps(&[0, 3]), others: vec![Resolved::Unknown] };
        let by_other = Pointers { default: Resolved::Unknown, others: vec![Resolved::Unknown, keeps(&[0])] };
        for (row, found, version) in every_version() {
            let inside = found.inside.map(|inside| Inside { special: true, foreign: true, ..inside });
            let found = Candidate { inside, ..found };
            for pointers in [&by_default, &by_other] {
                let verdict = decide(&found, &row, version.as_ref(), false, pointers, blocked);

                assert_eq!(verdict, Verdict::NotClaimed, "{} with {pointers:?}", row.id);
            }
        }
    }

    #[test]
    fn a_socket_inside_keeps_it_special() {
        let blocked = Scan { walk_complete: false, ..ON };
        let unknown = Pointers { default: Resolved::Unknown, others: vec![Resolved::Unknown] };
        for (row, found, version) in every_version() {
            let inside = found.inside.map(|inside| Inside { special: true, foreign: true, ..inside });
            let found = Candidate { inside, ..found };

            let verdict = decide(&found, &row, version.as_ref(), false, &unknown, blocked);

            assert_eq!(verdict, Verdict::LeftOut(Reason::Special), "{}", row.id);
        }
    }

    #[test]
    fn an_entry_not_yours_keeps_it() {
        let blocked = Scan { walk_complete: false, ..ON };
        let theirs = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
        let flagged = EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE };
        let restricted = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
        let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        for (row, found, version) in every_version() {
            for facts in [theirs, flagged, restricted, locked] {
                let found = Candidate { facts, ..found.clone() };

                let verdict = decide(&found, &row, version.as_ref(), false, &elsewhere(), blocked);

                assert_eq!(verdict, Verdict::LeftOut(Reason::NotYours), "{} {facts:?}", row.id);
            }
            let inside = found.inside.map(|inside| Inside { foreign: true, ..inside });
            let foreign = Candidate { inside, ..found };
            let verdict = decide(&foreign, &row, version.as_ref(), false, &elsewhere(), blocked);
            assert_eq!(verdict, Verdict::LeftOut(Reason::NotYours), "{}", row.id);
        }
    }

    #[test]
    fn a_walk_not_complete_keeps_it() {
        let blocked = Scan { walk_complete: false, ..ON };
        let unknown = Pointers { default: Resolved::Unknown, others: vec![Resolved::Unknown] };
        for (row, found, version) in every_version() {
            let verdict = decide(&found, &row, version.as_ref(), false, &unknown, blocked);

            assert_eq!(verdict, Verdict::LeftOut(Reason::PartialScan), "{}", row.id);
        }
    }

    #[test]
    fn an_unknown_default_keeps_it() {
        let unknown = Pointers { default: Resolved::Unknown, others: vec![keeps(&[2]), Resolved::Unknown] };
        for (row, found, version) in every_version() {
            let verdict = decide(&found, &row, version.as_ref(), false, &unknown, ON);

            assert_eq!(verdict, Verdict::LeftOut(Reason::DefaultUnknown), "{}", row.id);
        }
    }

    #[test]
    fn an_unknown_pin_keeps_it() {
        let unknown = Pointers { default: Resolved::Nothing, others: vec![keeps(&[2]), Resolved::Unknown] };
        for (row, found, version) in every_version() {
            let verdict = decide(&found, &row, version.as_ref(), false, &unknown, ON);

            assert_eq!(verdict, Verdict::LeftOut(Reason::PinUnknown), "{}", row.id);
        }
    }
}

mod positive {
    use fresher_core::{DangerLevel, Inside, Timestamp};

    use super::{ON, elsewhere, node, toolchain};
    use crate::categories::CATEGORIES;
    use crate::decide::{Verdict, decide, last_seen};
    use crate::pointers::{Pointers, Resolved};
    use crate::toolchains::Row;

    fn danger(row: &Row) -> Option<DangerLevel> {
        CATEGORIES.iter().find(|(category, _)| *category == row.category).map(|(_, danger)| *danger)
    }

    #[test]
    fn an_old_toolchain_nothing_points_at_is_offered_low() {
        for name in ["1.70.0-aarch64-apple-darwin", "1.75-x86_64-apple-darwin", "1.80.1-x86_64-unknown-linux-gnu"] {
            let (row, found, version) = toolchain(name);

            assert_eq!(decide(&found, &row, version.as_ref(), false, &elsewhere(), ON), Verdict::Offered, "{name}");
            assert_eq!(danger(&row), Some(DangerLevel::Low));
        }
    }

    #[test]
    fn a_dated_nightly_nothing_points_at_is_offered() {
        for name in ["nightly-2024-01-01-aarch64-apple-darwin", "beta-2023-12-24-x86_64-apple-darwin"] {
            let (row, found, version) = toolchain(name);
            let nothing = Pointers { default: Resolved::Keeps([1].into()), others: Vec::new() };

            assert_eq!(decide(&found, &row, version.as_ref(), false, &nothing, ON), Verdict::Offered, "{name}");
        }
    }

    #[test]
    fn an_old_node_version_is_offered_medium() {
        let (row, found, version) = node("v16.20.2");
        let no_default = Pointers { default: Resolved::Nothing, others: vec![Resolved::Nothing] };

        assert_eq!(decide(&found, &row, version.as_ref(), false, &no_default, ON), Verdict::Offered);
        assert_eq!(danger(&row), Some(DangerLevel::Medium));
    }

    #[test]
    fn the_date_is_the_newer_of_change_and_access() {
        let (old, new) = (Some(Timestamp::from_secs(100)), Some(Timestamp::from_secs(200)));
        for (change, access, seen) in
            [(old, new, new), (new, old, new), (old, None, old), (None, new, new), (None, None, None)]
        {
            let inside = Inside { newest_change: change, newest_file_access: access, ..Inside::EMPTY };

            assert_eq!(last_seen(&inside), seen, "{change:?} {access:?}");
        }
    }
}

mod property {
    use fresher_core::{EntryFacts, EntryKind, Inside, Timestamp};
    use proptest::prelude::*;

    use super::{every_version, node};
    use crate::decide::{Scan, Verdict, Version, decide};
    use crate::gathered::Candidate;
    use crate::pointers::{Pointers, Resolved};
    use crate::toolchains::Row;

    /// A version folder of either row in any state, most often one that would be offered.
    fn any_candidate(found: Candidate) -> impl Strategy<Value = Candidate> {
        let kind = prop_oneof![6 => Just(EntryKind::Folder), 1 => Just(EntryKind::File), 1 => Just(EntryKind::Symlink)];
        let flags = proptest::collection::vec(proptest::bool::weighted(0.1), 6);
        let times = (proptest::option::of(0..u64::MAX), proptest::option::of(0..u64::MAX));
        (kind, flags, times, proptest::bool::weighted(0.9)).prop_map(
            move |(kind, flags, (change, access), complete)| {
                let facts = EntryFacts {
                    owned_by_user: !flags[0],
                    user_immutable: flags[1],
                    system_immutable: flags[2],
                    restricted: flags[3],
                    ..EntryFacts::REMOVABLE
                };
                let inside = Inside {
                    newest_change: change.map(Timestamp::from_secs),
                    newest_file_access: access.map(Timestamp::from_secs),
                    special: flags[4],
                    foreign: flags[5],
                    complete,
                    ..Inside::EMPTY
                };
                Candidate { kind, facts, inside: Some(inside), ..found.clone() }
            },
        )
    }

    fn any_resolved() -> impl Strategy<Value = Resolved> {
        prop_oneof![
            Just(Resolved::Unknown),
            Just(Resolved::Nothing),
            proptest::collection::btree_set(1..5_usize, 0..3).prop_map(Resolved::Keeps),
        ]
    }

    /// Pointers of which none keeps the first version.
    fn any_pointers() -> impl Strategy<Value = Pointers> {
        (any_resolved(), proptest::collection::vec(any_resolved(), 0..4))
            .prop_map(|(default, others)| Pointers { default, others })
    }

    fn any_scan() -> impl Strategy<Value = Scan> {
        (any::<bool>(), any::<bool>()).prop_map(|(walk_complete, switch_on)| Scan { walk_complete, switch_on })
    }

    /// An old version of either row, in any state.
    fn any_old_version() -> impl Strategy<Value = (Row, Candidate, Option<Version>)> {
        (0..every_version().len()).prop_flat_map(|which| {
            let (row, found, version) = every_version().remove(which);
            any_candidate(found).prop_map(move |found| (row.clone(), found, version.clone()))
        })
    }

    proptest! {
        #[test]
        fn a_kept_version_is_never_offered_or_left_out_whatever_else_is_unknown(
            (row, found, version) in any_old_version(),
            pointers in any_pointers(),
            by_default: bool,
            scan in any_scan(),
        ) {
            let mut pointers = pointers;
            let keeps = Resolved::Keeps([0].into());
            if by_default { pointers.default = keeps } else { pointers.others.push(keeps) }

            prop_assert_eq!(decide(&found, &row, version.as_ref(), false, &pointers, scan), Verdict::NotClaimed);
        }

        #[test]
        fn the_default_and_the_newest_are_never_offered(
            found in any_candidate(node("v22.11.0").1),
            pointers in any_pointers(),
            scan in any_scan(),
        ) {
            let (row, _, version) = node("v22.11.0");
            prop_assert_eq!(decide(&found, &row, version.as_ref(), true, &pointers, scan), Verdict::NotClaimed);

            let by_default = Pointers { default: Resolved::Keeps([0].into()), ..pointers };
            prop_assert_eq!(decide(&found, &row, version.as_ref(), false, &by_default, scan), Verdict::NotClaimed);
        }

        #[test]
        fn the_switch_off_never_claims_a_node_version(
            found in any_candidate(node("v16.20.2").1),
            pointers in any_pointers(),
            newest: bool,
            walk_complete: bool,
        ) {
            let (row, _, version) = node("v16.20.2");
            let off = Scan { walk_complete, switch_on: false };

            prop_assert_eq!(decide(&found, &row, version.as_ref(), newest, &pointers, off), Verdict::NotClaimed);
        }
    }
}
