//! Check 10: no process of the user holds the item or anything inside it, and no running app owns it, in the
//! snapshots the pipeline keeps fresh.

use fresher_core::{OpenSet, RunningSet, SkipReason};

use crate::paths::Placed;

/// Check 10: `InUse` with the app's name when either form of the path, or anything inside it, is held open, or when
/// a running app owns it by bundle id (a name of the path is the app's id, or starts with it and a dot): the app may
/// write there any moment, whether or not its open files can be seen.
pub(super) fn check(placed: &Placed, open: &OpenSet, running: &RunningSet) -> Result<(), SkipReason> {
    let forms = [&placed.physical, &placed.lexical];
    let holder = forms.into_iter().find_map(|path| open.holder_under(path));
    match holder.or_else(|| forms.into_iter().find_map(|path| running.owner_of(path))) {
        Some(app) => Err(SkipReason::InUse { app: app.to_owned() }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::paths::Placed;

    fn placed() -> Placed {
        Placed {
            lexical: PathBuf::from("/var/home/cache"),
            physical: PathBuf::from("/private/var/home/cache"),
            root: PathBuf::from("/private/var/home"),
            device: 1,
            relative: Vec::new(),
        }
    }

    mod negative {
        use fresher_core::{BundleId, OpenSet, RunningApp, RunningSet, SkipReason};

        use super::placed;
        use crate::guard::in_use::check;

        #[test]
        fn a_held_file_inside_a_folder_makes_it_in_use() {
            let open = OpenSet::new(vec![("/private/var/home/cache/db.sqlite".into(), "Notes".into())]);

            assert_eq!(check(&placed(), &open, &RunningSet::default()), Err(SkipReason::InUse { app: "Notes".into() }));
        }

        #[test]
        fn an_item_named_after_a_running_app_or_inside_its_folder_is_in_use() {
            let running = RunningSet::new(vec![RunningApp {
                name: "Notes".into(),
                bundle_ids: vec![BundleId::new("com.example.Notes").unwrap()],
            }]);
            let owned = |lexical: &str| crate::paths::Placed {
                lexical: lexical.into(),
                physical: format!("/private{lexical}").into(),
                ..placed()
            };

            for path in [
                "/var/home/com.example.Notes",
                "/var/home/com.example.notes/Cache.db",
                "/var/home/com.example.Notes.plist",
            ] {
                let found = check(&owned(path), &OpenSet::default(), &running);
                assert_eq!(found, Err(SkipReason::InUse { app: "Notes".into() }), "{path}");
            }
        }

        #[test]
        fn a_held_folder_is_in_use() {
            let open = OpenSet::new(vec![("/var/home/cache".into(), "Terminal".into())]);

            assert_eq!(
                check(&placed(), &open, &RunningSet::default()),
                Err(SkipReason::InUse { app: "Terminal".into() })
            );
        }
    }

    mod positive {
        use fresher_core::{BundleId, OpenSet, RunningApp, RunningSet};

        use super::placed;
        use crate::guard::in_use::check;

        #[test]
        fn a_file_held_beside_the_item_does_not_count() {
            let open = OpenSet::new(vec![
                ("/private/var/home/cache2/db".into(), "Notes".into()),
                ("/private/var/home".into(), "Finder".into()),
            ]);

            assert_eq!(check(&placed(), &open, &RunningSet::default()), Ok(()));
        }

        #[test]
        fn an_item_of_an_app_that_is_not_running_is_not_in_use() {
            let running = RunningSet::new(vec![RunningApp {
                name: "Mail".into(),
                bundle_ids: vec![BundleId::new("com.example.Mail").unwrap()],
            }]);

            assert_eq!(check(&placed(), &OpenSet::default(), &running), Ok(()));
        }
    }
}
