//! The table read from text: every refusal by a table made for it, then the shipped table and its interests.

use fresher_core::DangerLevel;

use crate::toolchains::{Toolchains, ToolchainsError};

/// The categories and settings the tables made here are checked against.
const CATEGORIES: [(&str, DangerLevel); 2] =
    [("unused-toolchains.rust", DangerLevel::Low), ("unused-toolchains.node", DangerLevel::Medium)];
const SETTINGS: [&str; 1] = ["node-versions"];

/// A table of two rows, each with its fields, `rust` and `node` replaced where the key is present.
fn table(rust: &str, node: &str) -> String {
    format!("[[row]]\n{rust}\n[[row]]\n{node}\n")
}

const RUST: &str = r#"id = "rustup"
manager = "rustup"
versions = ".rustup/toolchains/*"
shape = "rustup-toolchain"
pointers = [{ place = ".rustup", holds = "settings.toml" }]
pins = [{ name = "rust-toolchain" }, { name = ".tool-versions", keys = ["rust"] }]
category = "unused-toolchains.rust""#;

const NODE: &str = r#"id = "nvm"
manager = "nvm"
versions = ".nvm/versions/node/*"
shape = "node-version"
pins = [{ name = ".nvmrc" }, { name = ".tool-versions", keys = ["node"] }]
category = "unused-toolchains.node"
switch = "node-versions""#;

fn parse(text: &str) -> Result<Toolchains, ToolchainsError> {
    Toolchains::parse(text, &CATEGORIES, &SETTINGS)
}

/// `RUST` or `NODE` with one line replaced by `line`.
fn with(base: &str, key: &str, line: &str) -> String {
    base.lines().map(|l| if l.starts_with(&format!("{key} =")) { line } else { l }).collect::<Vec<_>>().join("\n")
}

mod negative {
    use super::{NODE, RUST, parse, table, with};
    use crate::toolchains::ToolchainsError;

    fn rust_with(key: &str, line: &str) -> String {
        table(&with(RUST, key, line), NODE)
    }

    fn node_with(key: &str, line: &str) -> String {
        table(RUST, &with(NODE, key, line))
    }

    #[test]
    fn refuses_text_that_is_not_a_table() {
        assert!(matches!(parse("[[row"), Err(ToolchainsError::Format(_))));
        assert!(matches!(parse(&format!("{}\nsurprise = 1", table(RUST, NODE))), Err(ToolchainsError::Format(_))));
    }

    #[test]
    fn refuses_a_table_without_rows() {
        assert_eq!(parse(""), Err(ToolchainsError::Empty));
    }

    #[test]
    fn refuses_a_row_id_that_is_not_kebab_case() {
        let error = parse(&rust_with("id", "id = \"Rust_Up\"")).unwrap_err();

        assert_eq!(error, ToolchainsError::BadId { id: "Rust_Up".into() });
    }

    #[test]
    fn refuses_a_row_id_listed_twice() {
        let error = parse(&node_with("id", "id = \"rustup\"")).unwrap_err();

        assert_eq!(error, ToolchainsError::DuplicateId { id: "rustup".into() });
    }

    #[test]
    fn refuses_a_category_that_is_not_the_modules() {
        let error = parse(&rust_with("category", "category = \"unused-toolchains.go\"")).unwrap_err();

        assert_eq!(
            error,
            ToolchainsError::UnknownCategory { id: "rustup".into(), category: "unused-toolchains.go".into() }
        );
    }

    #[test]
    fn refuses_a_category_no_row_uses() {
        let error = parse(&format!("[[row]]\n{RUST}\n")).unwrap_err();

        assert_eq!(error, ToolchainsError::CategoryWithoutRow { category: "unused-toolchains.node".into() });
    }

    #[test]
    fn refuses_places_that_are_not_relative_plain_paths() {
        for path in ["/abs/*", "a//*", "../a/*", "a/./*", ""] {
            let error = parse(&rust_with("versions", &format!("versions = \"{path}\""))).unwrap_err();
            assert!(matches!(error, ToolchainsError::NotRelative { .. }), "{path}: {error:?}");
        }
        let pointer = "pointers = [{ place = \"a/../b\" }]";
        assert!(matches!(parse(&rust_with("pointers", pointer)), Err(ToolchainsError::NotRelative { .. })));
    }

    #[test]
    fn refuses_a_versions_place_that_does_not_end_in_a_star() {
        let error = parse(&rust_with("versions", "versions = \".rustup/toolchains\"")).unwrap_err();

        assert_eq!(error, ToolchainsError::NoStar { id: "rustup".into(), path: ".rustup/toolchains".into() });
    }

    #[test]
    fn refuses_a_name_that_is_not_one_name() {
        for pins in [
            "pins = [{ name = \"a/b\" }]",
            "pins = [{ name = \"\" }]",
            "pins = [{ name = \".tool-versions\", keys = [\"a b\"] }]",
        ] {
            let error = parse(&rust_with("pins", pins)).unwrap_err();
            assert!(matches!(error, ToolchainsError::NotAName { .. }), "{pins}: {error:?}");
        }
        let holds = "pointers = [{ place = \".rustup\", holds = \"a/b\" }]";
        assert!(matches!(parse(&rust_with("pointers", holds)), Err(ToolchainsError::NotAName { .. })));
    }

    #[test]
    fn refuses_a_tool_versions_pin_without_keys() {
        let error = parse(&rust_with("pins", "pins = [{ name = \".tool-versions\" }]")).unwrap_err();

        assert_eq!(error, ToolchainsError::KeysMissing { id: "rustup".into(), name: ".tool-versions".into() });
    }

    #[test]
    fn refuses_a_key_listed_twice() {
        let pins = "pins = [{ name = \".tool-versions\", keys = [\"rust\", \"rust\"] }]";

        let error = parse(&rust_with("pins", pins)).unwrap_err();

        assert_eq!(
            error,
            ToolchainsError::DuplicateKey { id: "rustup".into(), name: ".tool-versions".into(), key: "rust".into() }
        );
    }

    #[test]
    fn refuses_a_switch_that_is_not_a_setting_of_the_module() {
        let error = parse(&node_with("switch", "switch = \"enabled-now\"")).unwrap_err();

        assert_eq!(error, ToolchainsError::UnknownSwitch { id: "nvm".into(), switch: "enabled-now".into() });
    }
}

mod positive {
    use fresher_core::{Claim, Pattern};

    use super::{CATEGORIES, NODE, RUST, SETTINGS, table};
    use crate::toolchains::{InterestKind, Manager, Shape, Toolchains};

    #[test]
    fn the_shipped_table_has_the_two_rows() {
        let toolchains = Toolchains::shipped().unwrap();

        let rows = toolchains.rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            (rows[0].id.as_str(), rows[0].manager, rows[0].shape),
            ("rustup", Manager::Rustup, Shape::RustupToolchain)
        );
        assert_eq!((rows[1].id.as_str(), rows[1].manager, rows[1].shape), ("nvm", Manager::Nvm, Shape::NodeVersion));
        assert_eq!([rows[0].category, rows[1].category], [CATEGORIES[0].0, CATEGORIES[1].0]);
        assert_eq!((rows[0].switch, rows[1].switch), (None, Some(SETTINGS[0])));
        assert_eq!(
            rows[0].pins.iter().map(|pin| pin.name.as_str()).collect::<Vec<_>>(),
            ["rust-toolchain.toml", "rust-toolchain", ".tool-versions"]
        );
        assert_eq!(rows[1].pins[2].keys, ["nodejs", "node"]);
    }

    #[test]
    fn the_shipped_interests_are_in_the_designs_order() {
        let toolchains = Toolchains::shipped().unwrap();

        let interests = toolchains.interests().unwrap();

        let kinds: Vec<InterestKind> = toolchains.kinds().to_vec();
        assert_eq!(
            kinds,
            [
                InterestKind::Versions(0),
                InterestKind::Pointer(0, 0),
                InterestKind::Versions(1),
                InterestKind::Pointer(1, 0),
                InterestKind::Pointer(1, 1),
                InterestKind::Pin("rust-toolchain.toml".into()),
                InterestKind::Pin("rust-toolchain".into()),
                InterestKind::Pin(".tool-versions".into()),
                InterestKind::Pin(".nvmrc".into()),
                InterestKind::Pin(".node-version".into()),
            ]
        );
        assert_eq!(interests.len(), kinds.len());
        let claims: Vec<Claim> = interests.iter().map(fresher_core::Interest::claim).collect();
        assert_eq!(&claims[..5], [Claim::Whole, Claim::Item, Claim::Whole, Claim::Item, Claim::Item]);
        assert!(claims[5..].iter().all(|claim| *claim == Claim::Item));
        assert!(matches!(interests[0].pattern(), Pattern::Under(_, path) if path == ".rustup/toolchains/*"));
        assert!(matches!(interests[1].pattern(), Pattern::Under(_, path) if path == ".rustup"));
        assert_eq!(interests[1].holds(), ["settings.toml"]);
        assert!(matches!(interests[3].pattern(), Pattern::Under(_, path) if path == ".nvm/alias/*"));
        assert!(matches!(interests[4].pattern(), Pattern::Under(_, path) if path == ".nvm/alias/lts/*"));
        assert!(matches!(interests[7].pattern(), Pattern::MarkerFile(name) if name == ".tool-versions"));
    }

    #[test]
    fn a_made_table_with_both_rows_is_read() {
        let toolchains = super::parse(&table(RUST, NODE)).unwrap();

        assert_eq!(toolchains.rows().len(), 2);
        assert_eq!(toolchains.rows()[1].pointers, []);
    }
}
