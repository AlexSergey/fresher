use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

use fresher_core::{
    CategoryDescriptor, CategoryText, DangerLevel, DescriptorError, Dictionary, DictionaryError, Module,
    ModuleDescriptor, ModuleOutcome, SettingKey, Text,
};

use crate::ModuleRun;

/// The suffixes of a plural text: `<key>.one` and `<key>.other` stand for `<key>`.
const PLURAL_SUFFIXES: [&str; 2] = [".one", ".other"];

/// The last segment of the switch the settings schema adds for every module: `<module>.enabled`.
const ENABLED: &str = "enabled";

/// Whether a module's categories must be documented in the catalogue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Catalogue {
    /// Every category `<module>.<name>` has `<docs dir>/<module>-<name>.md` naming the same danger level.
    Required(PathBuf),
    /// A dev-only module (`demo`): no catalogue files.
    DevOnly,
}

/// One way a module breaks its contract.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ContractProblem {
    #[error("the descriptor is not valid: {0}")]
    Descriptor(DescriptorError),
    #[error("the dictionary is not valid: {0}")]
    Dictionary(DictionaryError),
    #[error("`{key}` has no text in the dictionary")]
    MissingText { key: String },
    #[error("category `{category}` has no catalogue file {}", .path.display())]
    MissingCatalogueFile { category: String, path: PathBuf },
    #[error("the catalogue file of `{category}` names the danger level {found:?}, the category has {expected:?}")]
    CatalogueDanger { category: String, expected: DangerLevel, found: Option<String> },
    #[error("`{key}` is in the dictionary but no descriptor key and no finding of the runs uses it")]
    UnusedText { key: String },
    #[error("the text of `{key}` names the placeholder `{arg}`, which a finding does not give")]
    MissingArg { key: String, arg: String },
}

/// Every way `module` breaks the module contract: its descriptor validates; every category has its five texts and
/// every setting (`<module>.enabled` included) its label and description; every key of its dictionary is in its
/// namespace; with [`Catalogue::Required`], every category has its catalogue file with the same danger level.
pub fn check_module_contract(module: &dyn Module, catalogue: &Catalogue) -> Vec<ContractProblem> {
    let descriptor = module.descriptor();
    let mut problems = Vec::new();
    let revalidated = ModuleDescriptor::new(
        descriptor.id(),
        descriptor.categories().to_vec(),
        descriptor.settings().to_vec(),
        descriptor.locales(),
    );
    if let Err(error) = revalidated {
        problems.push(ContractProblem::Descriptor(error));
    }
    match Dictionary::parse(descriptor.id().as_str(), descriptor.locales().en) {
        Ok(dictionary) => {
            let missing = descriptor_keys(descriptor).into_iter().filter(|key| texts_of(&dictionary, key).is_empty());
            problems.extend(missing.map(|key| ContractProblem::MissingText { key }));
        }
        Err(error) => problems.push(ContractProblem::Dictionary(error)),
    }
    if let Catalogue::Required(docs) = catalogue {
        problems.extend(descriptor.categories().iter().filter_map(|category| catalogue_problem(docs, category)));
    }
    problems
}

/// Fails the test with every problem [`check_module_contract`] finds.
pub fn assert_module_contract(module: &dyn Module, catalogue: &Catalogue) {
    let problems = check_module_contract(module, catalogue);
    assert!(problems.is_empty(), "module `{}` breaks its contract:{}", module.descriptor().id(), listed(&problems));
}

/// Every way the texts of `module` and the findings of `runs` disagree: every key of the dictionary is a descriptor
/// key or appears in a finding (or a failure) of the runs; every key a finding uses has a text, and the finding gives
/// every placeholder that text names. A plural text (`<key>.one`, `<key>.other`) stands for `<key>`.
pub fn check_texts_used(module: &dyn Module, runs: &[ModuleRun]) -> Vec<ContractProblem> {
    let descriptor = module.descriptor();
    let dictionary = match Dictionary::parse(descriptor.id().as_str(), descriptor.locales().en) {
        Ok(dictionary) => dictionary,
        Err(error) => return vec![ContractProblem::Dictionary(error)],
    };
    let texts: Vec<&Text> = runs.iter().flat_map(run_texts).collect();
    let mut used: BTreeSet<String> = descriptor_keys(descriptor).into_iter().collect();
    used.extend(texts.iter().map(|text| text.key().as_str().to_owned()));
    let mut problems: Vec<ContractProblem> = dictionary
        .keys()
        .map(fresher_core::TextKey::as_str)
        .filter(|key| !used.contains(*key) && !used.contains(singular(key)))
        .map(|key| ContractProblem::UnusedText { key: key.into() })
        .collect();
    for problem in texts.into_iter().flat_map(|text| text_problems(&dictionary, text)) {
        if !problems.contains(&problem) {
            problems.push(problem);
        }
    }
    problems
}

/// Fails the test with every problem [`check_texts_used`] finds.
pub fn assert_texts_used(module: &dyn Module, runs: &[ModuleRun]) {
    let problems = check_texts_used(module, runs);
    assert!(problems.is_empty(), "the texts of module `{}` are wrong:{}", module.descriptor().id(), listed(&problems));
}

/// The keys a descriptor implies: the five texts of every category, the label and description of every setting,
/// the `<module>.enabled` switch the settings schema adds included.
fn descriptor_keys(descriptor: &ModuleDescriptor) -> Vec<String> {
    let categories = descriptor
        .categories()
        .iter()
        .flat_map(|category| CategoryText::ALL.map(|text| category.text_key(text).as_str().to_owned()));
    let enabled = SettingKey::new(format!("{}.{ENABLED}", descriptor.id())).ok();
    let settings = enabled.into_iter().chain(descriptor.settings().iter().map(|setting| setting.key().clone()));
    let settings = settings.flat_map(|key| [key.label_key(), key.description_key()].map(|key| key.as_str().to_owned()));
    categories.chain(settings).collect()
}

/// `key` and its plural forms.
fn forms(key: &str) -> impl Iterator<Item = String> {
    std::iter::once(key.to_owned()).chain(PLURAL_SUFFIXES.map(|suffix| format!("{key}{suffix}")))
}

/// The texts of `key`: the text itself, or its plural forms.
fn texts_of<'d>(dictionary: &'d Dictionary, key: &str) -> Vec<&'d str> {
    forms(key).filter_map(|form| dictionary.get(&form)).collect()
}

/// `key` without a plural suffix.
fn singular(key: &str) -> &str {
    PLURAL_SUFFIXES.iter().find_map(|suffix| key.strip_suffix(suffix)).unwrap_or(key)
}

/// The texts a run shows: every finding's title and reason, and the reason of a failure.
fn run_texts(run: &ModuleRun) -> impl Iterator<Item = &Text> {
    let failure = match &run.outcome {
        ModuleOutcome::Failed { reason } => Some(reason),
        ModuleOutcome::Completed | ModuleOutcome::Cancelled => None,
    };
    run.findings.iter().flat_map(|finding| [finding.title(), finding.reason()]).chain(failure)
}

/// A text with no entry in the dictionary, or without an arg its text names.
fn text_problems(dictionary: &Dictionary, text: &Text) -> Vec<ContractProblem> {
    let key = text.key().as_str();
    if texts_of(dictionary, key).is_empty() {
        return vec![ContractProblem::MissingText { key: key.into() }];
    }
    let mut named = BTreeSet::new();
    for form in forms(key) {
        named.extend(dictionary.placeholders(&form).unwrap_or_default().into_iter().map(str::to_owned));
    }
    named
        .into_iter()
        .filter(|arg| !text.args().contains_key(arg))
        .map(|arg| ContractProblem::MissingArg { key: key.into(), arg })
        .collect()
}

/// What is wrong with the catalogue file of `category` in `docs`, if anything.
fn catalogue_problem(docs: &std::path::Path, category: &CategoryDescriptor) -> Option<ContractProblem> {
    let id = category.id();
    let file_name = format!("{}.md", id.as_str().replacen('.', "-", 1));
    let path = docs.join(file_name);
    let Ok(text) = fs::read_to_string(&path) else {
        return Some(ContractProblem::MissingCatalogueFile { category: id.as_str().into(), path });
    };
    let found = text.lines().find_map(|line| {
        let (_, after) = line.split_once("**Danger level:**")?;
        after.split_whitespace().next().map(str::to_owned)
    });
    (found.as_deref() != Some(danger_name(category.danger()))).then(|| ContractProblem::CatalogueDanger {
        category: id.as_str().into(),
        expected: category.danger(),
        found,
    })
}

/// The danger level as the catalogue writes it.
const fn danger_name(danger: DangerLevel) -> &'static str {
    match danger {
        DangerLevel::Safe => "Safe",
        DangerLevel::Low => "Low",
        DangerLevel::Medium => "Medium",
        DangerLevel::High => "High",
    }
}

/// The problems, one per line.
fn listed(problems: &[ContractProblem]) -> String {
    problems.iter().fold(String::new(), |mut out, problem| {
        let _ = write!(out, "\n  - {problem}");
        out
    })
}

#[cfg(test)]
mod tests {
    use crate::run_module::sample::{CacheModule, TREE};
    use crate::{FixtureTree, ModuleRun, Ports, run_module};

    /// The catalogue file of `sample.caches` with `danger` on its danger line.
    fn catalogue_file(danger: &str) -> String {
        format!("# Caches\n\n**Danger level:** {danger} · **Rule:** `caches` · **Since:** 0.1.0\n")
    }

    /// A docs folder holding `sample-caches.md` with `content`, or nothing.
    fn docs(content: Option<&str>) -> FixtureTree {
        let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"categories\"\n").unwrap();
        if let Some(content) = content {
            std::fs::write(tree.root().join("categories/sample-caches.md"), content).unwrap();
        }
        tree
    }

    fn runs(module: &CacheModule) -> Vec<ModuleRun> {
        let tree = FixtureTree::from_toml_str(TREE).unwrap();
        vec![run_module(module, &tree, &Ports::at(0)).unwrap()]
    }

    mod negative {
        use fresher_core::{
            DangerLevel, DescriptorError, DictionaryError, SettingDescriptor, SettingKey, SettingKind, SettingValue,
        };

        use super::{catalogue_file, docs, runs};
        use crate::run_module::sample::CacheModule;
        use crate::{Catalogue, ContractProblem, check_module_contract, check_texts_used};

        #[test]
        fn the_contract_fails_for_a_category_without_its_text() {
            let en = r#"{ "sample.caches.name": "Caches", "sample.caches.what": "Files.", "sample.caches.made-by": "Apps." }"#;

            let problems = check_module_contract(&CacheModule::with_dictionary(en), &Catalogue::DevOnly);

            assert_eq!(
                problems,
                [
                    ContractProblem::MissingText { key: "sample.caches.if-removed".into() },
                    ContractProblem::MissingText { key: "sample.caches.comes-back".into() },
                    ContractProblem::MissingText { key: "sample.enabled.label".into() },
                    ContractProblem::MissingText { key: "sample.enabled.description".into() },
                ]
            );
        }

        #[test]
        fn the_contract_fails_without_the_texts_of_the_enabled_switch() {
            let en = r#"{
                "sample.caches.name": "Caches", "sample.caches.what": "Files.", "sample.caches.made-by": "Apps.",
                "sample.caches.if-removed": "Rebuilt.", "sample.caches.comes-back": "Yes.",
                "sample.enabled.label": "Sample"
            }"#;

            let problems = check_module_contract(&CacheModule::with_dictionary(en), &Catalogue::DevOnly);

            assert_eq!(problems, [ContractProblem::MissingText { key: "sample.enabled.description".into() }]);
        }

        #[test]
        fn the_contract_fails_for_a_setting_without_its_label() {
            let key = SettingKey::new("sample.deep").unwrap();
            let setting = SettingDescriptor::new(key, SettingKind::Switch, SettingValue::Switch(false)).unwrap();
            let module = CacheModule::with_settings(crate::run_module::sample::EN, vec![setting]);

            let problems = check_module_contract(&module, &Catalogue::DevOnly);

            assert_eq!(
                problems,
                [
                    ContractProblem::MissingText { key: "sample.deep.label".into() },
                    ContractProblem::MissingText { key: "sample.deep.description".into() },
                ]
            );
        }

        #[test]
        fn the_contract_fails_for_a_key_outside_the_namespace() {
            let en = r#"{ "docker.images.name": "Images" }"#;

            let problems = check_module_contract(&CacheModule::with_dictionary(en), &Catalogue::DevOnly);

            let outside = DictionaryError::OutsideNamespace { key: "docker.images.name".into() };
            assert_eq!(problems, [ContractProblem::Dictionary(outside.clone())]);
            assert_eq!(
                check_texts_used(&CacheModule::with_dictionary(en), &[]),
                [ContractProblem::Dictionary(outside)]
            );
        }

        #[test]
        fn the_contract_fails_for_a_category_without_its_catalogue_file() {
            let docs = docs(None);
            let catalogue = Catalogue::Required(docs.root().join("categories"));

            let problems = check_module_contract(&CacheModule::new(), &catalogue);

            let path = docs.root().join("categories/sample-caches.md");
            assert_eq!(problems, [ContractProblem::MissingCatalogueFile { category: "sample.caches".into(), path }]);
        }

        #[test]
        fn the_contract_fails_when_the_catalogue_names_another_danger_level() {
            for (content, found) in [(catalogue_file("Medium"), Some("Medium")), ("# Caches\n".into(), None)] {
                let docs = docs(Some(&content));
                let catalogue = Catalogue::Required(docs.root().join("categories"));

                let problems = check_module_contract(&CacheModule::new(), &catalogue);

                let expected = ContractProblem::CatalogueDanger {
                    category: "sample.caches".into(),
                    expected: DangerLevel::Low,
                    found: found.map(String::from),
                };
                assert_eq!(problems, [expected]);
            }
        }

        #[test]
        fn an_unused_key_fails() {
            let en = r#"{
                "sample.caches.name": "Caches", "sample.caches.what": "Files.", "sample.caches.made-by": "Apps.",
                "sample.caches.if-removed": "Rebuilt.", "sample.caches.comes-back": "Yes.",
                "sample.item.title": "{name}", "sample.item.reason": "Named Caches.", "sample.item.old": "Unused."
            }"#;
            let module = CacheModule::with_dictionary(en);

            let problems = check_texts_used(&module, &runs(&module));

            assert_eq!(problems, [ContractProblem::UnusedText { key: "sample.item.old".into() }]);
        }

        #[test]
        fn a_missing_placeholder_arg_fails() {
            let en = r#"{
                "sample.caches.name": "Caches", "sample.caches.what": "Files.", "sample.caches.made-by": "Apps.",
                "sample.caches.if-removed": "Rebuilt.", "sample.caches.comes-back": "Yes.",
                "sample.item.title": "{name}", "sample.item.reason": "Named Caches, {size}."
            }"#;
            let module = CacheModule::with_dictionary(en);

            let problems = check_texts_used(&module, &runs(&module));

            assert_eq!(
                problems,
                [ContractProblem::MissingArg { key: "sample.item.reason".into(), arg: "size".into() }]
            );
        }

        #[test]
        fn a_finding_key_without_a_text_fails() {
            let module = CacheModule::failing();

            let problems = check_texts_used(&module, &runs(&module));

            assert_eq!(problems, [ContractProblem::MissingText { key: "sample.error.broken".into() }]);
        }

        #[test]
        fn problems_read_plainly() {
            let problem =
                ContractProblem::Descriptor(DescriptorError::DuplicateCategory { id: "sample.caches".into() });

            assert_eq!(problem.to_string(), "the descriptor is not valid: category `sample.caches` is listed twice");
        }
    }

    mod positive {
        use super::{catalogue_file, docs, runs};
        use crate::run_module::sample::CacheModule;
        use crate::{Catalogue, assert_module_contract, assert_texts_used, check_texts_used};

        #[test]
        fn a_complete_module_keeps_its_contract() {
            let docs = docs(Some(&catalogue_file("Low")));
            let module = CacheModule::new();

            assert_module_contract(&module, &Catalogue::Required(docs.root().join("categories")));
            assert_module_contract(&module, &Catalogue::DevOnly);
            assert_texts_used(&module, &runs(&module));
        }

        #[test]
        fn a_plural_text_stands_for_its_key() {
            let en = r#"{
                "sample.caches.name": "Caches", "sample.caches.what": "Files.", "sample.caches.made-by": "Apps.",
                "sample.caches.if-removed": "Rebuilt.", "sample.caches.comes-back": "Yes.",
                "sample.item.title.one": "{name}", "sample.item.title.other": "{name}s", "sample.item.reason": "Caches."
            }"#;
            let module = CacheModule::with_dictionary(en);

            assert_eq!(check_texts_used(&module, &runs(&module)), []);
        }
    }
}
