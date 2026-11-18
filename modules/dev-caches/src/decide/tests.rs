//! The decision on plain values: each row's keep before its offer, the precedence of the rows, then every offer, and
//! two properties.

use fresher_core::{EntryFacts, EntryKind, Inside, MarkerCheck, RunningApp, RunningSet, Sizes};

use crate::gathered::{Candidate, Sign};
use crate::home::place;
use crate::tools::{Cache, Tools};

const XCODE: &str = "com.apple.dt.Xcode";

/// Every row of the shipped table.
fn rows() -> Vec<Cache> {
    Tools::shipped().unwrap().caches().to_vec()
}

fn row(id: &str) -> Cache {
    rows().into_iter().find(|row| row.id == id).unwrap()
}

/// The index of the row `id` in the shipped table.
fn index(id: &str) -> usize {
    rows().iter().position(|row| row.id == id).unwrap()
}

/// The full sign of `row`, valid.
fn full_sign(row: &Cache) -> Sign {
    let marker = row.sign.iter().any(|name| name == "CACHEDIR.TAG").then_some(MarkerCheck::Valid);
    Sign { holds: row.sign.clone(), marker }
}

/// A folder of the user's at `row`'s place, read in full, holding its own sign when the row's sign is its own.
fn candidate(row: &Cache) -> Candidate {
    Candidate {
        row: index(&row.id),
        path: place(row),
        kind: EntryKind::Folder,
        sizes: Sizes::ZERO,
        facts: EntryFacts::REMOVABLE,
        inside: Some(Inside::EMPTY),
        sign: if row.sign_at.is_none() { full_sign(row) } else { Sign::default() },
    }
}

/// `candidate`'s sign: its own, or a holder's holding the row's full sign.
fn sign_of(row: &Cache, candidate: &Candidate) -> Sign {
    if row.sign_at.is_none() { candidate.sign.clone() } else { full_sign(row) }
}

fn running(ids: &[&str]) -> RunningSet {
    let apps: Vec<RunningApp> = ids.iter().map(|id| fresher_testing::running_app("App", &[id]).unwrap()).collect();
    RunningSet::new(apps)
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, Inside, MarkerCheck};

    use super::{XCODE, candidate, row, rows, running, sign_of};
    use crate::decide::{NoSystem, Reason, Verdict, decide};
    use crate::gathered::{Candidate, Holder, Sign};
    use crate::home::place;

    #[test]
    fn a_file_a_link_or_an_other_entry_is_not_claimed_for_any_row() {
        let none = running(&[]);
        for row in rows() {
            for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
                let found = Candidate { kind, ..candidate(&row) };

                let verdict = decide(&found, &row, Some(&sign_of(&row, &found)), Ok(&none));

                assert_eq!(verdict, Verdict::NotClaimed, "{} as {kind:?}", row.id);
            }
        }
    }

    #[test]
    fn a_derived_data_folder_without_a_project_hash_name_is_not_claimed() {
        let xcode = row("xcode-derived-data");
        let none = running(&[]);
        for name in ["Demo-abc", "Plain", "ModuleCache.noindex", "-abcdefghijklmnopqrstuvwxyzab"] {
            let found = Candidate { path: place(&xcode).with_file_name(name), ..candidate(&xcode) };

            assert_eq!(decide(&found, &xcode, Some(&found.sign), Ok(&none)), Verdict::NotClaimed, "{name}");
        }
        let root = Candidate { path: "/".into(), ..candidate(&xcode) };
        assert_eq!(decide(&root, &xcode, Some(&root.sign), Ok(&none)), Verdict::NotClaimed);
    }

    #[test]
    fn a_cache_without_a_sign_holder_is_not_claimed() {
        let none = running(&[]);
        for row in rows() {
            assert_eq!(decide(&candidate(&row), &row, None, Ok(&none)), Verdict::NotClaimed, "{}", row.id);
        }
    }

    #[test]
    fn a_sign_holder_that_is_a_link_holds_no_sign() {
        let cargo = row("cargo-src");
        let link = Holder { index: 0, kind: EntryKind::Symlink, sign: super::full_sign(&cargo) };

        let verdict = decide(&candidate(&cargo), &cargo, link.sign(), Ok(&running(&[])));

        assert_eq!(verdict, Verdict::NotClaimed);
    }

    #[test]
    fn a_sign_missing_one_name_is_not_claimed() {
        let none = running(&[]);
        for row in rows() {
            let found = candidate(&row);
            let full = sign_of(&row, &found);
            for missing in &row.sign {
                let holds = full.holds.iter().filter(|name| *name != missing).cloned().collect();
                let sign = Sign { holds, ..full.clone() };

                assert_eq!(
                    decide(&found, &row, Some(&sign), Ok(&none)),
                    Verdict::NotClaimed,
                    "{} without {missing}",
                    row.id
                );
            }
        }
    }

    #[test]
    fn a_cache_directory_tag_that_is_invalid_or_unchecked_is_not_claimed() {
        let none = running(&[]);
        for id in ["gradle", "cargo-cache", "cargo-src"] {
            let row = row(id);
            let found = candidate(&row);
            for marker in [Some(MarkerCheck::Invalid), None] {
                let sign = Sign { marker, ..sign_of(&row, &found) };

                assert_eq!(decide(&found, &row, Some(&sign), Ok(&none)), Verdict::NotClaimed, "{id} with {marker:?}");
            }
        }
    }

    #[test]
    fn a_folder_not_read_in_full_is_not_claimed() {
        let none = running(&[]);
        for row in rows() {
            for inside in [None, Some(Inside { complete: false, ..Inside::EMPTY })] {
                let found = Candidate { inside, ..candidate(&row) };

                let verdict = decide(&found, &row, Some(&sign_of(&row, &found)), Ok(&none));

                assert_eq!(verdict, Verdict::NotClaimed, "{} with {inside:?}", row.id);
            }
        }
    }

    #[test]
    fn a_folder_holding_a_socket_is_left_out_as_special() {
        let npm = row("npm");
        let found = Candidate { inside: Some(Inside { special: true, ..Inside::EMPTY }), ..candidate(&npm) };

        assert_eq!(decide(&found, &npm, Some(&found.sign), Ok(&running(&[]))), Verdict::LeftOut(Reason::Special));
    }

    #[test]
    fn a_folder_not_the_users_flagged_locked_or_holding_a_foreign_entry_is_left_out_as_not_yours() {
        let gradle = row("gradle");
        let none = running(&[]);
        let theirs = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
        let flagged = EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE };
        let restricted = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
        let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        for facts in [theirs, flagged, restricted, locked] {
            let found = Candidate { facts, ..candidate(&gradle) };

            assert_eq!(
                decide(&found, &gradle, Some(&found.sign), Ok(&none)),
                Verdict::LeftOut(Reason::NotYours),
                "{facts:?}"
            );
        }
        let foreign = Candidate { inside: Some(Inside { foreign: true, ..Inside::EMPTY }), ..candidate(&gradle) };
        assert_eq!(decide(&foreign, &gradle, Some(&foreign.sign), Ok(&none)), Verdict::LeftOut(Reason::NotYours));
    }

    #[test]
    fn derived_data_is_left_out_when_the_running_apps_cannot_be_listed() {
        let xcode = row("xcode-derived-data");
        let found = candidate(&xcode);

        assert_eq!(decide(&found, &xcode, Some(&found.sign), Err(NoSystem)), Verdict::LeftOut(Reason::NoSystem));
    }

    #[test]
    fn derived_data_is_left_out_while_xcode_is_open() {
        let xcode = row("xcode-derived-data");
        let found = candidate(&xcode);
        for id in [XCODE, "com.apple.dt.xcode"] {
            let open = running(&["com.example.Editor", id]);

            assert_eq!(decide(&found, &xcode, Some(&found.sign), Ok(&open)), Verdict::LeftOut(Reason::AppOpen), "{id}");
        }
    }

    mod precedence {
        use fresher_core::{EntryFacts, EntryKind, Inside};

        use super::super::{XCODE, candidate, row, rows, running, sign_of};
        use crate::decide::{NoSystem, Reason, Verdict, decide};
        use crate::gathered::Candidate;

        /// Every reason to keep at once: another owner, a socket and a foreign entry inside.
        fn kept(row: &crate::tools::Cache) -> Candidate {
            let inside = Inside { special: true, foreign: true, ..Inside::EMPTY };
            Candidate {
                facts: EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE },
                inside: Some(inside),
                ..candidate(row)
            }
        }

        #[test]
        fn not_a_folder_comes_before_every_other_row() {
            for row in rows() {
                let found = Candidate { kind: EntryKind::Symlink, ..kept(&row) };

                assert_eq!(
                    decide(&found, &row, Some(&sign_of(&row, &found)), Err(NoSystem)),
                    Verdict::NotClaimed,
                    "{}",
                    row.id
                );
            }
        }

        #[test]
        fn a_missing_sign_comes_before_the_inside() {
            for row in rows() {
                let found = kept(&row);

                assert_eq!(decide(&found, &row, None, Err(NoSystem)), Verdict::NotClaimed, "{}", row.id);
            }
        }

        #[test]
        fn special_comes_before_not_yours() {
            let npm = row("npm");
            let found = kept(&npm);

            assert_eq!(decide(&found, &npm, Some(&found.sign), Ok(&running(&[]))), Verdict::LeftOut(Reason::Special));
        }

        #[test]
        fn not_yours_comes_before_the_app_rows() {
            let xcode = row("xcode-derived-data");
            let found =
                Candidate { facts: EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE }, ..candidate(&xcode) };
            let open = running(&[XCODE]);

            assert_eq!(decide(&found, &xcode, Some(&found.sign), Err(NoSystem)), Verdict::LeftOut(Reason::NotYours));
            assert_eq!(decide(&found, &xcode, Some(&found.sign), Ok(&open)), Verdict::LeftOut(Reason::NotYours));
        }
    }
}

mod positive {
    use fresher_core::{EntryFacts, Inside};

    use super::{XCODE, candidate, row, rows, running, sign_of};
    use crate::decide::{NoSystem, Verdict, decide};
    use crate::gathered::{Candidate, Holder, Sign};
    use crate::home::place;

    #[test]
    fn every_rows_cache_with_its_sign_is_offered() {
        let none = running(&[]);
        for row in rows() {
            let found = candidate(&row);

            assert_eq!(decide(&found, &row, Some(&sign_of(&row, &found)), Ok(&none)), Verdict::Offered, "{}", row.id);
        }
    }

    #[test]
    fn a_sign_with_extra_names_is_offered() {
        let none = running(&[]);
        for row in rows() {
            let found = candidate(&row);
            let mut sign = sign_of(&row, &found);
            sign.holds.insert(0, "extra".into());

            assert_eq!(decide(&found, &row, Some(&sign), Ok(&none)), Verdict::Offered, "{}", row.id);
        }
    }

    #[test]
    fn a_sign_holder_that_is_a_folder_gives_its_sign() {
        let cargo = row("cargo-cache");
        let holder = Holder { index: 0, kind: fresher_core::EntryKind::Folder, sign: super::full_sign(&cargo) };

        assert_eq!(decide(&candidate(&cargo), &cargo, holder.sign(), Ok(&running(&[]))), Verdict::Offered);
    }

    #[test]
    fn a_folder_holding_lock_named_files_is_offered() {
        let gradle = row("gradle");
        let found = Candidate { inside: Some(Inside { lock_named: true, ..Inside::EMPTY }), ..candidate(&gradle) };

        assert_eq!(decide(&found, &gradle, Some(&found.sign), Ok(&running(&[]))), Verdict::Offered);
    }

    #[test]
    fn a_folder_the_app_cannot_write_is_still_the_users_and_offered() {
        let pip = row("pip");
        let facts = EntryFacts { parent_writable: false, folder_writable: false, ..EntryFacts::REMOVABLE };
        let found = Candidate { facts, ..candidate(&pip) };

        assert_eq!(decide(&found, &pip, Some(&found.sign), Ok(&running(&[]))), Verdict::Offered);
    }

    #[test]
    fn a_row_with_no_app_is_offered_when_the_running_apps_cannot_be_listed_or_xcode_runs() {
        let npm = row("npm");
        let found = candidate(&npm);

        assert_eq!(decide(&found, &npm, Some(&found.sign), Err(NoSystem)), Verdict::Offered);
        assert_eq!(decide(&found, &npm, Some(&found.sign), Ok(&running(&[XCODE]))), Verdict::Offered);
    }

    #[test]
    fn derived_data_is_offered_while_other_apps_run() {
        let xcode = row("xcode-derived-data");
        let found = candidate(&xcode);
        let others = running(&["com.example.Editor", "com.apple.dt.XcodeHelper"]);

        assert_eq!(decide(&found, &xcode, Some(&found.sign), Ok(&others)), Verdict::Offered);
    }

    #[test]
    fn a_project_with_dashes_in_its_name_is_offered() {
        let xcode = row("xcode-derived-data");
        let path = place(&xcode).with_file_name(format!("My-App-{}", crate::home::HASH));
        let found = Candidate { path, ..candidate(&xcode) };

        assert_eq!(
            decide(&found, &xcode, Some(&Sign { holds: vec!["info.plist".into()], marker: None }), Ok(&running(&[]))),
            Verdict::Offered
        );
    }
}

mod property {
    use fresher_core::{BundleId, EntryFacts, EntryKind, Inside, MarkerCheck, RunningApp, RunningSet};
    use proptest::prelude::*;

    use super::{candidate, rows};
    use crate::decide::{Verdict, decide};
    use crate::gathered::{Candidate, Sign};
    use crate::home::place;

    /// `true` most of the time: most generated cases are offered unless a property's change keeps them.
    fn mostly() -> proptest::bool::Weighted {
        proptest::bool::weighted(0.9)
    }

    /// `true` now and then.
    fn rarely() -> proptest::bool::Weighted {
        proptest::bool::weighted(0.1)
    }

    /// A candidate of any row in any state, most often an offered one, with most of the row's sign and other names.
    fn any_case() -> impl Strategy<Value = (usize, Candidate, Sign)> {
        let kind = prop_oneof![
            6 => Just(EntryKind::Folder),
            1 => Just(EntryKind::File),
            1 => Just(EntryKind::Symlink),
            1 => Just(EntryKind::Other)
        ];
        let facts = (mostly(), rarely(), rarely(), rarely()).prop_map(|(owned, user_lock, system_lock, restricted)| {
            EntryFacts {
                owned_by_user: owned,
                user_immutable: user_lock,
                system_immutable: system_lock,
                restricted,
                ..EntryFacts::REMOVABLE
            }
        });
        let inside =
            proptest::option::weighted(
                0.9,
                (rarely(), rarely(), any::<bool>(), mostly()).prop_map(|(special, foreign, lock_named, complete)| {
                    Inside { special, foreign, lock_named, complete, ..Inside::EMPTY }
                }),
            );
        let marker = proptest::option::weighted(
            0.9,
            prop_oneof![4 => Just(MarkerCheck::Valid), 1 => Just(MarkerCheck::Invalid)],
        );
        let name = prop_oneof![
            Just("Demo-abcdefghijklmnopqrstuvwxyzab"),
            Just("Demo-short"),
            Just("My-App-zyxwvutsrqponmlkjihgfedcba")
        ];
        let kept = proptest::collection::vec(mostly(), 2);
        let extra = proptest::collection::vec("[a-z.]{1,8}", 0..3);
        (0..rows().len(), kind, facts, inside, marker, kept, extra, name).prop_map(
            |(index, kind, facts, inside, marker, kept, extra, name)| {
                let row = &rows()[index];
                let path = if row.name.is_some() { place(row).with_file_name(name) } else { place(row) };
                let mut holds: Vec<String> =
                    row.sign.iter().zip(kept).filter(|(_, kept)| *kept).map(|(name, _)| name.clone()).collect();
                holds.extend(extra);
                let found = Candidate { path, kind, facts, inside, ..candidate(row) };
                (index, found, Sign { holds, marker })
            },
        )
    }

    /// The bundle ids of any running apps.
    fn any_running() -> impl Strategy<Value = Vec<Vec<String>>> {
        proptest::collection::vec(proptest::collection::vec("[a-z]{1,6}(\\.[a-zA-Z]{1,6}){1,3}", 1..3), 0..4)
    }

    /// The apps of `ids`, each without `app`.
    fn running_without(ids: Vec<Vec<String>>, app: Option<&BundleId>) -> RunningSet {
        let apps = ids
            .into_iter()
            .map(|ids| RunningApp {
                name: "App".into(),
                bundle_ids: ids
                    .into_iter()
                    .filter_map(|id| BundleId::new(id).ok())
                    .filter(|id| Some(id) != app)
                    .collect(),
            })
            .collect();
        RunningSet::new(apps)
    }

    proptest! {
        #[test]
        fn adding_names_to_a_sign_never_turns_an_offer_into_a_keep(
            (index, found, sign) in any_case(),
            added in proptest::collection::vec("[a-zA-Z.-]{1,10}", 1..4),
        ) {
            let row = &rows()[index];
            let none = RunningSet::default();
            let mut more = sign.clone();
            more.holds.extend(added);
            more.holds.reverse();

            if decide(&found, row, Some(&sign), Ok(&none)) == Verdict::Offered {
                prop_assert_eq!(decide(&found, row, Some(&more), Ok(&none)), Verdict::Offered);
            }
        }

        #[test]
        fn a_running_set_without_the_rows_app_never_keeps_what_the_empty_set_offers(
            (index, found, sign) in any_case(),
            ids in any_running(),
        ) {
            let row = &rows()[index];
            let none = RunningSet::default();
            let others = running_without(ids, row.app.as_ref());

            if decide(&found, row, Some(&sign), Ok(&none)) == Verdict::Offered {
                prop_assert_eq!(decide(&found, row, Some(&sign), Ok(&others)), Verdict::Offered);
            }
        }
    }
}
