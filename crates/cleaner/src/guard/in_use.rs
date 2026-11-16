//! Check 10: no process of the user holds the item or anything inside it, in the snapshot the pipeline keeps fresh.

use fresher_core::{OpenSet, SkipReason};

use crate::paths::Placed;

/// Check 10: `InUse` with the holder's name when either form of the path, or anything inside it, is held open.
pub(super) fn check(placed: &Placed, open: &OpenSet) -> Result<(), SkipReason> {
    match [&placed.physical, &placed.lexical].into_iter().find_map(|path| open.holder_under(path)) {
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
        use fresher_core::{OpenSet, SkipReason};

        use super::placed;
        use crate::guard::in_use::check;

        #[test]
        fn a_held_file_inside_a_folder_makes_it_in_use() {
            let open = OpenSet::new(vec![("/private/var/home/cache/db.sqlite".into(), "Notes".into())]);

            assert_eq!(check(&placed(), &open), Err(SkipReason::InUse { app: "Notes".into() }));
        }

        #[test]
        fn a_held_folder_is_in_use() {
            let open = OpenSet::new(vec![("/var/home/cache".into(), "Terminal".into())]);

            assert_eq!(check(&placed(), &open), Err(SkipReason::InUse { app: "Terminal".into() }));
        }
    }

    mod positive {
        use fresher_core::OpenSet;

        use super::placed;
        use crate::guard::in_use::check;

        #[test]
        fn a_file_held_beside_the_item_does_not_count() {
            let open = OpenSet::new(vec![
                ("/private/var/home/cache2/db".into(), "Notes".into()),
                ("/private/var/home".into(), "Finder".into()),
            ]);

            assert_eq!(check(&placed(), &open), Ok(()));
        }
    }
}
