//! The places a scan walks outside its roots: the temporary folders (`KnownFolder::is_walked_place`), when a
//! registered module's interest is anchored in one, it lies outside the home, and no scan root holds it or lies in it.
//! The home's own folders are never places: the user chose which of them are scanned.

use std::path::PathBuf;

use fresher_core::{KnownFolder, KnownFolders, Pattern, Place, Protected, Protection};

use crate::registry::Registry;

/// Every walked-place folder of a registered module's `Under` interest, enabled or not (turning a module off never
/// changes what the walk reads), located by `folders`, that lies outside the home, neither inside nor around any of
/// `roots` (absolute), and that `protection` does not refuse as a place nothing may ever be removed from. Sorted, so
/// every scan asks the same.
pub(crate) fn walked_places(
    registry: &Registry,
    folders: &dyn KnownFolders,
    roots: &[PathBuf],
    protection: &Protection,
) -> Vec<Place> {
    let anchored = registry.registered().iter().flat_map(|module| module.interests()).filter_map(|interest| {
        match interest.pattern() {
            Pattern::Under(folder, _) if folder.is_walked_place() => Some(*folder),
            Pattern::Under(..) | Pattern::FolderName(_) | Pattern::MarkerFile(_) | Pattern::Extension(_) => None,
        }
    });
    let home = folders.locate(KnownFolder::Home);
    let mut places: Vec<Place> = anchored
        .filter_map(|folder| Some(Place { folder, path: folders.locate(folder)? }))
        .filter(|place| home.as_ref().is_none_or(|home| !place.path.starts_with(home)))
        .filter(|place| !roots.iter().any(|root| place.path.starts_with(root) || root.starts_with(&place.path)))
        .filter(|place| protection.check(&place.path) != Err(Protected::Never))
        .collect();
    places.sort();
    places.dedup();
    places
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use fresher_core::{Claim, Interest, KnownFolder, Pattern};
    use fresher_testing::FakeKnownFolders;

    use crate::registry::Registry;
    use crate::testing::TestModule;

    fn under(folder: KnownFolder) -> Interest {
        Interest::new(Pattern::Under(folder, "*".into()), Claim::Whole, Vec::new()).unwrap()
    }

    fn registry(interests: Vec<Interest>) -> Registry {
        let module = interests
            .into_iter()
            .fold(TestModule::builder("temp"), crate::testing::TestModuleBuilder::interest)
            .build();
        let mut registry = Registry::new().unwrap();
        registry.register(Box::new(module)).unwrap();
        registry
    }

    fn folders() -> FakeKnownFolders {
        FakeKnownFolders::new()
            .with(KnownFolder::Home, "/Users/me")
            .with(KnownFolder::AppCaches, "/Users/me/Library/Caches")
            .with(KnownFolder::UserTemp, "/private/var/folders/xx/hash/T")
            .with(KnownFolder::SharedTemp, "/private/tmp")
    }

    fn home() -> Vec<PathBuf> {
        vec![PathBuf::from("/Users/me")]
    }

    mod negative {
        use std::path::PathBuf;

        use fresher_core::{Claim, Interest, KnownFolder, Pattern, Protection};

        use super::{folders, home, registry, under};
        use crate::places::walked_places;

        #[test]
        fn no_module_naming_a_place_walks_none() {
            let by_name = Interest::new(Pattern::FolderName("tmp".into()), Claim::Whole, Vec::new()).unwrap();
            let registry = registry(vec![by_name, under(KnownFolder::AppCaches)]);

            assert_eq!(walked_places(&registry, &folders(), &home(), &Protection::default()), []);
        }

        #[test]
        fn a_place_inside_a_scan_root_is_not_walked_again() {
            let registry = registry(vec![under(KnownFolder::SharedTemp)]);
            let roots = [PathBuf::from("/private")];

            assert_eq!(walked_places(&registry, &folders(), &roots, &Protection::default()), []);
        }

        #[test]
        fn the_homes_own_folders_are_never_places_even_outside_the_scan_roots() {
            let registry = registry(vec![under(KnownFolder::Home), under(KnownFolder::AppCaches)]);
            let roots = [PathBuf::from("/Users/me/work")];

            assert_eq!(walked_places(&registry, &folders(), &roots, &Protection::default()), []);
        }

        #[test]
        fn only_the_temporary_folders_are_ever_places() {
            let registry = registry(vec![under(KnownFolder::Applications), under(KnownFolder::SystemApplications)]);
            let folders = folders()
                .with(KnownFolder::Applications, "/Applications")
                .with(KnownFolder::SystemApplications, "/System/Applications");

            assert_eq!(walked_places(&registry, &folders, &home(), &Protection::default()), []);
        }

        #[test]
        fn a_place_that_holds_a_scan_root_is_not_walked() {
            let registry = registry(vec![under(KnownFolder::SharedTemp)]);
            let roots = [PathBuf::from("/private/tmp/work")];

            assert_eq!(walked_places(&registry, &folders(), &roots, &Protection::default()), []);
        }

        #[test]
        fn a_place_protected_for_ever_is_not_walked() {
            let registry = registry(vec![under(KnownFolder::SharedTemp)]);
            let protection = Protection::new(vec![PathBuf::from("/private")], vec![], vec![]).unwrap();

            assert_eq!(walked_places(&registry, &folders(), &home(), &protection), []);
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{KnownFolder, Place, Protection};

        use super::{folders, home, registry, under};
        use crate::places::walked_places;

        #[test]
        fn the_temporary_folders_a_module_names_are_walked_in_order_once_each() {
            let registry = registry(vec![
                under(KnownFolder::SharedTemp),
                under(KnownFolder::UserTemp),
                under(KnownFolder::SharedTemp),
            ]);
            let whole = Protection::new(vec![], vec![PathBuf::from("/private/tmp")], vec![]).unwrap();

            let places = walked_places(&registry, &folders(), &home(), &whole);

            assert_eq!(
                places,
                [
                    Place { folder: KnownFolder::UserTemp, path: "/private/var/folders/xx/hash/T".into() },
                    Place { folder: KnownFolder::SharedTemp, path: "/private/tmp".into() },
                ]
            );
        }
    }
}
