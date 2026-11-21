//! The table read from text: every refusal by a table made for it, then the shipped table and its interests.

use std::fmt::Write as _;

use fresher_core::DangerLevel;

/// The categories the tables made here are checked against.
const CATEGORIES: [(&str, DangerLevel); 3] = [
    ("local-ai-models.hugging-face-models", DangerLevel::Low),
    ("local-ai-models.hugging-face-datasets", DangerLevel::Low),
    ("local-ai-models.lm-studio", DangerLevel::Medium),
];

/// A row with `defaults` (key, raw TOML value), each replaced by the one of `fields` with its key, then the rest of
/// `fields` added.
fn row(defaults: &[(&str, &str)], fields: &[(&str, &str)]) -> String {
    let mut text = String::from("[[store]]\n");
    for (key, value) in defaults {
        let value = fields.iter().find(|(field, _)| field == key).map_or(*value, |(_, value)| *value);
        writeln!(text, "{key} = {value}").unwrap();
    }
    for (key, value) in fields.iter().filter(|(key, _)| !defaults.iter().any(|(default, _)| default == key)) {
        writeln!(text, "{key} = {value}").unwrap();
    }
    text
}

/// The hub row with its two kinds, `fields` in place of its own or added to them.
fn hub(fields: &[(&str, &str)]) -> String {
    row(
        &[
            ("id", "\"hugging-face\""),
            ("tool", "\"hugging-face\""),
            ("folder", "\"home\""),
            ("path", "\".cache/huggingface/hub/*\""),
            ("shape", "\"hub-repo\""),
            ("sign", "[\"blobs\", \"snapshots\"]"),
        ],
        fields,
    ) + &kinds(&[("models", "hugging-face-models"), ("datasets", "hugging-face-datasets")])
}

/// `[[store.kind]]` entries for the kinds given, each a kind and the id of its category after `local-ai-models.`.
fn kinds(list: &[(&str, &str)]) -> String {
    let mut text = String::new();
    for (kind, category) in list {
        writeln!(text, "[[store.kind]]\nkind = \"{kind}\"\ncategory = \"local-ai-models.{category}\"").unwrap();
    }
    text
}

/// LM Studio's row, `fields` in place of its own or added to them.
fn lm(fields: &[(&str, &str)]) -> String {
    row(
        &[
            ("id", "\"lm-studio\""),
            ("tool", "\"lm-studio\""),
            ("folder", "\"home\""),
            ("path", "\".lmstudio/models/*/*\""),
            ("shape", "\"publisher-model\""),
            ("app", "\"ai.elementlabs.lmstudio\""),
            ("category", "\"local-ai-models.lm-studio\""),
        ],
        fields,
    )
}

mod negative {
    use super::{CATEGORIES, hub, kinds, lm};
    use crate::stores::{Stores, StoresError};

    fn parse(text: &str) -> Result<Stores, StoresError> {
        Stores::parse(text, &CATEGORIES)
    }

    #[test]
    fn refuses_text_that_is_not_a_table() {
        for text in [
            "[[store]]\nid = 1\n".to_owned(),
            hub(&[("size", "3")]) + &lm(&[]),
            hub(&[("folder", "\"nowhere\"")]) + &lm(&[]),
            hub(&[("tool", "\"ollama\"")]) + &lm(&[]),
            hub(&[("shape", "\"round\"")]) + &lm(&[]),
            hub(&[("sign", "\"blobs\"")]) + &lm(&[]),
        ] {
            assert!(matches!(parse(&text), Err(StoresError::Format(_))), "{text}");
        }
    }

    #[test]
    fn refuses_a_table_without_a_row() {
        assert_eq!(parse(""), Err(StoresError::Empty));
    }

    #[test]
    fn refuses_a_row_id_that_is_not_kebab_case() {
        for id in ["\"\"", "\"Hub\"", "\"hub_repo\"", "\"-hub\"", "\"hub-\"", "\"hub--repo\"", "\"hub.repo\""] {
            let text = hub(&[("id", id)]) + &lm(&[]);

            assert!(matches!(parse(&text), Err(StoresError::BadId { .. })), "{id}");
        }
    }

    #[test]
    fn refuses_a_row_id_listed_twice() {
        let text = hub(&[]) + &lm(&[("id", "\"hugging-face\"")]);

        assert_eq!(parse(&text), Err(StoresError::DuplicateId { id: "hugging-face".into() }));
    }

    #[test]
    fn refuses_a_category_that_is_not_the_modules() {
        let text = hub(&[]) + &lm(&[("category", "\"local-ai-models.ollama\"")]);

        assert_eq!(
            parse(&text),
            Err(StoresError::UnknownCategory { id: "lm-studio".into(), category: "local-ai-models.ollama".into() })
        );
    }

    #[test]
    fn refuses_a_publisher_row_with_no_category() {
        let text = "[[store]]\nid = \"lm-studio\"\ntool = \"lm-studio\"\nfolder = \"home\"\npath = \".a/*/*\"\nshape = \"publisher-model\"\n";

        assert_eq!(parse(text), Err(StoresError::CategoryMissing { id: "lm-studio".into() }));
    }

    #[test]
    fn refuses_a_category_of_the_module_that_no_row_uses() {
        let text = lm(&[]);

        assert_eq!(
            parse(&text),
            Err(StoresError::CategoryWithoutRow { category: "local-ai-models.hugging-face-models".into() })
        );
    }

    #[test]
    fn refuses_a_place_that_is_not_a_plain_relative_path() {
        for path in ["\"/a/*\"", "\"a//*\"", "\"../a/*\"", "\"a/./*\"", "\"a/*/\"", "\"\""] {
            let text = hub(&[("path", path)]) + &lm(&[]);

            assert!(matches!(parse(&text), Err(StoresError::NotRelative { .. })), "{path}");
        }
    }

    #[test]
    fn refuses_a_place_that_does_not_end_in_a_star() {
        let text = hub(&[("path", "\".cache/huggingface/hub\"")]) + &lm(&[]);

        assert!(matches!(parse(&text), Err(StoresError::NoStar { .. })));
    }

    #[test]
    fn refuses_a_place_its_shape_does_not_fit() {
        for (shape_row, path) in [
            (hub(&[("path", "\"a/*/*\"")]), "hub-repo with two stars"),
            (hub(&[("path", "\"*\"")]), "hub-repo with nothing before the star"),
            (hub(&[("path", "\"*/a/*\"")]), "hub-repo with a star before a name"),
            (hub(&[("path", "\"a*/*\"")]), "hub-repo with a part of a name"),
        ] {
            assert!(matches!(parse(&(shape_row + &lm(&[]))), Err(StoresError::ShapeMismatch { .. })), "{path}");
        }
        for path in ["\"a/*\"", "\"a/*/*/*\"", "\"*/*\"", "\"*/a/*\""] {
            let text = hub(&[]) + &lm(&[("path", path)]);

            assert!(matches!(parse(&text), Err(StoresError::ShapeMismatch { .. })), "{path}");
        }
    }

    #[test]
    fn refuses_a_hub_row_without_kinds() {
        let text = super::row(
            &[
                ("id", "\"hugging-face\""),
                ("tool", "\"hugging-face\""),
                ("folder", "\"home\""),
                ("path", "\".cache/hub/*\""),
                ("shape", "\"hub-repo\""),
            ],
            &[],
        ) + &lm(&[]);

        assert_eq!(parse(&text), Err(StoresError::KindsMissing { id: "hugging-face".into() }));
    }

    #[test]
    fn refuses_a_hub_row_with_a_category_of_its_own() {
        let text = hub(&[("category", "\"local-ai-models.lm-studio\"")]) + &lm(&[]);

        assert_eq!(parse(&text), Err(StoresError::CategoryNotAllowed { id: "hugging-face".into() }));
    }

    #[test]
    fn refuses_kinds_on_a_publisher_row() {
        let text = hub(&[]) + &lm(&[]) + &kinds(&[("models", "lm-studio")]);

        assert_eq!(parse(&text), Err(StoresError::KindsNotAllowed { id: "lm-studio".into() }));
    }

    #[test]
    fn refuses_a_kind_listed_twice() {
        let text = hub(&[]) + &kinds(&[("models", "hugging-face-models")]) + &lm(&[]);

        assert_eq!(parse(&text), Err(StoresError::DuplicateKind { id: "hugging-face".into(), kind: "models".into() }));
    }

    #[test]
    fn refuses_a_sign_or_a_kind_that_is_not_one_name() {
        for sign in ["[\"\"]", "[\"a/b\"]", "[\"..\"]", "[\"blobs\", \".\"]"] {
            let text = hub(&[("sign", sign)]) + &lm(&[]);

            assert!(matches!(parse(&text), Err(StoresError::NotAName { .. })), "{sign}");
        }
        let text = hub(&[]).replace("kind = \"datasets\"", "kind = \"data--sets\"") + &lm(&[]);

        assert!(matches!(parse(&text), Err(StoresError::NotAName { .. })));
    }

    #[test]
    fn refuses_a_bundle_id_that_is_not_one() {
        let text = hub(&[]) + &lm(&[("app", "\"not a bundle id\"")]);

        assert!(matches!(parse(&text), Err(StoresError::BundleId(_))));
    }
}

mod positive {
    use fresher_core::{Claim, KnownFolder, Pattern};

    use crate::stores::{Categories, Shape, Stores, Tool};

    fn shipped() -> Stores {
        Stores::shipped().expect("the shipped table is valid")
    }

    #[test]
    fn the_shipped_table_has_its_three_rows_in_order() {
        let shipped = shipped();

        let rows: Vec<(&str, Tool, Shape)> =
            shipped.stores().iter().map(|store| (store.id.as_str(), store.tool, store.shape)).collect();

        assert_eq!(
            rows,
            [
                ("hugging-face", Tool::HuggingFace, Shape::HubRepo),
                ("lm-studio", Tool::LmStudio, Shape::PublisherModel),
                ("lm-studio-old", Tool::LmStudio, Shape::PublisherModel),
            ]
        );
    }

    #[test]
    fn the_shipped_table_uses_every_category() {
        let shipped = shipped();

        for (category, _) in crate::categories::CATEGORIES {
            assert!(shipped.stores().iter().any(|store| store.categories.contains(category)), "{category}");
        }
    }

    #[test]
    fn the_hub_row_has_its_kinds_and_its_sign() {
        let shipped = shipped();
        let hub = &shipped.stores()[0];

        assert_eq!(
            hub.categories,
            Categories::ByKind(vec![
                ("models".into(), "local-ai-models.hugging-face-models"),
                ("datasets".into(), "local-ai-models.hugging-face-datasets"),
            ])
        );
        assert_eq!(hub.sign, ["blobs", "snapshots"]);
        assert_eq!(hub.app, None);
    }

    #[test]
    fn the_lm_studio_rows_have_no_sign_and_name_the_app() {
        let shipped = shipped();

        for store in &shipped.stores()[1..] {
            assert_eq!(store.sign, Vec::<String>::new());
            assert_eq!(store.app.as_ref().map(ToString::to_string).as_deref(), Some("ai.elementlabs.lmstudio"));
            assert_eq!(store.categories, Categories::One("local-ai-models.lm-studio"));
        }
    }

    #[test]
    fn the_interests_are_three_whole_claims_in_the_tables_order() {
        let interests = shipped().interests().expect("the interests are valid");

        let places: Vec<&Pattern> = interests.iter().map(fresher_core::Interest::pattern).collect();

        assert_eq!(
            places,
            [
                &Pattern::Under(KnownFolder::Home, ".cache/huggingface/hub/*".into()),
                &Pattern::Under(KnownFolder::Home, ".lmstudio/models/*/*".into()),
                &Pattern::Under(KnownFolder::Home, ".cache/lm-studio/models/*/*".into()),
            ]
        );
        assert!(interests.iter().all(|interest| interest.claim() == Claim::Whole));
        assert_eq!(interests[0].holds(), ["blobs", "snapshots"]);
        assert_eq!(interests[1].holds(), Vec::<String>::new());
        assert_eq!(interests[2].holds(), Vec::<String>::new());
    }
}
