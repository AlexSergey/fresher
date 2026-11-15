//! Versions in names (`2026.3`, `2025.2.1`): captured by a rule's pattern, ordered by their numeric parts, so that a
//! module can tell the current version of a product from the stale ones.

use std::collections::BTreeMap;

use fresher_core::Match;

use crate::RuleSet;

/// One part of a version pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    /// A `{name}` placeholder: a run of digits.
    Number,
    /// Text between placeholders, matched as it is.
    Literal(String),
}

/// A version pattern such as `{major}.{minor}`: placeholders capture runs of digits, the text between them is matched
/// as it is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VersionPattern {
    tokens: Vec<Token>,
}

/// A version: its numeric parts, compared part by part (`2025.10` is newer than `2025.9`).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Version(Vec<u64>);

/// A name split into the product (the name with its version cut out) and the version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Versioned {
    pub(crate) product: String,
    pub(crate) version: Version,
}

/// A product: the rule that serves the match and the name with its version cut out.
type Product = (usize, String);

impl VersionPattern {
    /// The pattern of `template`; `None` unless it holds at least one placeholder, every placeholder is a non-empty
    /// name, two placeholders are never adjacent, and the text between them holds no digit and no brace.
    pub(crate) fn new(template: &str) -> Option<Self> {
        let mut tokens = Vec::new();
        let mut rest = template;
        while !rest.is_empty() {
            if let Some(after) = rest.strip_prefix('{') {
                let (name, tail) = after.split_once('}')?;
                let valid_name =
                    !name.is_empty() && name.chars().all(|char| char.is_ascii_alphanumeric() || char == '-');
                if !valid_name || tokens.last() == Some(&Token::Number) {
                    return None;
                }
                tokens.push(Token::Number);
                rest = tail;
            } else {
                let end = rest.find('{').unwrap_or(rest.len());
                let (literal, tail) = rest.split_at(end);
                if literal.contains(|char: char| char.is_ascii_digit() || char == '}' || char == '/' || char == '\0') {
                    return None;
                }
                tokens.push(Token::Literal(literal.into()));
                rest = tail;
            }
        }
        tokens.contains(&Token::Number).then_some(Self { tokens })
    }

    /// The last place in `name` where the pattern matches, with whole runs of digits only (`10.2` is never read out
    /// of `110.25`); `None` when it matches nowhere.
    pub(crate) fn find(&self, name: &str) -> Option<Versioned> {
        let starts = name.char_indices().map(|(start, _)| start).rev();
        starts.filter(|&start| name.get(..start).is_some_and(|before| !ends_with_digit(before))).find_map(|start| {
            let (end, parts) = self.match_at(name, start)?;
            let (before, after) = (name.get(..start)?, name.get(end..)?);
            (!after.starts_with(|char: char| char.is_ascii_digit()))
                .then(|| Versioned { product: format!("{before}{after}"), version: Version(parts) })
        })
    }

    /// Where a match starting at `start` ends, and the numbers it captured.
    fn match_at(&self, name: &str, start: usize) -> Option<(usize, Vec<u64>)> {
        let mut rest = name.get(start..)?;
        let mut parts = Vec::new();
        for token in &self.tokens {
            match token {
                Token::Number => {
                    let digits = rest.find(|char: char| !char.is_ascii_digit()).unwrap_or(rest.len());
                    let (number, tail) = rest.split_at(digits);
                    parts.push(number.parse().ok()?);
                    rest = tail;
                }
                Token::Literal(literal) => rest = rest.strip_prefix(literal.as_str())?,
            }
        }
        Some((name.len() - rest.len(), parts))
    }
}

fn ends_with_digit(text: &str) -> bool {
    text.ends_with(|char: char| char.is_ascii_digit())
}

impl RuleSet {
    /// The match holding the newest version of each product, among `matches` of rules with a `version` pattern,
    /// ordered by product. A product is one rule and the name with its version cut out (`IntelliJIdea` of
    /// `IntelliJIdea2026.3`). The other versioned matches of a product are [`RuleSet::stale`].
    pub fn newest_per_product<'m>(&self, matches: &'m [Match]) -> Vec<&'m Match> {
        let newest = self.newest_versions(matches);
        let mut picked: BTreeMap<Product, &'m Match> = BTreeMap::new();
        for (matched, product, version) in self.versioned(matches) {
            if newest.get(&product) == Some(&version) {
                picked.entry(product).or_insert(matched);
            }
        }
        picked.into_values().collect()
    }

    /// The matches whose version is older than the newest of the same product among `matches`, in their order. The
    /// newest version of a product is never stale, and neither is a product seen in one version only.
    pub fn stale<'m>(&self, matches: &'m [Match]) -> Vec<&'m Match> {
        let newest = self.newest_versions(matches);
        self.versioned(matches)
            .filter(|(_, product, version)| newest.get(product).is_some_and(|newest| version < newest))
            .map(|(matched, _, _)| matched)
            .collect()
    }

    fn newest_versions(&self, matches: &[Match]) -> BTreeMap<Product, Version> {
        let mut newest: BTreeMap<Product, Version> = BTreeMap::new();
        for (_, product, version) in self.versioned(matches) {
            let known = newest.entry(product).or_insert_with(|| version.clone());
            if version > *known {
                *known = version;
            }
        }
        newest
    }

    /// Each match of a versioned rule whose name carries a version, with its product and its version.
    fn versioned<'m>(&self, matches: &'m [Match]) -> impl Iterator<Item = (&'m Match, Product, Version)> {
        matches.iter().filter_map(|matched| {
            let pattern = self.rules.get(matched.interest)?.version.as_ref()?;
            let Versioned { product, version } = pattern.find(matched.path.file_name()?.to_str()?)?;
            Some((matched, (matched.interest, product), version))
        })
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{EntryKind, Match, Sizes};

    use crate::RuleSet;
    use crate::rules::sample;

    /// Two rules: `ides` (versioned, `{major}.{minor}`) and `plain` (no version).
    fn rules() -> RuleSet {
        let toml = r#"
[[rule]]
id = "ides"
category = "example.settings"
danger = "medium"
removal = "trash"
[rule.look]
under-home = "Library/Application Support/Example/*"
[rule.recognize]
version = "{major}.{minor}"
[[rule.must-not-match]]
name = "*.lock"

[[rule]]
id = "plain"
category = "example.caches"
danger = "low"
removal = "trash"
[rule.look]
folder-name = "Caches"
[[rule.must-not-match]]
name = "*.lock"
"#;
        RuleSet::parse(&sample::module(), toml).unwrap()
    }

    fn matched(interest: usize, name: &str) -> Match {
        Match {
            interest,
            path: format!("/home/Library/Application Support/Example/{name}").into(),
            kind: EntryKind::Folder,
            sizes: Sizes::ZERO,
            modified: None,
            siblings: vec![],
        }
    }

    fn names<'m>(matches: impl IntoIterator<Item = &'m Match>) -> Vec<String> {
        matches.into_iter().map(|matched| matched.path.file_name().unwrap().to_string_lossy().into_owned()).collect()
    }

    mod negative {
        use super::{matched, names, rules};
        use crate::version::VersionPattern;

        #[test]
        fn rejects_a_pattern_without_a_clear_placeholder() {
            for template in ["", "2026", "{}", "{major", "{major}{minor}", "{major}.1{minor}", "{a b}", "{major}}"] {
                assert_eq!(VersionPattern::new(template), None, "{template:?}");
            }
        }

        #[test]
        fn finds_no_version_in_a_name_without_one() {
            let pattern = VersionPattern::new("{major}.{minor}").unwrap();

            for name in ["IntelliJIdea", "Idea2026", "Idea.3", "Idea2026.", "Idea99999999999999999999.1"] {
                assert_eq!(pattern.find(name), None, "{name}");
            }
        }

        #[test]
        fn the_current_version_is_not_stale() {
            let matches = [matched(0, "Idea2025.2"), matched(0, "Idea2026.1"), matched(0, "Idea2025.10")];

            let stale = rules().stale(&matches);

            assert_eq!(names(stale), ["Idea2025.2", "Idea2025.10"]);
        }

        #[test]
        fn a_product_in_one_version_or_without_one_is_never_stale() {
            let matches =
                [matched(0, "Idea2025.2"), matched(0, "Goland2024.1"), matched(1, "Idea2024.1"), matched(0, "Idea")];
            let twice = [matched(0, "Idea2025.2"), matched(0, "Idea2025.2")];

            assert_eq!(names(rules().stale(&matches)), Vec::<String>::new());
            assert_eq!(names(rules().stale(&twice)), Vec::<String>::new());
            assert_eq!(names(rules().newest_per_product(&twice)), ["Idea2025.2"]);
        }
    }

    mod positive {
        use super::{matched, names, rules};
        use crate::version::{Version, VersionPattern, Versioned};

        #[test]
        fn finds_the_last_version_in_a_name() {
            let pattern = VersionPattern::new("{major}.{minor}").unwrap();

            let found = pattern.find("Idea1.2-2026.3.app").unwrap();

            assert_eq!(found, Versioned { product: "Idea1.2-.app".into(), version: Version(vec![2026, 3]) });
            let three = VersionPattern::new("v{a}.{b}.{c}").unwrap().find("tool-v2025.2.1").unwrap();
            assert_eq!(three, Versioned { product: "tool-".into(), version: Version(vec![2025, 2, 1]) });
        }

        #[test]
        fn versions_are_ordered_by_their_numbers() {
            assert!(Version(vec![2025, 10]) > Version(vec![2025, 9]));
            assert!(Version(vec![2026]) > Version(vec![2025, 99]));
            assert!(Version(vec![2025, 2, 1]) > Version(vec![2025, 2]));
        }

        #[test]
        fn older_versions_are_stale_once_a_newer_one_exists() {
            let matches = [matched(0, "Idea2025.2"), matched(0, "Goland2024.1"), matched(0, "Goland2024.3")];
            let newer = [matches.as_slice(), &[matched(0, "Idea2026.1")]].concat();

            assert_eq!(names(rules().stale(&matches)), ["Goland2024.1"]);
            assert_eq!(names(rules().stale(&newer)), ["Idea2025.2", "Goland2024.1"]);
            assert_eq!(names(rules().newest_per_product(&newer)), ["Goland2024.3", "Idea2026.1"]);
        }
    }

    mod property {
        use proptest::prelude::*;

        use crate::version::{Version, VersionPattern};

        fn version() -> impl Strategy<Value = Version> {
            prop::collection::vec(0_u64..20, 1..4).prop_map(Version)
        }

        proptest! {
            #[test]
            fn version_order_is_total_and_transitive(a in version(), b in version(), c in version()) {
                let holding = [a < b, a == b, a > b].into_iter().filter(|&holds| holds).count();
                prop_assert_eq!(holding, 1);
                if a <= b && b <= c {
                    prop_assert!(a <= c);
                }
            }

            #[test]
            fn a_formatted_version_is_found_again(major in 0_u64..100_000, minor in 0_u64..100_000) {
                let pattern = VersionPattern::new("{major}.{minor}").unwrap();

                let found = pattern.find(&format!("Idea{major}.{minor}")).unwrap();

                prop_assert_eq!(found.version, Version(vec![major, minor]));
                prop_assert_eq!(found.product, "Idea");
            }
        }
    }
}
