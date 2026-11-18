//! The table read from text: every refusal by a table made for it, then the shipped table and its interests.

use std::fmt::Write as _;

use fresher_core::DangerLevel;

/// The categories the tables made here are checked against.
const CATEGORIES: [(&str, DangerLevel); 2] =
    [("dev-caches.npm", DangerLevel::Low), ("dev-caches.cargo", DangerLevel::Low)];

/// A row of `dev-caches.npm`, with `fields` (raw TOML values) in place of its own or added to them.
fn npm(fields: &[(&str, &str)]) -> String {
    row(
        &[
            ("id", "\"npm\""),
            ("category", "\"dev-caches.npm\""),
            ("tool", "\"npm\""),
            ("folder", "\"home\""),
            ("path", "\".npm/_cacache\""),
            ("sign", "[\"index-v5\", \"content-v2\"]"),
            ("comes-back", "\"download\""),
        ],
        fields,
    )
}

/// A row of `dev-caches.cargo` whose sign is beside its folder, with `fields` in place of its own or added to them.
fn cargo(fields: &[(&str, &str)]) -> String {
    row(
        &[
            ("id", "\"cargo-src\""),
            ("category", "\"dev-caches.cargo\""),
            ("tool", "\"Cargo\""),
            ("folder", "\"home\""),
            ("path", "\".cargo/registry/src\""),
            ("sign-at", "\".cargo/registry\""),
            ("sign", "[\"CACHEDIR.TAG\"]"),
            ("comes-back", "\"download\""),
        ],
        fields,
    )
}

fn row(defaults: &[(&str, &str)], fields: &[(&str, &str)]) -> String {
    let mut text = String::from("[[cache]]\n");
    for (key, value) in defaults {
        let value = fields.iter().find(|(field, _)| field == key).map_or(*value, |(_, value)| *value);
        writeln!(text, "{key} = {value}").unwrap();
    }
    for (key, value) in fields.iter().filter(|(key, _)| !defaults.iter().any(|(default, _)| default == key)) {
        writeln!(text, "{key} = {value}").unwrap();
    }
    text
}

/// A list of `count` distinct sign names, as a TOML array.
fn signs(count: usize) -> String {
    let names: Vec<String> = (0..count).map(|index| format!("\"name-{index}\"")).collect();
    format!("[{}]", names.join(", "))
}

mod negative {
    use fresher_core::{AppIdError, KnownFolder};

    use super::{CATEGORIES, cargo, npm, signs};
    use crate::tools::{Tools, ToolsError};

    fn parse(text: &str) -> Result<Tools, ToolsError> {
        Tools::parse(text, &CATEGORIES)
    }

    #[test]
    fn refuses_text_that_is_not_a_table() {
        for text in [
            "[[cache]]\nid = 1\n".to_owned(),
            npm(&[("size", "3")]) + &cargo(&[]),
            npm(&[("folder", "\"nowhere\"")]) + &cargo(&[]),
            npm(&[("comes-back", "\"maybe\"")]) + &cargo(&[]),
            npm(&[("name", "\"other-shape\"")]) + &cargo(&[]),
            npm(&[("sign", "\"index-v5\"")]) + &cargo(&[]),
        ] {
            assert!(matches!(parse(&text), Err(ToolsError::Format(_))), "{text}");
        }
    }

    #[test]
    fn refuses_a_table_with_no_row() {
        assert_eq!(parse(""), Err(ToolsError::Empty));
    }

    #[test]
    fn refuses_a_row_id_that_is_not_kebab_case() {
        for id in ["", "Npm", "npm.cache", "npm_cache", "-npm", "npm-", "npm--cache", "1npm", "npm cache"] {
            let text = npm(&[("id", &format!("\"{id}\""))]) + &cargo(&[]);

            assert_eq!(parse(&text), Err(ToolsError::BadId { id: id.into() }), "{id}");
        }
    }

    #[test]
    fn refuses_a_row_id_listed_twice() {
        let text = npm(&[]) + &cargo(&[]) + &cargo(&[("id", "\"npm\""), ("path", "\".cargo/registry/cache\"")]);

        assert_eq!(parse(&text), Err(ToolsError::DuplicateId { id: "npm".into() }));
    }

    #[test]
    fn refuses_a_category_that_is_not_the_modules() {
        for category in ["dev-caches.bower", "temp.old-files", "npm", ""] {
            let text = npm(&[("category", &format!("\"{category}\""))]) + &cargo(&[]);

            assert_eq!(
                parse(&text),
                Err(ToolsError::UnknownCategory { id: "npm".into(), category: category.into() }),
                "{category}"
            );
        }
    }

    #[test]
    fn refuses_a_category_without_a_row() {
        assert_eq!(parse(&npm(&[])), Err(ToolsError::CategoryWithoutRow { category: "dev-caches.cargo".into() }));
    }

    #[test]
    fn refuses_a_place_the_scan_walks_as_its_own() {
        for (name, folder) in [("user-temp", KnownFolder::UserTemp), ("shared-temp", KnownFolder::SharedTemp)] {
            let text = npm(&[("folder", &format!("\"{name}\""))]) + &cargo(&[]);

            assert_eq!(parse(&text), Err(ToolsError::WalkedPlace { id: "npm".into(), folder }), "{name}");
        }
    }

    #[test]
    fn refuses_a_path_that_is_not_relative_plain_names() {
        for path in ["", "/.npm/_cacache", ".npm/../_cacache", "./.npm", ".npm//_cacache", ".npm/", "..", ".npm/./x"] {
            let text = npm(&[("path", &format!("\"{path}\""))]) + &cargo(&[]);

            assert_eq!(parse(&text), Err(ToolsError::NotRelative { id: "npm".into(), path: path.into() }), "{path}");
        }
    }

    #[test]
    fn refuses_a_sign_place_that_is_not_relative_plain_names() {
        for sign_at in ["", "/.cargo", ".cargo/../registry", ".cargo//registry", ".cargo/registry/"] {
            let text = npm(&[]) + &cargo(&[("sign-at", &format!("\"{sign_at}\""))]);

            assert_eq!(
                parse(&text),
                Err(ToolsError::NotRelative { id: "cargo-src".into(), path: sign_at.into() }),
                "{sign_at}"
            );
        }
    }

    #[test]
    fn refuses_a_sign_place_that_is_not_strictly_above_the_folder() {
        for sign_at in [".cargo/registry/src", ".cargo/other", ".cargo/registry/src/deeper", ".cargo/reg", ".carg"] {
            let text = npm(&[]) + &cargo(&[("sign-at", &format!("\"{sign_at}\""))]);

            assert_eq!(
                parse(&text),
                Err(ToolsError::SignNotAbove {
                    id: "cargo-src".into(),
                    sign_at: sign_at.into(),
                    path: ".cargo/registry/src".into()
                }),
                "{sign_at}"
            );
        }
    }

    #[test]
    fn refuses_two_rows_at_the_same_place() {
        let text = npm(&[]) + &cargo(&[]) + &cargo(&[("id", "\"cargo-again\"")]);

        assert_eq!(
            parse(&text),
            Err(ToolsError::SamePlace { first: "cargo-src".into(), second: "cargo-again".into() })
        );
    }

    #[test]
    fn refuses_two_rows_with_one_sign_place_and_different_signs() {
        let other = cargo(&[("id", "\"cargo-cache\""), ("path", "\".cargo/registry/cache\""), ("sign", "[\"index\"]")]);
        let text = npm(&[]) + &cargo(&[]) + &other;

        assert_eq!(
            parse(&text),
            Err(ToolsError::SignsDiffer { first: "cargo-src".into(), second: "cargo-cache".into() })
        );
    }

    #[test]
    fn refuses_an_empty_sign() {
        let text = npm(&[("sign", "[]")]) + &cargo(&[]);

        assert_eq!(parse(&text), Err(ToolsError::EmptySign { id: "npm".into() }));
    }

    #[test]
    fn refuses_a_sign_that_is_not_a_single_name() {
        for name in ["", ".", "..", "index-v5/x", "/index-v5"] {
            let text = npm(&[("sign", &format!("[\"{name}\"]"))]) + &cargo(&[]);

            assert_eq!(parse(&text), Err(ToolsError::NotAName { id: "npm".into(), name: name.into() }), "{name}");
        }
    }

    #[test]
    fn refuses_a_sign_name_listed_twice() {
        let text = npm(&[("sign", "[\"index-v5\", \"content-v2\", \"index-v5\"]")]) + &cargo(&[]);

        assert_eq!(parse(&text), Err(ToolsError::DuplicateSign { id: "npm".into(), name: "index-v5".into() }));
    }

    #[test]
    fn refuses_more_than_64_sign_names() {
        let text = npm(&[("sign", &signs(65))]) + &cargo(&[]);

        assert_eq!(parse(&text), Err(ToolsError::TooManySigns { id: "npm".into(), count: 65 }));
    }

    #[test]
    fn refuses_an_app_that_is_not_a_bundle_id() {
        let text = npm(&[("app", "\"com apple\"")]) + &cargo(&[]);

        assert_eq!(parse(&text), Err(ToolsError::BundleId(AppIdError::NotABundleId { id: "com apple".into() })));
    }

    #[test]
    fn refuses_a_name_shape_without_a_star_at_the_end_of_the_path() {
        for path in [".npm/_cacache", ".npm/*/x", ".npm/_cacache*"] {
            let text = npm(&[("path", &format!("\"{path}\"")), ("name", "\"project-hash\"")]) + &cargo(&[]);

            assert_eq!(parse(&text), Err(ToolsError::ShapeWithoutStar { id: "npm".into() }), "{path}");
        }
    }
}

mod positive {
    use std::collections::BTreeSet;

    use fresher_core::{BundleId, Claim, KnownFolder, Pattern};

    use super::{CATEGORIES, cargo, npm, signs};
    use crate::categories;
    use crate::tools::{ComesBack, NameShape, Role, Tools};

    #[test]
    fn reads_a_row_with_every_field() {
        let text = npm(&[
            ("path", "\"Developer/*\""),
            ("app", "\"com.example.App\""),
            ("name", "\"project-hash\""),
            ("comes-back", "\"build\""),
        ]) + &cargo(&[]);

        let tools = Tools::parse(&text, &CATEGORIES).unwrap();

        let row = &tools.caches()[0];
        assert_eq!(row.id, "npm");
        assert_eq!(row.category, "dev-caches.npm");
        assert_eq!(row.tool, "npm");
        assert_eq!(row.folder, KnownFolder::Home);
        assert_eq!(row.path, "Developer/*");
        assert_eq!(row.sign_at, None);
        assert_eq!(row.sign, ["index-v5", "content-v2"]);
        assert_eq!(row.comes_back, ComesBack::Build);
        assert_eq!(row.app, Some(BundleId::new("com.example.App").unwrap()));
        assert_eq!(row.name, Some(NameShape::ProjectHash));
        assert_eq!(tools.caches()[1].sign_at.as_deref(), Some(".cargo/registry"));
    }

    #[test]
    fn accepts_64_sign_names() {
        let text = npm(&[("sign", &signs(64))]) + &cargo(&[]);

        assert_eq!(Tools::parse(&text, &CATEGORIES).unwrap().caches()[0].sign.len(), 64);
    }

    #[test]
    fn rows_with_one_sign_place_may_list_its_names_in_another_order() {
        let first = npm(&[("path", "\"cache/a\""), ("sign-at", "\"cache\""), ("sign", "[\"x\", \"y\"]")]);
        let second =
            npm(&[("id", "\"npm-b\""), ("path", "\"cache/b\""), ("sign-at", "\"cache\""), ("sign", "[\"y\", \"x\"]")]);

        let tools = Tools::parse(&(first + &second + &cargo(&[])), &CATEGORIES).unwrap();

        assert_eq!(tools.caches().len(), 3);
    }

    #[test]
    fn the_shipped_table_has_its_ten_rows_in_order_and_uses_every_category() {
        let tools = Tools::shipped().unwrap();

        let ids: Vec<&str> = tools.caches().iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            [
                "npm",
                "yarn",
                "homebrew",
                "cargo-cache",
                "cargo-src",
                "cocoapods",
                "gradle",
                "pip",
                "go-build",
                "xcode-derived-data"
            ]
        );
        let used: BTreeSet<&str> = tools.caches().iter().map(|row| row.category).collect();
        let all: BTreeSet<&str> = categories::CATEGORIES.iter().map(|(id, _)| *id).collect();
        assert_eq!(used, all);
    }

    #[test]
    fn the_shipped_rows_carry_the_designs_places_and_signs() {
        let tools = Tools::shipped().unwrap();

        let rows: Vec<_> = tools
            .caches()
            .iter()
            .map(|row| {
                let sign: Vec<&str> = row.sign.iter().map(String::as_str).collect();
                (row.tool.as_str(), row.folder, row.path.as_str(), row.sign_at.as_deref(), sign, row.comes_back)
            })
            .collect();
        let (home, caches, library) = (KnownFolder::Home, KnownFolder::AppCaches, KnownFolder::Library);
        let (download, build) = (ComesBack::Download, ComesBack::Build);
        assert_eq!(
            rows,
            [
                ("npm", home, ".npm/_cacache", None, vec!["index-v5", "content-v2"], download),
                ("Yarn", caches, "Yarn", None, vec!["v6"], download),
                ("Homebrew", caches, "Homebrew/downloads", Some("Homebrew"), vec!["api"], download),
                ("Cargo", home, ".cargo/registry/cache", Some(".cargo/registry"), vec!["CACHEDIR.TAG"], download),
                ("Cargo", home, ".cargo/registry/src", Some(".cargo/registry"), vec!["CACHEDIR.TAG"], download),
                ("CocoaPods", caches, "CocoaPods", None, vec!["Pods"], download),
                ("Gradle", home, ".gradle/caches", None, vec!["CACHEDIR.TAG"], download),
                ("pip", caches, "pip", None, vec!["http-v2"], download),
                ("Go", caches, "go-build", None, vec!["README", "trim.txt"], build),
                ("Xcode", library, "Developer/Xcode/DerivedData/*", None, vec!["info.plist"], build),
            ]
        );
    }

    #[test]
    fn only_the_xcode_row_names_an_app_and_a_name_shape() {
        let tools = Tools::shipped().unwrap();

        let special: Vec<_> = tools
            .caches()
            .iter()
            .filter(|row| row.app.is_some() || row.name.is_some())
            .map(|row| (row.id.as_str(), row.app.as_ref().map(BundleId::as_str), row.name))
            .collect();
        assert_eq!(special, [("xcode-derived-data", Some("com.apple.dt.Xcode"), Some(NameShape::ProjectHash))]);
    }

    #[test]
    fn the_shipped_interests_are_ten_folders_whole_then_two_sign_holders_as_items() {
        let tools = Tools::shipped().unwrap();

        let built = tools.interests().unwrap();

        let interests: Vec<_> = built
            .interests
            .iter()
            .map(|interest| {
                let holds: Vec<&str> = interest.holds().iter().map(String::as_str).collect();
                (interest.pattern().clone(), interest.claim(), holds)
            })
            .collect();
        let under = |folder: KnownFolder, path: &str| Pattern::Under(folder, path.into());
        let (home, caches, library) = (KnownFolder::Home, KnownFolder::AppCaches, KnownFolder::Library);
        let (whole, item) = (Claim::Whole, Claim::Item);
        assert_eq!(
            interests,
            [
                (under(home, ".npm/_cacache"), whole, vec!["index-v5", "content-v2"]),
                (under(caches, "Yarn"), whole, vec!["v6"]),
                (under(caches, "Homebrew/downloads"), whole, vec![]),
                (under(home, ".cargo/registry/cache"), whole, vec![]),
                (under(home, ".cargo/registry/src"), whole, vec![]),
                (under(caches, "CocoaPods"), whole, vec!["Pods"]),
                (under(home, ".gradle/caches"), whole, vec!["CACHEDIR.TAG"]),
                (under(caches, "pip"), whole, vec!["http-v2"]),
                (under(caches, "go-build"), whole, vec!["README", "trim.txt"]),
                (under(library, "Developer/Xcode/DerivedData/*"), whole, vec!["info.plist"]),
                (under(caches, "Homebrew"), item, vec!["api"]),
                (under(home, ".cargo/registry"), item, vec!["CACHEDIR.TAG"]),
            ]
        );
        assert!(built.interests.iter().all(|interest| interest.siblings().is_empty()));
    }

    #[test]
    fn each_shipped_interest_has_its_role_and_each_row_its_sign_holder() {
        let tools = Tools::shipped().unwrap();

        let built = tools.interests().unwrap();

        let mut roles: Vec<Role> = (0..10).map(Role::Cache).collect();
        roles.extend([Role::SignHolder(0), Role::SignHolder(1)]);
        assert_eq!(built.roles, roles);
        let holders: Vec<Option<usize>> = (0..10).map(|row| built.holder_of(row)).collect();
        assert_eq!(holders, [None, None, Some(0), Some(1), Some(1), None, None, None, None, None]);
        assert_eq!(built.holder_of(10), None);
    }
}
