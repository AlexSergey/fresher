//! A module's rules, parsed from its `rules.toml` and validated against its descriptor.

use std::collections::BTreeSet;

use fresher_core::{
    CategoryDescriptor, Claim, DangerLevel, Interest, InterestError, ModuleDescriptor, Pattern, TextKey,
};

use crate::format::{LookSpec, RemovalSpec, RuleFile, RuleSpec};
use crate::glob::Glob;
use crate::version::VersionPattern;

/// The validated rules of one module, in the order of its `rules.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSet {
    pub(crate) rules: Vec<Rule>,
}

/// One validated rule: where to look, how the item is recognized, what must not match, and what a finding says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub(crate) id: String,
    pub(crate) category: CategoryDescriptor,
    pub(crate) danger: DangerLevel,
    pub(crate) removal: RemovalSpec,
    pub(crate) interest: Interest,
    pub(crate) name: Option<Glob>,
    pub(crate) version: Option<VersionPattern>,
    pub(crate) min_age_days: Option<u32>,
    pub(crate) sibling: Option<String>,
    pub(crate) must_not_match: Vec<Glob>,
    pub(crate) title: TextKey,
    pub(crate) reason: TextKey,
}

/// Why a module's rules were refused. Every variant but `Toml` names the rule by its id.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum RuleError {
    #[error("the rules are not valid TOML for the rule format: {reason}")]
    Toml { reason: String },
    #[error("rule `{rule}` has no `must-not-match`: every rule states what it must never claim")]
    MissingMustNotMatch { rule: String },
    #[error("rule `{rule}` has a danger level below its category's")]
    DangerBelowCategory { rule: String },
    #[error("rule `{rule}` names a category its module does not have")]
    UnknownCategory { rule: String },
    #[error("rule `{rule}` is defined twice")]
    DuplicateId { rule: String },
    #[error("rule `{rule}` has an id that is not a kebab-case name")]
    InvalidId { rule: String },
    #[error("rule `{rule}` looks at a path that is not inside its known folder")]
    PathOutOfFolder { rule: String },
    #[error("rule `{rule}` has a pattern that is not a name with `*` and `?` only")]
    BadPattern { rule: String },
}

impl RuleSet {
    /// Parses the `rules.toml` of `module` and validates every rule: its category is one of the module's, its danger
    /// level is not below the category's, it has at least one `must-not-match`, its ids are unique and its patterns
    /// are plain names with `*` and `?`, inside the home folder.
    pub fn parse(module: &ModuleDescriptor, toml: &str) -> Result<Self, RuleError> {
        let file: RuleFile = toml::from_str(toml).map_err(|error| RuleError::Toml { reason: error.to_string() })?;
        let mut ids = BTreeSet::new();
        let mut rules = Vec::with_capacity(file.rule.len());
        for spec in file.rule {
            if !ids.insert(spec.id.clone()) {
                return Err(RuleError::DuplicateId { rule: spec.id });
            }
            rules.push(Rule::validate(module, spec)?);
        }
        Ok(Self { rules })
    }

    /// The rules, in the order of the file.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}

impl Rule {
    fn validate(module: &ModuleDescriptor, spec: RuleSpec) -> Result<Self, RuleError> {
        let id = spec.id;
        let bad_pattern = || RuleError::BadPattern { rule: id.clone() };
        let key = |text: &str| {
            TextKey::new(format!("{}.rule.{id}.{text}", module.id()))
                .map_err(|_| RuleError::InvalidId { rule: id.clone() })
        };
        let (title, reason) = (key("title")?, key("reason")?);
        let category = module
            .categories()
            .iter()
            .find(|category| category.id().as_str() == spec.category)
            .cloned()
            .ok_or_else(|| RuleError::UnknownCategory { rule: id.clone() })?;
        if spec.danger < category.danger() {
            return Err(RuleError::DangerBelowCategory { rule: id });
        }
        if spec.must_not_match.is_empty() {
            return Err(RuleError::MissingMustNotMatch { rule: id });
        }
        let must_not_match = spec
            .must_not_match
            .iter()
            .map(|spec| Glob::new(&spec.name))
            .collect::<Option<Vec<_>>>()
            .ok_or_else(bad_pattern)?;
        let recognize = spec.recognize;
        let name = recognize.name.as_deref().map(Glob::new).map(|glob| glob.ok_or_else(bad_pattern)).transpose()?;
        let version = recognize
            .version
            .as_deref()
            .map(VersionPattern::new)
            .map(|pattern| pattern.ok_or_else(bad_pattern))
            .transpose()?;
        let interest =
            compile_look(spec.look, recognize.sibling.iter().cloned().collect()).map_err(|error| match error {
                InterestError::OutsideFolder { .. } => RuleError::PathOutOfFolder { rule: id.clone() },
                // A rule names no held entries: the last two cannot come from `Interest::new`.
                InterestError::EmptyName
                | InterestError::NotAName { .. }
                | InterestError::HoldsNotAnchored
                | InterestError::TooManyHolds { .. } => bad_pattern(),
            })?;
        if let Pattern::Under(_, path) = interest.pattern()
            && path.split('/').any(|segment| Glob::new(segment).is_none())
        {
            return Err(bad_pattern());
        }
        Ok(Self {
            id,
            category,
            danger: spec.danger,
            removal: spec.removal,
            interest,
            name,
            version,
            min_age_days: recognize.min_age_days,
            sibling: recognize.sibling,
            must_not_match,
            title,
            reason,
        })
    }

    /// The id the rule has in its file; findings name it.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The category the rule produces.
    pub const fn category(&self) -> &CategoryDescriptor {
        &self.category
    }

    /// The danger level of the rule's findings: never below the category's.
    pub const fn danger(&self) -> DangerLevel {
        self.danger
    }
}

/// The interest a rule's `[rule.look]` compiles to; a rule claims the matched item as a whole.
fn compile_look(look: LookSpec, siblings: Vec<String>) -> Result<Interest, InterestError> {
    let pattern = match look {
        LookSpec::Under(under) => Pattern::Under(under.folder, under.path),
        LookSpec::FolderName(name) => Pattern::FolderName(name),
        LookSpec::Marker(name) => Pattern::MarkerFile(name),
        LookSpec::Extension(extension) => Pattern::Extension(extension),
    };
    Interest::new(pattern, Claim::Whole, siblings)
}

#[cfg(test)]
pub(crate) mod sample {
    use fresher_core::{CategoryDescriptor, CategoryId, DangerLevel, Locales, ModuleDescriptor, ModuleId};

    /// A module `example` with `example.caches` (Low) and `example.settings` (Medium).
    pub(crate) fn module() -> ModuleDescriptor {
        let id = ModuleId::new("example").unwrap();
        let category = |name, danger| CategoryDescriptor::new(CategoryId::new(id, name).unwrap(), danger);
        let categories =
            vec![category("example.caches", DangerLevel::Low), category("example.settings", DangerLevel::Medium)];
        ModuleDescriptor::new(id, fresher_core::ModuleGroup::Development, categories, vec![], Locales { en: "{}" })
            .unwrap()
    }

    /// A complete rule; `extra` is appended inside its `[[rule]]` table, before `must-not-match`.
    pub(crate) const COMPLETE: &str = r#"
[[rule]]
id = "caches"
category = "example.caches"
danger = "medium"
removal = "empty-folder"
[rule.look]
under = { folder = "home", path = "Library/Caches/*" }
[rule.recognize]
name = "com.example.*"
version = "{major}.{minor}"
min-age-days = 30
sibling = "package.json"
[[rule.must-not-match]]
name = "com.apple.*"
"#;
}

#[cfg(test)]
mod tests {
    use crate::{RuleError, RuleSet};

    use super::sample;

    fn parse(toml: &str) -> Result<RuleSet, RuleError> {
        RuleSet::parse(&sample::module(), toml)
    }

    /// A minimal valid rule with one line replaced.
    fn minimal_with(from: &str, to: &str) -> String {
        let minimal = "[[rule]]\nid = \"caches\"\ncategory = \"example.caches\"\ndanger = \"low\"\nremoval = \"trash\"\n\
                       [rule.look]\nfolder-name = \"Caches\"\n[[rule.must-not-match]]\nname = \"com.apple.*\"\n";
        assert!(minimal.contains(from), "{from:?} is not in the minimal rule");
        minimal.replacen(from, to, 1)
    }

    mod negative {
        use super::{minimal_with, parse};
        use crate::RuleError;

        fn caches() -> String {
            "caches".into()
        }

        #[test]
        fn rejects_a_rule_without_must_not_match() {
            let toml = minimal_with("[[rule.must-not-match]]\nname = \"com.apple.*\"\n", "");

            assert_eq!(parse(&toml), Err(RuleError::MissingMustNotMatch { rule: caches() }));
        }

        #[test]
        fn rejects_a_danger_level_below_its_category() {
            let toml = minimal_with(
                "category = \"example.caches\"\ndanger = \"low\"",
                "category = \"example.settings\"\ndanger = \"low\"",
            );

            assert_eq!(parse(&toml), Err(RuleError::DangerBelowCategory { rule: caches() }));
        }

        #[test]
        fn rejects_a_category_of_another_module() {
            for category in ["docker.images", "example.unknown", "example"] {
                let toml = minimal_with("\"example.caches\"", &format!("\"{category}\""));

                assert_eq!(parse(&toml), Err(RuleError::UnknownCategory { rule: caches() }), "{category}");
            }
        }

        #[test]
        fn rejects_a_folder_that_is_not_a_known_folder() {
            for under in [
                r#"under = { folder = "caches", path = "*" }"#,
                r#"under = { folder = "/Library", path = "*" }"#,
                r#"under = { path = "*" }"#,
                r#"under = { folder = "home", path = "*", deep = true }"#,
                r#"under-home = "Library""#,
            ] {
                let toml = minimal_with("folder-name = \"Caches\"", under);

                assert!(matches!(parse(&toml), Err(RuleError::Toml { .. })), "{under}");
            }
        }

        #[test]
        fn rejects_a_path_that_climbs_out_of_its_folder() {
            for path in ["../Library", "Library/../../etc", "/Library/Caches"] {
                let toml = minimal_with(
                    "folder-name = \"Caches\"",
                    &format!("under = {{ folder = \"home\", path = \"{path}\" }}"),
                );

                assert_eq!(parse(&toml), Err(RuleError::PathOutOfFolder { rule: caches() }), "{path}");
            }
        }

        #[test]
        fn rejects_an_unknown_field() {
            for (from, to) in [
                ("removal = \"trash\"", "removal = \"trash\"\ncolour = \"red\""),
                ("folder-name = \"Caches\"", "folder-name = \"Caches\"\nmarker = \"CACHEDIR.TAG\""),
                ("folder-name = \"Caches\"", "regex = \"Caches\""),
                ("name = \"com.apple.*\"", "name = \"com.apple.*\"\npath = \"Library\""),
                ("removal = \"trash\"", "removal = \"shred\""),
            ] {
                let toml = minimal_with(from, to);

                assert!(matches!(parse(&toml), Err(RuleError::Toml { .. })), "{to}");
            }
        }

        #[test]
        fn rejects_a_duplicate_rule_id() {
            let rule = minimal_with("", "");

            assert_eq!(parse(&format!("{rule}{rule}")), Err(RuleError::DuplicateId { rule: caches() }));
        }

        #[test]
        fn rejects_an_id_that_is_not_kebab_case() {
            let toml = minimal_with("id = \"caches\"", "id = \"Old Caches\"");

            assert_eq!(parse(&toml), Err(RuleError::InvalidId { rule: "Old Caches".into() }));
        }

        #[test]
        fn rejects_a_glob_with_brackets() {
            for (from, to) in [
                ("name = \"com.apple.*\"", "name = \"com.[ab]*\""),
                ("folder-name = \"Caches\"", "under = { folder = \"home\", path = \"Library/[Cc]aches\" }"),
                ("folder-name = \"Caches\"", "folder-name = \"a/b\""),
                ("removal = \"trash\"", "removal = \"trash\"\n[rule.recognize]\nname = \"{a,b}\""),
                ("removal = \"trash\"", "removal = \"trash\"\n[rule.recognize]\nversion = \"{major}{minor}\""),
                ("removal = \"trash\"", "removal = \"trash\"\n[rule.recognize]\nsibling = \"a/package.json\""),
            ] {
                let toml = minimal_with(from, to);

                assert_eq!(parse(&toml), Err(RuleError::BadPattern { rule: caches() }), "{to}");
            }
        }

        #[test]
        fn errors_name_the_rule() {
            let error = RuleError::MissingMustNotMatch { rule: caches() };

            assert_eq!(
                error.to_string(),
                "rule `caches` has no `must-not-match`: every rule states what it must never claim"
            );
        }
    }

    mod positive {
        use fresher_core::{Claim, DangerLevel, Interest, KnownFolder, Pattern, TextKey};

        use super::{minimal_with, parse};
        use crate::format::RemovalSpec;
        use crate::glob::Glob;
        use crate::rules::sample;
        use crate::version::VersionPattern;

        #[test]
        fn parses_a_complete_rule() {
            let rules = parse(sample::COMPLETE).unwrap();
            assert_eq!(rules.rules().len(), 1);
            let rule = &rules.rules()[0];

            assert_eq!(rule.id(), "caches");
            assert_eq!(rule.category().id().as_str(), "example.caches");
            assert_eq!(rule.danger(), DangerLevel::Medium);
            assert_eq!(rule.removal, RemovalSpec::EmptyFolder);
            let interest = Interest::new(
                Pattern::Under(KnownFolder::Home, "Library/Caches/*".into()),
                Claim::Whole,
                vec!["package.json".into()],
            );
            assert_eq!(rule.interest, interest.unwrap());
            assert_eq!(rule.name, Glob::new("com.example.*"));
            assert_eq!(rule.version, VersionPattern::new("{major}.{minor}"));
            assert_eq!(rule.min_age_days, Some(30));
            assert_eq!(rule.sibling.as_deref(), Some("package.json"));
            assert_eq!(rule.must_not_match, [Glob::new("com.apple.*").unwrap()]);
            assert_eq!(rule.title, TextKey::new("example.rule.caches.title").unwrap());
            assert_eq!(rule.reason, TextKey::new("example.rule.caches.reason").unwrap());
        }

        #[test]
        fn the_optional_parts_may_be_left_out() {
            let rules = parse(&minimal_with("", "")).unwrap();
            assert_eq!(rules.rules().len(), 1);
            let rule = &rules.rules()[0];

            assert_eq!(rule.removal, RemovalSpec::Trash);
            assert_eq!(rule.interest.pattern(), &Pattern::FolderName("Caches".into()));
            assert_eq!((rule.name.as_ref(), rule.version.as_ref()), (None, None));
            assert_eq!((rule.min_age_days, rule.sibling.as_ref()), (None, None));
        }

        #[test]
        fn every_kind_of_look_compiles_to_its_pattern() {
            for (look, pattern) in [
                ("marker = \"CACHEDIR.TAG\"", Pattern::MarkerFile("CACHEDIR.TAG".into())),
                ("extension = \"dmg\"", Pattern::Extension("dmg".into())),
                ("under = { folder = \"logs\", path = \"*\" }", Pattern::Under(KnownFolder::Logs, "*".into())),
            ] {
                let rules = parse(&minimal_with("folder-name = \"Caches\"", look)).unwrap();

                assert_eq!(rules.rules()[0].interest.pattern(), &pattern);
            }
        }

        #[test]
        fn a_file_without_rules_is_an_empty_set() {
            assert_eq!(parse("").unwrap().rules(), []);
        }
    }
}
