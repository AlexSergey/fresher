//! The decision on plain values: one test per row, each keep before the offers, then a property.

use std::cell::Cell;
use std::path::Path;

use fresher_core::{BundleId, ByteSize, EntryFacts, EntryKind, Inside, InstalledApp, RunningApp, RunningSet, Sizes};

use crate::decide::{NoSystem, Verdict, decide};
use crate::exclude::Exclude;
use crate::gathered::Candidate;

const CACHES: &str = "/home/Library/Caches";
const EDITOR: &str = "com.example.Editor";
const HELPER: &str = "com.example.Editor.Helper";

/// An installed app of `id` named `name`, with the helpers `helpers`.
fn app(id: &str, name: &str, helpers: &[&str]) -> InstalledApp {
    let app = fresher_testing::installed_app(id, name, "1.0").unwrap();
    InstalledApp { helpers: helpers.iter().map(|helper| BundleId::new(*helper).unwrap()).collect(), ..app }
}

/// Editor with its helper, a viewer, Safari and Fresher itself.
fn installed() -> Vec<InstalledApp> {
    vec![
        app(EDITOR, "Editor", &[HELPER]),
        app("com.example.Viewer", "Viewer", &[]),
        app("com.apple.Safari", "Safari", &[]),
        app("io.github.alexsergey.fresher", "Fresher", &["io.github.alexsergey.fresher.dev"]),
    ]
}

/// A folder of the user's named `name` in `Caches`, read in full, of 1000 logical bytes.
fn folder(name: &str) -> Candidate {
    Candidate {
        row: 0,
        path: Path::new(CACHES).join(name),
        kind: EntryKind::Folder,
        sizes: Sizes::complete(ByteSize::new(1000), ByteSize::new(4096)),
        facts: EntryFacts::REMOVABLE,
        inside: Some(Inside::EMPTY),
    }
}

fn running(apps: &[(&str, &[&str])]) -> RunningSet {
    let apps: Vec<RunningApp> =
        apps.iter().map(|(name, ids)| fresher_testing::running_app(name, ids).unwrap()).collect();
    RunningSet::new(apps)
}

/// The verdict on `candidate` with `apps` installed and `open` running, and how many times the running apps were
/// asked.
fn verdict<'a>(
    candidate: &Candidate,
    apps: Result<&'a [InstalledApp], NoSystem>,
    open: Result<&RunningSet, NoSystem>,
) -> (Verdict<'a>, usize) {
    let (exclude, asked) = (Exclude::shipped().unwrap(), Cell::new(0));
    let found = decide(candidate, &exclude, apps, || {
        asked.set(asked.get() + 1);
        open
    });
    (found, asked.get())
}

mod negative {
    use fresher_core::{EntryFacts, EntryKind, Inside, Sizes};

    use super::{EDITOR, HELPER, folder, installed, running, verdict};
    use crate::decide::{NoSystem, Reason, Verdict};
    use crate::gathered::Candidate;

    #[test]
    fn a_file_or_a_link_is_not_claimed() {
        let (apps, none) = (installed(), running(&[]));
        for kind in [EntryKind::File, EntryKind::Symlink, EntryKind::Other] {
            let found = Candidate { kind, ..folder(EDITOR) };

            assert_eq!(verdict(&found, Ok(&apps), Ok(&none)), (Verdict::NotClaimed, 0), "{kind:?}");
        }
    }

    #[test]
    fn a_name_that_is_not_an_id_is_not_claimed() {
        let (apps, none) = (installed(), running(&[]));
        for name in ["Editor", "JetBrains", "com example.Editor"] {
            assert_eq!(verdict(&folder(name), Ok(&apps), Ok(&none)), (Verdict::NotClaimed, 0), "{name}");
        }
    }

    #[test]
    fn an_excluded_id_is_not_claimed() {
        let (apps, none) = (installed(), running(&[]));
        for name in
            ["com.apple.Safari", "COM.APPLE.Safari", "io.github.alexsergey.fresher", "io.github.alexsergey.fresher.dev"]
        {
            assert_eq!(verdict(&folder(name), Ok(&apps), Ok(&none)), (Verdict::NotClaimed, 0), "{name}");
        }
    }

    #[test]
    fn nothing_is_claimed_when_the_installed_apps_cannot_be_listed() {
        let none = running(&[]);
        for name in [EDITOR, HELPER, "com.example.Gone"] {
            assert_eq!(verdict(&folder(name), Err(NoSystem), Ok(&none)), (Verdict::NotClaimed, 0), "{name}");
        }
    }

    #[test]
    fn a_removed_app_s_cache_is_not_claimed() {
        let (apps, none) = (installed(), running(&[]));
        for name in ["com.example.Gone", "com.example", "com.example.Editorx", "com.example.Viewerx"] {
            assert_eq!(verdict(&folder(name), Ok(&apps), Ok(&none)), (Verdict::NotClaimed, 0), "{name}");
        }
    }

    #[test]
    fn a_folder_read_in_part_is_not_claimed() {
        let (apps, none) = (installed(), running(&[]));
        for inside in [None, Some(Inside { complete: false, ..Inside::EMPTY })] {
            let found = Candidate { inside, ..folder(EDITOR) };

            assert_eq!(verdict(&found, Ok(&apps), Ok(&none)), (Verdict::NotClaimed, 0), "{inside:?}");
        }
    }

    #[test]
    fn an_empty_folder_is_not_claimed() {
        let (apps, none) = (installed(), running(&[]));
        let empty = Candidate { sizes: Sizes::ZERO, ..folder(EDITOR) };

        assert_eq!(verdict(&empty, Ok(&apps), Ok(&none)), (Verdict::NotClaimed, 0));
    }

    #[test]
    fn a_socket_inside_keeps_it_special() {
        let (apps, none) = (installed(), running(&[]));
        let inside = Some(Inside { special: true, foreign: true, ..Inside::EMPTY });
        let found = Candidate { inside, ..folder(EDITOR) };

        assert_eq!(verdict(&found, Ok(&apps), Ok(&none)), (Verdict::LeftOut(Reason::Special), 0));
    }

    #[test]
    fn an_entry_not_yours_keeps_it() {
        let (apps, none) = (installed(), running(&[]));
        let theirs = EntryFacts { owned_by_user: false, ..EntryFacts::REMOVABLE };
        let flagged = EntryFacts { system_immutable: true, ..EntryFacts::REMOVABLE };
        let restricted = EntryFacts { restricted: true, ..EntryFacts::REMOVABLE };
        let locked = EntryFacts { user_immutable: true, ..EntryFacts::REMOVABLE };
        for facts in [theirs, flagged, restricted, locked] {
            let found = Candidate { facts, ..folder(EDITOR) };

            assert_eq!(verdict(&found, Ok(&apps), Ok(&none)), (Verdict::LeftOut(Reason::NotYours), 0), "{facts:?}");
        }
        let foreign = Candidate { inside: Some(Inside { foreign: true, ..Inside::EMPTY }), ..folder(EDITOR) };
        assert_eq!(verdict(&foreign, Ok(&apps), Ok(&none)), (Verdict::LeftOut(Reason::NotYours), 0));
    }

    #[test]
    fn it_is_kept_when_the_running_apps_cannot_be_listed() {
        let apps = installed();

        assert_eq!(verdict(&folder(EDITOR), Ok(&apps), Err(NoSystem)), (Verdict::LeftOut(Reason::NoSystem), 1));
    }

    #[test]
    fn it_is_kept_while_its_app_runs() {
        let apps = installed();
        let open = running(&[("Mail", &["com.example.Mail"]), ("Editor", &[EDITOR])]);
        for name in [EDITOR, HELPER, "com.example.Editor.Cache"] {
            assert_eq!(verdict(&folder(name), Ok(&apps), Ok(&open)), (Verdict::LeftOut(Reason::AppOpen), 1), "{name}");
        }
    }

    #[test]
    fn it_is_kept_while_a_helper_runs() {
        let apps = installed();
        let helper = running(&[("Editor Helper", &[HELPER])]);

        assert_eq!(verdict(&folder(EDITOR), Ok(&apps), Ok(&helper)), (Verdict::LeftOut(Reason::AppOpen), 1));
    }

    #[test]
    fn it_is_kept_while_a_running_app_s_id_names_the_folder() {
        let apps = installed();
        let unlisted = running(&[("Editor Cache Service", &["com.example.Editor.Cache"])]);

        let found = verdict(&folder("com.example.Editor.Cache"), Ok(&apps), Ok(&unlisted));

        assert_eq!(found, (Verdict::LeftOut(Reason::AppOpen), 1));
    }
}

mod positive {
    use fresher_core::{EntryFacts, Inside};

    use super::{EDITOR, HELPER, folder, installed, running, verdict};
    use crate::decide::Verdict;
    use crate::gathered::Candidate;
    use crate::owner::Owner;

    #[test]
    fn an_installed_app_s_cache_is_offered() {
        let apps = installed();
        let others = running(&[("Mail", &["com.example.Mail"]), ("Viewer", &["com.example.Viewer"])]);
        let lock_named = Candidate { inside: Some(Inside { lock_named: true, ..Inside::EMPTY }), ..folder(EDITOR) };
        let cache = folder("com.example.Viewer.Cache");

        let own = Verdict::Offered(Owner { app: &apps[0], own: true });
        assert_eq!(verdict(&folder(EDITOR), Ok(&apps), Ok(&others)), (own, 1));
        assert_eq!(verdict(&folder("COM.EXAMPLE.EDITOR"), Ok(&apps), Ok(&others)), (own, 1));
        assert_eq!(verdict(&lock_named, Ok(&apps), Ok(&others)), (own, 1));
        let part = Verdict::Offered(Owner { app: &apps[0], own: false });
        assert_eq!(verdict(&folder("com.example.Editor.Cache"), Ok(&apps), Ok(&others)), (part, 1));
        let unrelated = running(&[("Editor", &["org.example.Editor"])]);
        assert_eq!(verdict(&cache, Ok(&apps), Ok(&unrelated)).0, Verdict::Offered(Owner { app: &apps[1], own: false }));
    }

    #[test]
    fn a_helper_s_cache_is_offered_in_its_app_s_name() {
        let apps = installed();
        let facts = EntryFacts::REMOVABLE;

        let found = verdict(&Candidate { facts, ..folder(HELPER) }, Ok(&apps), Ok(&running(&[])));

        assert_eq!(found, (Verdict::Offered(Owner { app: &apps[0], own: false }), 1));
        let Verdict::Offered(owner) = found.0 else { unreachable!() };
        assert_eq!(owner.app.bundle_id.as_str(), EDITOR);
        assert_eq!(owner.app.name, "Editor");
    }
}

mod property {
    use fresher_core::{ByteSize, EntryFacts, EntryKind, Inside, RunningSet, Sizes};
    use proptest::prelude::*;

    use super::{folder, installed, running, verdict};
    use crate::decide::{NoSystem, Reason, Verdict};
    use crate::gathered::Candidate;

    /// A folder of `Caches` of any name and state, most often one an installed app owns.
    fn any_candidate() -> impl Strategy<Value = Candidate> {
        let kind = prop_oneof![6 => Just(EntryKind::Folder), 1 => Just(EntryKind::File), 1 => Just(EntryKind::Symlink)];
        let name = prop_oneof![
            Just("com.example.Editor".to_owned()),
            Just("com.example.Editor.Helper".to_owned()),
            Just("com.example.Viewer.Cache".to_owned()),
            Just("com.apple.Safari".to_owned()),
            Just("com.example.Gone".to_owned()),
            Just("com.example".to_owned()),
            "[A-Za-z.]{0,16}",
        ];
        let flags = proptest::collection::vec(proptest::bool::weighted(0.1), 6);
        let read = prop_oneof![1 => Just(None), 8 => proptest::bool::weighted(0.9).prop_map(Some)];
        (kind, name, flags, read, 0..3_u64).prop_map(|(kind, name, flags, read, bytes)| {
            let facts = EntryFacts {
                owned_by_user: !flags[0],
                user_immutable: flags[1],
                system_immutable: flags[2],
                restricted: flags[3],
                ..EntryFacts::REMOVABLE
            };
            let inside =
                read.map(|complete| Inside { special: flags[4], foreign: flags[5], complete, ..Inside::EMPTY });
            let sizes = Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes));
            Candidate { kind, sizes, facts, inside, ..folder(&name) }
        })
    }

    proptest! {
        #[test]
        fn a_candidate_stopped_before_row_9_never_asks_the_running_apps(
            found in any_candidate(),
            listed in proptest::bool::weighted(0.9),
            open in proptest::option::weighted(0.8, proptest::bool::ANY),
        ) {
            let apps = installed();
            let apps = if listed { Ok(apps.as_slice()) } else { Err(NoSystem) };
            let editor: RunningSet = running(&[("Editor", &["com.example.Editor"])]);
            let none = running(&[]);
            let open = match open {
                None => Err(NoSystem),
                Some(true) => Ok(&editor),
                Some(false) => Ok(&none),
            };

            let (decided, asked) = verdict(&found, apps, open);

            let stopped = matches!(decided, Verdict::NotClaimed | Verdict::LeftOut(Reason::Special | Reason::NotYours));
            prop_assert_eq!(asked, usize::from(!stopped));
        }
    }
}
