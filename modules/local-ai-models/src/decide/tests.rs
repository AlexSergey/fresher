//! The decision on plain values: each row's keep before the offers, then the offers, the date, and two properties.

use std::path::Path;

use fresher_core::{EntryFacts, EntryKind, Inside, RunningApp, RunningSet, Sizes, Timestamp};

use crate::gathered::Candidate;
use crate::stores::{Store, Stores};

const LM_STUDIO: &str = "ai.elementlabs.lmstudio";
const HUB: &str = "/home/.cache/huggingface/hub";
const LM_MODELS: &str = "/home/.lmstudio/models";
const CHANGED: u64 = 1_700_000_000;

fn row(id: &str) -> Store {
    Stores::shipped().unwrap().stores().iter().find(|row| row.id == id).cloned().unwrap()
}

/// A folder of the user's at `path`, read in full, changed once, holding `row`'s whole sign.
fn candidate(row: &Store, path: &Path) -> Candidate {
    Candidate {
        row: 0,
        ollama: None,
        path: path.to_path_buf(),
        kind: EntryKind::Folder,
        sizes: Sizes::ZERO,
        modified: None,
        accessed: None,
        facts: EntryFacts::REMOVABLE,
        holds: row.sign.clone(),
        inside: Some(Inside { newest_change: Some(Timestamp::from_secs(CHANGED)), ..Inside::EMPTY }),
    }
}

/// A Hugging Face repository folder named `name`.
fn repo(name: &str) -> (Store, Candidate) {
    let hub = row("hugging-face");
    let found = candidate(&hub, &Path::new(HUB).join(name));
    (hub, found)
}

/// An LM Studio model folder of the row `id`.
fn lm_model(id: &str) -> (Store, Candidate) {
    let lm = row(id);
    let found = candidate(&lm, &Path::new(LM_MODELS).join("lmstudio-community").join("Qwen2.5-7B-Instruct-GGUF"));
    (lm, found)
}

/// A model folder of each row, as `candidate` makes it.
fn every_model() -> Vec<(Store, Candidate)> {
    vec![repo("models--google-bert--bert-base-uncased"), lm_model("lm-studio"), lm_model("lm-studio-old")]
}

fn running(ids: &[&str]) -> RunningSet {
    let apps: Vec<RunningApp> = ids.iter().map(|id| fresher_testing::running_app("App", &[id]).unwrap()).collect();
    RunningSet::new(apps)
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, Inside};

    use super::{LM_STUDIO, every_model, lm_model, repo, running};
    use crate::decide::{NoSystem, Reason, Verdict, decide};
    use crate::gathered::Candidate;

    #[test]
    fn a_file_or_a_link_is_not_claimed() {
        let none = running(&[]);
        for (row, found) in every_model() {
            for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
                let found = Candidate { kind, ..found.clone() };

                assert_eq!(decide(&found, &row, Ok(&none)), Verdict::NotClaimed, "{} as {kind:?}", row.id);
            }
        }
    }

    #[test]
    fn a_space_is_not_claimed() {
        for name in ["spaces--owner--demo", "models--a--b--c", ".locks", "version.txt"] {
            let (hub, found) = repo(name);

            assert_eq!(decide(&found, &hub, Ok(&running(&[]))), Verdict::NotClaimed, "{name}");
        }
    }

    #[test]
    fn a_repo_without_blobs_or_snapshots_is_not_claimed() {
        let (hub, found) = repo("models--gpt2");
        for holds in [vec![], vec!["blobs".to_owned()], vec!["snapshots".to_owned()], vec!["refs".to_owned()]] {
            let found = Candidate { holds: holds.clone(), ..found.clone() };

            assert_eq!(decide(&found, &hub, Ok(&running(&[]))), Verdict::NotClaimed, "{holds:?}");
        }
    }

    #[test]
    fn a_folder_read_in_part_is_not_claimed() {
        let none = running(&[]);
        let timeless = Inside { complete: true, ..Inside::EMPTY };
        for (row, found) in every_model() {
            let partial = found.inside.map(|inside| Inside { complete: false, ..inside });
            for inside in [None, partial, Some(timeless)] {
                let found = Candidate { inside, ..found.clone() };

                assert_eq!(decide(&found, &row, Ok(&none)), Verdict::NotClaimed, "{} with {inside:?}", row.id);
            }
        }
    }

    #[test]
    fn a_socket_inside_keeps_it_special() {
        for (row, found) in every_model() {
            let inside = found.inside.map(|inside| Inside { special: true, foreign: true, ..inside });
            let found = Candidate { inside, ..found };

            assert_eq!(decide(&found, &row, Ok(&running(&[]))), Verdict::LeftOut(Reason::Special), "{}", row.id);
        }
    }

    #[test]
    fn an_entry_not_yours_keeps_it() {
        let (hub, found) = repo("models--gpt2");
        let theirs = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
        let flagged = EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE };
        let restricted = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
        let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        for facts in [theirs, flagged, restricted, locked] {
            let found = Candidate { facts, ..found.clone() };

            assert_eq!(decide(&found, &hub, Ok(&running(&[]))), Verdict::LeftOut(Reason::NotYours), "{facts:?}");
        }
        let inside = found.inside.map(|inside| Inside { foreign: true, ..inside });
        let foreign = Candidate { inside, ..found };
        assert_eq!(decide(&foreign, &hub, Ok(&running(&[]))), Verdict::LeftOut(Reason::NotYours));

        let (lm, found) = lm_model("lm-studio");
        let locked = Candidate { facts: locked, ..found };
        assert_eq!(decide(&locked, &lm, Err(NoSystem)), Verdict::LeftOut(Reason::NotYours));
    }

    #[test]
    fn lm_studio_s_model_is_kept_when_the_apps_cannot_be_listed() {
        for id in ["lm-studio", "lm-studio-old"] {
            let (lm, found) = lm_model(id);

            assert_eq!(decide(&found, &lm, Err(NoSystem)), Verdict::LeftOut(Reason::NoSystem), "{id}");
        }
    }

    #[test]
    fn lm_studio_s_model_is_kept_while_it_runs() {
        let open = running(&["com.example.Editor", LM_STUDIO]);
        for id in ["lm-studio", "lm-studio-old"] {
            let (lm, found) = lm_model(id);

            assert_eq!(decide(&found, &lm, Ok(&open)), Verdict::LeftOut(Reason::AppOpen), "{id}");
        }
    }
}

mod positive {
    use fresher_core::{Inside, Timestamp};

    use super::{CHANGED, LM_STUDIO, lm_model, repo, running};
    use crate::decide::{NoSystem, Verdict, decide, last_seen};
    use crate::gathered::Candidate;
    use crate::name::ModelName;

    fn offered(category: &'static str, kind: Option<&str>, title: &str) -> Verdict {
        let name = ModelName { kind: kind.map(str::to_owned), title: title.to_owned() };
        Verdict::Offered { category, name, seen: Timestamp::from_secs(CHANGED) }
    }

    #[test]
    fn a_hugging_face_model_is_offered_in_the_models_category() {
        let (hub, found) = repo("models--google-bert--bert-base-uncased");
        let mut more = found.clone();
        more.holds.push("refs".into());
        let lock_named =
            Candidate { inside: found.inside.map(|inside| Inside { lock_named: true, ..inside }), ..found.clone() };
        let expected = offered("local-ai-models.hugging-face-models", Some("models"), "google-bert/bert-base-uncased");

        assert_eq!(decide(&found, &hub, Err(NoSystem)), expected);
        assert_eq!(decide(&more, &hub, Ok(&running(&[LM_STUDIO]))), expected);
        assert_eq!(decide(&lock_named, &hub, Err(NoSystem)), expected);
    }

    #[test]
    fn a_dataset_is_offered_in_the_datasets_category() {
        let (hub, found) = repo("datasets--squad");

        assert_eq!(
            decide(&found, &hub, Err(NoSystem)),
            offered("local-ai-models.hugging-face-datasets", Some("datasets"), "squad")
        );
    }

    #[test]
    fn an_lm_studio_model_is_offered_at_medium() {
        let others = running(&["com.example.Editor"]);
        for id in ["lm-studio", "lm-studio-old"] {
            let (lm, found) = lm_model(id);

            let verdict = decide(&found, &lm, Ok(&others));

            assert_eq!(
                verdict,
                offered("local-ai-models.lm-studio", None, "lmstudio-community/Qwen2.5-7B-Instruct-GGUF"),
                "{id}"
            );
            let danger =
                crate::categories::CATEGORIES.iter().find(|(category, _)| *category == "local-ai-models.lm-studio");
            assert_eq!(danger.map(|(_, danger)| *danger), Some(fresher_core::DangerLevel::Medium));
        }
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
    use fresher_core::{EntryFacts, EntryKind, Inside, RunningApp, RunningSet, Timestamp};
    use proptest::prelude::*;

    use super::{every_model, row};
    use crate::decide::{NoSystem, decide};
    use crate::gathered::Candidate;

    /// A Hugging Face folder of any name and state, most often an offered one.
    fn any_hub_folder() -> impl Strategy<Value = Candidate> {
        let kind = prop_oneof![6 => Just(EntryKind::Folder), 1 => Just(EntryKind::File), 1 => Just(EntryKind::Symlink)];
        let name = prop_oneof![
            Just("models--owner--name".to_owned()),
            Just("datasets--name".to_owned()),
            Just("spaces--owner--name".to_owned()),
            "[a-z.-]{0,12}",
        ];
        let holds =
            proptest::sample::subsequence(vec!["blobs".to_owned(), "snapshots".to_owned(), "refs".to_owned()], 0..=3);
        let flags = proptest::collection::vec(proptest::bool::weighted(0.1), 6);
        let times = (proptest::option::of(0..u64::MAX), proptest::option::of(0..u64::MAX));
        (kind, name, holds, flags, times, proptest::bool::weighted(0.9)).prop_map(
            |(kind, name, holds, flags, (change, access), complete)| {
                let (_, model) = every_model().remove(0);
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
                Candidate { path: model.path.with_file_name(name), kind, holds, facts, inside: Some(inside), ..model }
            },
        )
    }

    /// Any running apps, LM Studio among them or not.
    fn any_running() -> impl Strategy<Value = RunningSet> {
        let ids = prop_oneof![Just("ai.elementlabs.lmstudio".to_owned()), "[a-z]{1,6}(\\.[a-z]{1,6}){1,2}"];
        proptest::collection::vec(ids, 0..4).prop_map(|ids| {
            let apps: Vec<RunningApp> =
                ids.iter().filter_map(|id| fresher_testing::running_app("App", &[id.as_str()]).ok()).collect();
            RunningSet::new(apps)
        })
    }

    proptest! {
        #[test]
        fn a_row_without_an_app_never_asks_the_running_apps(found in any_hub_folder(), running in any_running()) {
            let hub = row("hugging-face");
            prop_assert!(hub.app.is_none());

            prop_assert_eq!(decide(&found, &hub, Ok(&running)), decide(&found, &hub, Err(NoSystem)));
        }
    }
}
