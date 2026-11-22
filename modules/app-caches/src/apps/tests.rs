//! The apps table: every refusal by a table made for it, then the shipped table.

use fresher_core::DangerLevel;

use super::{Apps, AppsError};

const CATEGORIES: [(&str, DangerLevel); 1] = [("app-caches.browsers", DangerLevel::Low)];

/// A row of the browsers category with the given `id`, `folder`, `path` and `caches` (a TOML array's inside).
fn row_with(id: &str, folder: &str, path: &str, caches: &str) -> String {
    format!(
        "[[app]]\nid = \"{id}\"\ncategory = \"app-caches.browsers\"\nbundle-id = \"com.example.Browser\"\n\
         folder = \"{folder}\"\npath = \"{path}\"\ncaches = [{caches}]\n"
    )
}

/// A row of the browsers category under `folder`, with the given `id` and `path`, and one cache, `Cache`.
fn row(id: &str, folder: &str, path: &str) -> String {
    row_with(id, folder, path, "\"Cache\"")
}

fn refused(text: &str) -> AppsError {
    Apps::parse(text, &CATEGORIES).unwrap_err()
}

mod negative {
    use fresher_core::DangerLevel;

    use super::{Apps, AppsError, refused, row, row_with};

    #[test]
    fn a_table_that_is_not_toml_or_has_an_unknown_field_is_refused() {
        assert!(matches!(refused("[[app"), AppsError::Format(_)));
        let unknown = format!("{}colour = 1\n", row("a", "app-caches", "Browser/*"));
        assert!(matches!(refused(&unknown), AppsError::Format(_)));
        let missing = row("a", "app-caches", "Browser/*").replace("caches = [\"Cache\"]\n", "");
        assert!(matches!(refused(&missing), AppsError::Format(_)));
    }

    #[test]
    fn a_table_with_no_row_is_refused() {
        assert_eq!(refused(""), AppsError::Empty);
    }

    #[test]
    fn a_row_id_that_is_not_kebab_case_is_refused() {
        for id in ["Chrome", "a.b", "", "a_b"] {
            assert_eq!(refused(&row(id, "app-caches", "Browser/*")), AppsError::BadId { id: id.into() }, "{id}");
        }
    }

    #[test]
    fn a_row_id_twice_is_refused() {
        let text = format!("{}{}", row("a", "app-caches", "Browser/*"), row("a", "app-support", "Browser/*"));
        assert_eq!(refused(&text), AppsError::DuplicateId { id: "a".into() });
    }

    #[test]
    fn a_bundle_id_that_is_not_one_or_has_no_dot_is_refused() {
        for bundle_id in ["Browser", "", "com..example", ".com.example", "com.example.", "com.example/App"] {
            let text = row("a", "app-caches", "Browser/*").replace("com.example.Browser", bundle_id);
            assert_eq!(
                refused(&text),
                AppsError::BadBundleId { id: "a".into(), bundle_id: bundle_id.into() },
                "{bundle_id}"
            );
        }
    }

    #[test]
    fn a_category_not_of_the_table_is_refused() {
        for category in ["app-caches.other", "app-caches.library-caches"] {
            let text = row("a", "app-caches", "Browser/*").replace("app-caches.browsers", category);
            assert_eq!(
                refused(&text),
                AppsError::UnknownCategory { id: "a".into(), category: category.into() },
                "{category}"
            );
        }
    }

    #[test]
    fn a_category_without_a_row_is_refused() {
        let categories = [("app-caches.browsers", DangerLevel::Low), ("app-caches.web-engine", DangerLevel::Low)];

        let error = Apps::parse(&row("a", "app-caches", "Browser/*"), &categories).unwrap_err();

        assert_eq!(error, AppsError::CategoryWithoutRow { category: "app-caches.web-engine".into() });
    }

    #[test]
    fn a_walked_place_is_refused() {
        for folder in ["user-temp", "shared-temp"] {
            let error = refused(&row("a", folder, "Browser/*"));
            assert!(
                matches!(&error, AppsError::WalkedPlace { id, folder: f } if id == "a" && f.as_str() == folder),
                "{folder}: {error:?}"
            );
        }
    }

    #[test]
    fn a_path_with_a_bad_name_or_two_stars_is_refused() {
        let bad = ["", "Browser//x", "Browser/", "../x", "./x", "Browser/..", "Browser/a*", "Browser/*b", "/x"];
        for path in bad.into_iter().chain(["Browser/*/*", "Browser/**", "*/*"]) {
            assert_eq!(
                refused(&row("a", "app-caches", path)),
                AppsError::BadPath { id: "a".into(), path: path.into() },
                "{path}"
            );
        }
        let error = refused(&row("a", "app-caches", "Browser/a\\u0000b"));
        assert_eq!(error, AppsError::BadPath { id: "a".into(), path: "Browser/a\0b".into() });
    }

    #[test]
    fn a_path_starting_with_a_star_is_refused() {
        for path in ["*", "*/Profiles"] {
            assert_eq!(
                refused(&row("a", "app-caches", path)),
                AppsError::StarFirst { id: "a".into(), path: path.into() },
                "{path}"
            );
        }
    }

    #[test]
    fn a_path_starting_with_an_id_shaped_name_is_refused() {
        for path in ["com.example.Browser", "com.example.Browser/*"] {
            assert_eq!(
                refused(&row("a", "app-caches", path)),
                AppsError::IdShapedName { id: "a".into(), path: path.into() },
                "{path}"
            );
        }
    }

    #[test]
    fn a_row_without_caches_is_refused() {
        assert_eq!(refused(&row_with("a", "app-caches", "Browser/*", "")), AppsError::NoCaches { id: "a".into() });
    }

    #[test]
    fn a_bad_or_repeated_cache_name_is_refused() {
        for (caches, cache) in [
            ("\"\"", ""),
            ("\".\"", "."),
            ("\"..\"", ".."),
            ("\"a/b\"", "a/b"),
            ("\"*\"", "*"),
            ("\"Cache*\"", "Cache*"),
            ("\"a\\u0000b\"", "a\0b"),
            ("\"Cache\", \"Code Cache\", \"Cache\"", "Cache"),
        ] {
            assert_eq!(
                refused(&row_with("a", "app-caches", "Browser/*", caches)),
                AppsError::BadCache { id: "a".into(), cache: cache.into() },
                "{caches}"
            );
        }
    }

    #[test]
    fn overlapping_places_are_refused() {
        let text =
            format!("{}{}", row("a", "app-caches", "Google/Chrome/*"), row("b", "app-caches", "Google/*/Default"));
        assert_eq!(refused(&text), AppsError::OverlappingPlaces { first: "a".into(), second: "b".into() });
        let same = format!("{}{}", row("a", "app-caches", "Browser"), row("b", "app-caches", "Browser"));
        assert_eq!(refused(&same), AppsError::OverlappingPlaces { first: "a".into(), second: "b".into() });
    }
}

mod positive {
    use fresher_core::{Claim, KnownFolder, Pattern};

    use super::{CATEGORIES, row};
    use crate::apps::Apps;
    use crate::exclude::Exclude;

    #[test]
    fn the_same_path_under_two_folders_and_paths_of_different_lengths_do_not_overlap() {
        let text = format!(
            "{}{}{}",
            row("a", "app-caches", "Google/Chrome/*"),
            row("b", "app-support", "Google/Chrome/*"),
            row("c", "app-caches", "Google/*"),
        );

        let apps = Apps::parse(&text, &CATEGORIES).unwrap();

        let ids: Vec<&str> = apps.rows().iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, ["a", "b", "c"]);
    }

    #[test]
    fn the_shipped_table_has_its_four_rows() {
        let apps = Apps::shipped().unwrap();

        let rows: Vec<_> = apps
            .rows()
            .iter()
            .map(|row| (row.id.as_str(), row.category, row.bundle_id.as_str(), row.folder, row.profile))
            .collect();
        assert_eq!(
            rows,
            [
                ("chrome", "app-caches.browsers", "com.google.Chrome", KnownFolder::AppCaches, Some(2)),
                ("chromium", "app-caches.browsers", "org.chromium.Chromium", KnownFolder::AppCaches, Some(1)),
                ("firefox", "app-caches.browsers", "org.mozilla.firefox", KnownFolder::AppCaches, Some(2)),
                (
                    "redis-insight",
                    "app-caches.web-engine",
                    "org.RedisLabs.RedisInsight-V2",
                    KnownFolder::AppSupport,
                    None
                ),
            ]
        );
        assert_eq!(apps.rows()[2].caches, ["cache2", "startupCache"]);
    }

    #[test]
    fn the_shipped_table_gives_one_whole_interest_per_cache() {
        let interests = Apps::shipped().unwrap().interests().unwrap();

        assert_eq!(interests.len(), 11);
        assert_eq!(interests[0].0.pattern(), &Pattern::Under(KnownFolder::AppCaches, "Google/Chrome/*/Cache".into()));
        assert_eq!(
            interests[10].0.pattern(),
            &Pattern::Under(KnownFolder::AppSupport, "RedisInsight/DawnWebGPUCache".into())
        );
        let indexes: Vec<(usize, usize)> = interests.iter().map(|(_, row, cache)| (*row, *cache)).collect();
        assert_eq!(indexes, [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1), (3, 0), (3, 1), (3, 2), (3, 3), (3, 4)]);
        for (interest, ..) in &interests {
            assert_eq!(interest.claim(), Claim::Whole);
            assert!(!interest.matches_only_bundle_ids());
        }
    }

    #[test]
    fn no_shipped_row_is_kept_by_the_exclusions() {
        let exclude = Exclude::shipped().unwrap();

        for row in Apps::shipped().unwrap().rows() {
            assert!(!exclude.keeps(&row.bundle_id), "{}", row.id);
        }
    }
}
