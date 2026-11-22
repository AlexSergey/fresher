//! The places table: every refusal by a table made for it, then the shipped table.

use fresher_core::DangerLevel;

use super::{Places, PlacesError};

const CATEGORIES: [(&str, DangerLevel); 2] =
    [("xcode.device-support", DangerLevel::Low), ("xcode.previews", DangerLevel::Safe)];

/// A device support row of the first category with the given `id`, `folder` and `path`.
fn row(id: &str, folder: &str, path: &str) -> String {
    format!(
        "[[place]]\nid = \"{id}\"\ncategory = \"xcode.device-support\"\nfolder = \"{folder}\"\npath = \"{path}\"\n\
         shape = \"device-support\"\nplatform = \"iOS\"\nsign = [\"Symbols\"]\nremoval = \"trash\"\n"
    )
}

/// A `fixed` row of the second category.
fn fixed(id: &str, path: &str) -> String {
    format!(
        "[[place]]\nid = \"{id}\"\ncategory = \"xcode.previews\"\nfolder = \"library\"\npath = \"{path}\"\n\
         shape = \"fixed\"\nsign = [\"Simulator Devices\"]\nremoval = \"empty-folder\"\n"
    )
}

fn good() -> String {
    format!("{}{}", row("a", "library", "D/*"), fixed("b", "P"))
}

fn refused(text: &str) -> PlacesError {
    Places::parse(text, &CATEGORIES).unwrap_err()
}

mod negative {
    use super::{CATEGORIES, Places, PlacesError, fixed, good, refused, row};

    #[test]
    fn a_table_that_is_not_toml_or_has_an_unknown_field_is_refused() {
        assert!(matches!(refused("[[place"), PlacesError::Format(_)));
        assert!(matches!(refused(&format!("{}colour = 1\n", good())), PlacesError::Format(_)));
        assert!(matches!(refused(&good().replace("\"trash\"", "\"burn\"")), PlacesError::Format(_)));
    }

    #[test]
    fn a_table_with_no_row_is_refused() {
        assert_eq!(refused(""), PlacesError::Empty);
    }

    #[test]
    fn a_row_id_that_is_not_kebab_case_is_refused() {
        for id in ["Caches", "a.b", "", "a_b"] {
            let text = format!("{}{}", row(id, "library", "D/*"), fixed("b", "P"));
            assert_eq!(refused(&text), PlacesError::BadId { id: id.into() }, "{id}");
        }
    }

    #[test]
    fn a_row_id_listed_twice_is_refused() {
        let text = format!("{}{}", good(), fixed("a", "Q"));
        assert_eq!(refused(&text), PlacesError::DuplicateId { id: "a".into() });
    }

    #[test]
    fn a_category_that_is_not_the_modules_is_refused() {
        let text = good().replace("xcode.previews", "xcode.other");
        assert_eq!(refused(&text), PlacesError::UnknownCategory { id: "b".into(), category: "xcode.other".into() });
    }

    #[test]
    fn a_module_category_no_row_uses_is_refused() {
        let error = Places::parse(&row("a", "library", "D/*"), &CATEGORIES).unwrap_err();

        assert_eq!(error, PlacesError::CategoryWithoutRow { category: "xcode.previews".into() });
    }

    #[test]
    fn a_known_folder_the_scan_walks_as_its_own_is_refused() {
        let text = format!("{}{}", row("a", "user-temp", "D/*"), fixed("b", "P"));

        let error = refused(&text);

        assert!(matches!(&error, PlacesError::WalkedPlace { id, .. } if id == "a"), "{error:?}");
    }

    #[test]
    fn a_path_that_is_not_plain_names_and_whole_stars_is_refused() {
        for path in ["", "a/*b", "*b", "a//*", "../*", "./*", "/*", "a/**", "a/b*/*"] {
            let text = format!("{}{}", row("a", "library", path), fixed("b", "P"));
            assert_eq!(refused(&text), PlacesError::BadPath { id: "a".into(), path: path.into() }, "{path}");
        }
    }

    #[test]
    fn a_fixed_row_with_a_star_or_a_device_support_row_without_a_last_star_is_refused() {
        let star = format!("{}{}", row("a", "library", "D/*"), fixed("b", "P/*"));
        assert_eq!(refused(&star), PlacesError::ShapeAndPath { id: "b".into(), path: "P/*".into() });
        let plain = format!("{}{}", row("a", "library", "D/Name"), fixed("b", "P"));
        assert_eq!(refused(&plain), PlacesError::ShapeAndPath { id: "a".into(), path: "D/Name".into() });
        let inner = format!("{}{}", row("a", "library", "D/*/Name"), fixed("b", "P"));
        assert_eq!(refused(&inner), PlacesError::ShapeAndPath { id: "a".into(), path: "D/*/Name".into() });
    }

    #[test]
    fn a_platform_where_it_does_not_belong_or_missing_where_it_does_is_refused() {
        let on_fixed = format!("{}{}platform = \"iOS\"\n", row("a", "library", "D/*"), fixed("b", "P"));
        assert_eq!(refused(&on_fixed), PlacesError::Platform { id: "b".into() });
        let missing = good().replace("platform = \"iOS\"\n", "");
        assert_eq!(refused(&missing), PlacesError::Platform { id: "a".into() });
    }

    #[test]
    fn a_row_with_no_sign_is_refused() {
        let text = good().replace("[\"Symbols\"]", "[]");
        assert_eq!(refused(&text), PlacesError::NoSign { id: "a".into() });
    }

    #[test]
    fn two_rows_of_one_place_are_refused() {
        let text = format!("{}{}", good(), row("c", "library", "D/*"));
        assert_eq!(refused(&text), PlacesError::DuplicatePlace { first: "a".into(), second: "c".into() });
    }

    #[test]
    fn a_sign_that_is_not_a_single_name_is_refused_when_the_interests_are_built() {
        let text = good().replace("[\"Symbols\"]", "[\"a/b\"]");

        let places = Places::parse(&text, &CATEGORIES).unwrap();

        assert!(places.interests().is_err());
    }
}

mod positive {
    use fresher_core::{Claim, Interest, KnownFolder, Pattern};

    use super::{CATEGORIES, good};
    use crate::categories::CATEGORIES as SHIPPED_CATEGORIES;
    use crate::places::{Places, RowRemoval, Shape};

    #[test]
    fn rows_are_read_in_order_with_their_fields() {
        let places = Places::parse(&good(), &CATEGORIES).unwrap();

        let rows = places.rows();
        assert_eq!(rows.iter().map(|row| row.id.as_str()).collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(rows[0].shape, Shape::DeviceSupport);
        assert_eq!(rows[0].platform.as_deref(), Some("iOS"));
        assert_eq!(rows[0].removal, RowRemoval::Trash);
        assert_eq!(rows[1].shape, Shape::Fixed);
        assert_eq!(rows[1].platform, None);
        assert_eq!(rows[1].removal, RowRemoval::EmptyFolder);
    }

    #[test]
    fn the_shipped_table_has_seven_rows_in_the_designs_order() {
        let places = Places::shipped().unwrap();

        let ids: Vec<&str> = places.rows().iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "ios-device-support",
                "watchos-device-support",
                "tvos-device-support",
                "visionos-device-support",
                "previews",
                "simulator-caches",
                "archives"
            ]
        );
    }

    #[test]
    fn the_shipped_table_uses_the_four_categories() {
        let places = Places::shipped().unwrap();

        for (category, _) in SHIPPED_CATEGORIES {
            assert!(places.rows().iter().any(|row| row.category == category), "{category}");
        }
        let platforms: Vec<Option<&str>> = places.rows().iter().map(|row| row.platform.as_deref()).collect();
        assert_eq!(platforms, [Some("iOS"), Some("watchOS"), Some("tvOS"), Some("visionOS"), None, None, None]);
    }

    #[test]
    fn the_shipped_interests_are_whole_claims_under_library_holding_their_sign() {
        let places = Places::shipped().unwrap();

        let interests = places.interests().unwrap();

        assert_eq!(interests.len(), 7);
        for (interest, row) in interests.iter().zip(places.rows()) {
            assert_eq!(interest.pattern(), &Pattern::Under(KnownFolder::Library, row.path.clone()));
            assert_eq!(interest.claim(), Claim::Whole);
            assert_eq!(interest.holds(), row.sign.as_slice());
        }
    }

    #[test]
    fn only_the_archives_interest_is_shaped_to_xcarchive() {
        let interests = Places::shipped().unwrap().interests().unwrap();

        let suffixes: Vec<&[String]> = interests.iter().map(Interest::name_suffixes).collect();
        assert!(suffixes[..6].iter().all(|suffixes| suffixes.is_empty()));
        assert_eq!(suffixes[6], [".xcarchive".to_owned()]);
    }
}
