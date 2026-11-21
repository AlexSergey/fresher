//! The places table: every refusal by a table made for it, then the shipped table.

use fresher_core::DangerLevel;

use super::{Places, PlacesError};

const CATEGORIES: [(&str, DangerLevel); 1] = [("app-caches.library-caches", DangerLevel::Low)];

/// A row of the module's category under `folder`, with the given `id` and `path`.
fn row(id: &str, folder: &str, path: &str) -> String {
    format!(
        "[[place]]\nid = \"{id}\"\ncategory = \"app-caches.library-caches\"\nfolder = \"{folder}\"\npath = \"{path}\"\n"
    )
}

fn refused(text: &str) -> PlacesError {
    Places::parse(text, &CATEGORIES).unwrap_err()
}

mod negative {
    use fresher_core::DangerLevel;

    use super::{Places, PlacesError, refused, row};

    #[test]
    fn a_table_that_is_not_toml_or_has_an_unknown_field_is_refused() {
        assert!(matches!(refused("[[place"), PlacesError::Format(_)));
        let text = format!("{}colour = 1\n", row("a", "app-caches", "*"));
        assert!(matches!(refused(&text), PlacesError::Format(_)));
    }

    #[test]
    fn a_table_with_no_row_is_refused() {
        assert_eq!(refused(""), PlacesError::Empty);
    }

    #[test]
    fn a_row_id_that_is_not_kebab_case_is_refused() {
        for id in ["Caches", "a.b", "", "a_b"] {
            assert_eq!(refused(&row(id, "app-caches", "*")), PlacesError::BadId { id: id.into() }, "{id}");
        }
    }

    #[test]
    fn a_row_id_listed_twice_is_refused() {
        let text = format!("{}{}", row("a", "app-caches", "*"), row("a", "app-support", "*"));
        assert_eq!(refused(&text), PlacesError::DuplicateId { id: "a".into() });
    }

    #[test]
    fn a_category_that_is_not_the_modules_is_refused() {
        let text = row("a", "app-caches", "*").replace("app-caches.library-caches", "app-caches.other");
        assert_eq!(
            refused(&text),
            PlacesError::UnknownCategory { id: "a".into(), category: "app-caches.other".into() }
        );
    }

    #[test]
    fn a_module_category_no_row_uses_is_refused() {
        let categories = [("app-caches.library-caches", DangerLevel::Low), ("app-caches.second", DangerLevel::Low)];

        let error = Places::parse(&row("a", "app-caches", "*"), &categories).unwrap_err();

        assert_eq!(error, PlacesError::CategoryWithoutRow { category: "app-caches.second".into() });
    }

    #[test]
    fn a_known_folder_the_scan_walks_as_its_own_is_refused() {
        for folder in ["user-temp", "shared-temp"] {
            let error = refused(&row("a", folder, "*"));
            assert!(
                matches!(&error, PlacesError::WalkedPlace { id, folder: f } if id == "a" && f.as_str() == folder),
                "{folder}: {error:?}"
            );
        }
    }

    #[test]
    fn a_path_that_is_not_plain_names_then_a_star_is_refused() {
        for path in ["", "Editor", "*/x", "a/*/*", "a//*", "../*", "./*", "a*", "*b", "/*", "a/**"] {
            assert_eq!(
                refused(&row("a", "app-caches", path)),
                PlacesError::NotPlainStar { id: "a".into(), path: path.into() },
                "{path}"
            );
        }
    }

    #[test]
    fn two_rows_of_one_place_are_refused() {
        let text = format!("{}{}", row("a", "app-caches", "*"), row("b", "app-caches", "*"));
        assert_eq!(refused(&text), PlacesError::DuplicatePlace { first: "a".into(), second: "b".into() });
    }
}

mod positive {
    use fresher_core::{Claim, KnownFolder, Pattern};

    use super::{CATEGORIES, row};
    use crate::places::Places;

    #[test]
    fn rows_of_different_places_are_read_in_order() {
        let text = format!("{}{}", row("a", "app-caches", "*"), row("b", "app-support", "Vendor/*"));

        let places = Places::parse(&text, &CATEGORIES).unwrap();

        let ids: Vec<&str> = places.rows().iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
    }

    #[test]
    fn the_shipped_table_has_one_row_using_the_modules_category() {
        let places = Places::shipped().unwrap();

        let rows = places.rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "library-caches");
        assert_eq!(rows[0].category, "app-caches.library-caches");
        assert_eq!((rows[0].folder, rows[0].path.as_str()), (KnownFolder::AppCaches, "*"));
    }

    #[test]
    fn the_shipped_interest_is_one_whole_claim_matching_only_bundle_ids() {
        let interests = Places::shipped().unwrap().interests().unwrap();

        assert_eq!(interests.len(), 1);
        assert_eq!(interests[0].pattern(), &Pattern::Under(KnownFolder::AppCaches, "*".into()));
        assert_eq!(interests[0].claim(), Claim::Whole);
        assert!(interests[0].matches_only_bundle_ids());
    }
}
