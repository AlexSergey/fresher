//! The places table: every refusal by a table made for it, then the shipped table.

use fresher_core::DangerLevel;

use super::Places;

const CATEGORIES: [(&str, DangerLevel); 2] =
    [("app-leftovers.caches", DangerLevel::Safe), ("app-leftovers.data", DangerLevel::High)];

/// A row of the first category, with `extra` lines after its fixed ones.
fn row(id: &str, extra: &str) -> String {
    format!(
        "[[place]]\nid = \"{id}\"\ncategory = \"app-leftovers.caches\"\nfolder = \"app-caches\"\npath = \"*\"\nkind = \"folder\"\n{extra}\n"
    )
}

/// A valid one-row-per-category table to put rows into: a proof in each.
fn valid() -> String {
    let data = "[[place]]\nid = \"data\"\ncategory = \"app-leftovers.data\"\nfolder = \"app-support\"\npath = \"*\"\nkind = \"folder\"\nproof = true\n";
    format!("{}{data}", row("caches", ""))
}

fn refused(text: &str) -> super::PlacesError {
    Places::parse(text, &CATEGORIES).unwrap_err()
}

mod negative {
    use super::{refused, row, valid};
    use crate::places::{Places, PlacesError};

    #[test]
    fn a_table_that_is_not_toml_or_has_an_unknown_field_is_refused() {
        assert!(matches!(refused("[[place"), PlacesError::Format(_)));
        assert!(matches!(refused(&format!("{}{}", valid(), row("x", "colour = 1"))), PlacesError::Format(_)));
    }

    #[test]
    fn a_table_with_no_row_is_refused() {
        assert_eq!(refused(""), PlacesError::Empty);
    }

    #[test]
    fn a_row_id_that_is_not_kebab_case_is_refused() {
        for id in ["Caches", "a.b", "", "a_b"] {
            assert_eq!(refused(&row(id, "")), PlacesError::BadId { id: id.into() }, "{id}");
        }
    }

    #[test]
    fn a_row_id_listed_twice_is_refused() {
        let text = format!("{}{}", row("caches", ""), row("caches", "suffix = \".x\""));
        assert_eq!(refused(&text), PlacesError::DuplicateId { id: "caches".into() });
    }

    #[test]
    fn a_category_that_is_not_the_modules_is_refused() {
        let text = row("a", "").replace("app-leftovers.caches", "app-leftovers.other");
        assert_eq!(
            refused(&text),
            PlacesError::UnknownCategory { id: "a".into(), category: "app-leftovers.other".into() }
        );
    }

    #[test]
    fn a_category_without_a_row_is_refused() {
        assert_eq!(
            refused(&row("a", "proof = true")),
            PlacesError::CategoryWithoutRow { category: "app-leftovers.data".into() }
        );
    }

    #[test]
    fn a_folder_that_is_not_a_library_place_is_refused() {
        for folder in ["user-temp", "shared-temp", "home", "trash", "downloads", "applications"] {
            let text = row("a", "").replace("app-caches", folder);
            let error = refused(&text);
            assert!(
                matches!(&error, PlacesError::WalkedPlace { id, folder: f } if id == "a" && f.as_str() == folder),
                "{folder}: {error:?}"
            );
        }
    }

    #[test]
    fn the_group_containers_are_refused() {
        let text = row("a", "").replace("app-caches", "group-containers");
        assert_eq!(refused(&text), PlacesError::GroupContainers { id: "a".into() });
    }

    #[test]
    fn a_library_path_into_the_group_containers_the_launch_agents_or_a_known_folder_is_refused() {
        let under_library = |path: &str| {
            row("a", "").replace("app-caches", "library").replace("path = \"*\"", &format!("path = \"{path}\""))
        };
        assert_eq!(refused(&under_library("group containers/*")), PlacesError::GroupContainers { id: "a".into() });
        assert_eq!(
            refused(&under_library("LaunchAgents/*")),
            PlacesError::WalkedPlace { id: "a".into(), folder: fresher_core::KnownFolder::Library }
        );
        for path in [
            "Caches/*",
            "application support/x/*",
            "Preferences/*",
            "LOGS/*",
            "Containers/*",
            "Saved Application State/*",
        ] {
            assert_eq!(
                refused(&under_library(path)),
                PlacesError::OwnKnownFolder { id: "a".into(), path: path.into() },
                "{path}"
            );
        }
    }

    #[test]
    fn a_path_that_is_not_plain_names_then_a_star_is_refused() {
        for path in ["", "Notes", "*/a", "a/*/b", "a/", "/*", "../*", "a//*", "./*", "a*"] {
            let text = row("a", "").replace("path = \"*\"", &format!("path = \"{path}\""));
            assert_eq!(refused(&text), PlacesError::NotAStarPath { id: "a".into(), path: path.into() }, "{path}");
        }
    }

    #[test]
    fn two_rows_of_one_place_with_the_same_kind_and_suffix_are_refused() {
        let text = format!("{}{}", row("a", ""), row("b", ""));
        assert_eq!(refused(&text), PlacesError::SameForm { first: "a".into(), second: "b".into() });
    }

    #[test]
    fn a_suffix_that_is_not_a_dot_and_a_plain_end_is_refused() {
        for suffix in ["plist", ".", "", ".a/b", ".a b", ".a..b", ".a.", "..a"] {
            let text = row("a", &format!("suffix = \"{suffix}\""));
            assert_eq!(refused(&text), PlacesError::BadSuffix { id: "a".into(), suffix: suffix.into() }, "{suffix}");
        }
    }

    #[test]
    fn rows_of_one_place_that_disagree_on_optional_are_refused() {
        let text = format!("{}{}", row("a", ""), row("b", "suffix = \".x\"\noptional = true"));
        assert_eq!(refused(&text), PlacesError::OptionalDiffers { first: "a".into(), second: "b".into() });
    }

    #[test]
    fn an_index_of_no_interest_has_no_rows() {
        let places = Places::shipped().unwrap();

        assert_eq!(places.rows_of(9), []);
        assert_eq!(places.row(10), None);
    }

    #[test]
    fn a_table_with_no_proof_is_refused() {
        let data = "[[place]]\nid = \"data\"\ncategory = \"app-leftovers.data\"\nfolder = \"app-support\"\npath = \"*\"\nkind = \"folder\"\n";
        assert_eq!(refused(&format!("{}{data}", row("caches", ""))), PlacesError::NoProof);
    }
}

mod positive {
    use fresher_core::{Claim, KnownFolder, Pattern};

    use crate::categories::CATEGORIES;
    use crate::places::{Form, Places};

    fn shipped() -> Places {
        Places::shipped().unwrap()
    }

    #[test]
    fn a_valid_table_made_here_is_read() {
        assert!(Places::parse(&super::valid(), &super::CATEGORIES).is_ok());
    }

    #[test]
    fn a_library_path_with_a_suffix_of_several_names_is_read() {
        let text = super::valid().replacen(
            "folder = \"app-caches\"\npath = \"*\"",
            "folder = \"library\"\npath = \"WebKit/*\"",
            1,
        );
        let text = text.replacen("kind = \"folder\"\n", "kind = \"folder\"\nsuffix = \".a.b-c_d\"\n", 1);

        assert!(Places::parse(&text, &super::CATEGORIES).is_ok());
    }

    #[test]
    fn the_rows_of_an_interest_are_those_of_its_place_with_their_indexes() {
        let places = shipped();

        let of = |interest: usize| -> Vec<(usize, &str)> {
            places.rows_of(interest).into_iter().map(|(index, row)| (index, row.id.as_str())).collect()
        };
        assert_eq!(of(0), [(0, "caches")]);
        assert_eq!(of(4), [(4, "http-storage"), (5, "http-cookies")]);
        assert_eq!(of(8), [(9, "container")]);
        assert_eq!(places.row(2).map(|row| row.id.as_str()), Some("saved-state"));
        assert_eq!(places.rows().len(), 10);
    }

    #[test]
    fn the_shipped_table_has_ten_rows_in_the_designs_order() {
        let places = shipped();
        let ids: Vec<&str> = places.rows.iter().map(|row| row.id.as_str()).collect();

        assert_eq!(
            ids,
            [
                "caches",
                "logs",
                "saved-state",
                "preferences",
                "http-storage",
                "http-cookies",
                "web-data",
                "cookies",
                "support",
                "container"
            ]
        );
    }

    #[test]
    fn every_category_has_a_row() {
        let places = shipped();

        for (category, _) in CATEGORIES {
            assert!(places.rows.iter().any(|row| row.category == category), "{category}");
        }
    }

    #[test]
    fn saved_state_and_container_are_the_proofs_and_container_alone_is_optional() {
        let places = shipped();

        let proofs: Vec<&str> = places.rows.iter().filter(|row| row.proof).map(|row| row.id.as_str()).collect();
        let optional: Vec<&str> = places.rows.iter().filter(|row| row.optional).map(|row| row.id.as_str()).collect();
        assert_eq!(proofs, ["saved-state", "container"]);
        assert_eq!(optional, ["container"]);
    }

    #[test]
    fn preferences_are_a_plist_file_and_saved_state_a_folder_with_its_suffix() {
        let places = shipped();

        let of = |id: &str| places.rows.iter().find(|row| row.id == id).unwrap();
        assert_eq!((of("preferences").kind, of("preferences").suffix.as_deref()), (Form::File, Some(".plist")));
        assert_eq!((of("saved-state").kind, of("saved-state").suffix.as_deref()), (Form::Folder, Some(".savedState")));
        assert_eq!((of("caches").kind, of("caches").suffix.as_deref()), (Form::Folder, None));
    }

    #[test]
    fn nine_interests_claim_whole_names_shaped_as_ids_and_the_http_rows_share_one() {
        let interests = shipped().interests().unwrap();

        let places: Vec<(KnownFolder, &str)> = interests
            .iter()
            .filter_map(|interest| match interest.pattern() {
                Pattern::Under(folder, path) => Some((*folder, path.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(
            places,
            [
                (KnownFolder::AppCaches, "*"),
                (KnownFolder::Logs, "*"),
                (KnownFolder::SavedState, "*"),
                (KnownFolder::Preferences, "*"),
                (KnownFolder::Library, "HTTPStorages/*"),
                (KnownFolder::Library, "WebKit/*"),
                (KnownFolder::Library, "Cookies/*"),
                (KnownFolder::AppSupport, "*"),
                (KnownFolder::Containers, "*"),
            ]
        );
        assert!(
            interests.iter().all(|interest| interest.claim() == Claim::Whole && interest.matches_only_bundle_ids())
        );
    }
}
