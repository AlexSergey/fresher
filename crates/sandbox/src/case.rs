use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

use fresher_core::{CategoryDescriptor, CategoryId, DangerLevel, IdError, ModuleId, SkipCode};
use fresher_testing::TreeDescription;
use serde::Deserialize;

use crate::CaseError;

/// The cases of one feature (`fixtures/sandboxes/<feature>.toml`), checked when parsed: names unique, negatives
/// first, every path plain and inside the sandbox, no path written by two cases, every positive's item its own.
#[derive(Clone, Debug)]
pub struct CaseFile {
    feature: String,
    module: Option<ModuleId>,
    cases: Vec<Case>,
}

/// One case: the files it writes and, for a positive case, what the app must do with them.
#[derive(Clone, Debug)]
pub struct Case {
    name: String,
    kind: CaseKind,
    tree: TreeDescription,
    expect: Option<Expectation>,
}

/// A negative case is never offered and stays untouched; a positive case is offered and cleaned as it expects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseKind {
    Negative,
    Positive,
}

/// What a positive case expects: the finding's item, its category and danger, and what a cleanup does with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expectation {
    item: PathBuf,
    category: CategoryId,
    danger: DangerLevel,
    after: AfterCleanup,
}

/// What a cleanup of the item does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterCleanup {
    /// The item is moved into the sandbox's Trash.
    InTrash,
    /// The folder is kept and its contents go to the Trash.
    Emptied,
    /// The item is skipped for this reason.
    Skipped(SkipCode),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCaseFile {
    feature: String,
    module: Option<String>,
    #[serde(default, rename = "case")]
    cases: Vec<RawCase>,
}

// Its files are every key but these three, read as a `TreeDescription`, which refuses an unknown one.
#[derive(Deserialize)]
struct RawCase {
    name: String,
    kind: CaseKind,
    expect: Option<RawExpectation>,
    #[serde(flatten)]
    files: toml::Table,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawExpectation {
    item: PathBuf,
    category: String,
    danger: DangerLevel,
    after_cleanup: RawAfterCleanup,
    reason: Option<SkipCode>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RawAfterCleanup {
    InTrash,
    Emptied,
    Skipped,
}

impl CaseFile {
    /// Parses and checks a case file; the categories are checked against a module's descriptor by [`Self::check`].
    pub fn parse(text: &str) -> Result<Self, CaseError> {
        let raw: RawCaseFile = toml::from_str(text).map_err(|source| CaseError::Parse { source: Box::new(source) })?;
        if let Some(module) = &raw.module {
            ModuleId::check(module).map_err(|source| CaseError::Module { module: module.clone(), source })?;
        }
        let mut names = HashSet::new();
        let mut written: HashMap<PathBuf, String> = HashMap::new();
        let mut seen_positive = false;
        let mut cases = Vec::with_capacity(raw.cases.len());
        for raw_case in raw.cases {
            let (case, pending) = Case::from_raw(raw_case)?;
            if !names.insert(case.name.clone()) {
                return Err(CaseError::DuplicateName { case: case.name });
            }
            match case.kind {
                CaseKind::Positive => seen_positive = true,
                CaseKind::Negative if seen_positive => {
                    return Err(CaseError::PositiveBeforeNegative { case: case.name });
                }
                CaseKind::Negative => {}
            }
            for path in case.tree.paths() {
                if let Some(other) = written.insert(path.to_path_buf(), case.name.clone()) {
                    return Err(CaseError::PathWrittenTwice { case: case.name, other, path: path.to_path_buf() });
                }
            }
            cases.push((case, pending));
        }
        merge_trees(cases.iter().map(|(case, _)| case))?;
        // Every check passed: only now are the ids kept, so a refused file keeps nothing.
        let module = raw
            .module
            .map(|module| ModuleId::new(leak(module.clone())).map_err(|source| CaseError::Module { module, source }))
            .transpose()?;
        let cases = cases
            .into_iter()
            .map(|(mut case, pending)| {
                case.expect = pending.map(|pending| pending.into_expectation(&case.name)).transpose()?;
                Ok(case)
            })
            .collect::<Result<_, CaseError>>()?;
        Ok(Self { feature: raw.feature, module, cases })
    }

    /// Checks every expectation against the categories of the module: the category exists and the danger is not
    /// below its level.
    pub fn check(&self, categories: &[CategoryDescriptor]) -> Result<(), CaseError> {
        for case in &self.cases {
            let Some(expect) = &case.expect else { continue };
            let descriptor =
                categories.iter().find(|descriptor| descriptor.id() == expect.category).ok_or_else(|| {
                    CaseError::UnknownCategory { case: case.name.clone(), category: expect.category.to_string() }
                })?;
            if expect.danger < descriptor.danger() {
                return Err(CaseError::DangerBelowCategory {
                    case: case.name.clone(),
                    danger: expect.danger,
                    floor: descriptor.danger(),
                });
            }
        }
        Ok(())
    }

    /// Every case's files as one tree.
    pub fn tree(&self) -> Result<TreeDescription, CaseError> {
        merge_trees(self.cases.iter())
    }

    /// The feature: a module id or a part of the core (`cleanup-guards`).
    pub fn feature(&self) -> &str {
        &self.feature
    }

    /// The module the autotest registers; `None` registers every module of the app.
    pub const fn module(&self) -> Option<ModuleId> {
        self.module
    }

    /// The cases, negatives first.
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }
}

impl Case {
    /// The case without its expectation, which is kept apart until the whole file is accepted.
    fn from_raw(raw: RawCase) -> Result<(Self, Option<PendingExpectation>), CaseError> {
        let name = raw.name;
        let tree: TreeDescription = toml::Value::Table(raw.files)
            .try_into()
            .map_err(|source| CaseError::Files { case: name.clone(), source: Box::new(source) })?;
        // Merging into an empty tree runs the tree's own checks: no path written twice, ages, the fill cap.
        let tree =
            TreeDescription::default().merge(tree).map_err(|source| CaseError::Tree { case: name.clone(), source })?;
        let outside = tree.paths().chain(tree.markers()).chain(tree.hardlink_targets()).find(|path| !is_plain(path));
        if let Some(path) = outside {
            return Err(CaseError::PathOutsideSandbox { case: name, path: path.to_path_buf() });
        }
        let expect = match (raw.kind, raw.expect) {
            (CaseKind::Negative, Some(_)) => return Err(CaseError::NegativeWithExpect { case: name }),
            (CaseKind::Positive, None) => return Err(CaseError::PositiveWithoutExpect { case: name }),
            (CaseKind::Negative, None) => None,
            (CaseKind::Positive, Some(raw)) => Some(PendingExpectation::from_raw(&name, &tree, raw)?),
        };
        Ok((Self { name, kind: raw.kind, tree, expect: None }, expect))
    }

    /// The case's name, unique in its file: a failure names the case by it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the case is negative (never offered) or positive (offered and cleaned as expected).
    pub const fn kind(&self) -> CaseKind {
        self.kind
    }

    /// The files the case writes, relative to the sandbox folder.
    pub const fn tree(&self) -> &TreeDescription {
        &self.tree
    }

    /// The expectation of a positive case; a negative case has none.
    pub const fn expect(&self) -> Option<&Expectation> {
        self.expect.as_ref()
    }
}

/// A checked expectation whose category id is not kept yet.
struct PendingExpectation {
    item: PathBuf,
    category: String,
    danger: DangerLevel,
    after: AfterCleanup,
}

impl PendingExpectation {
    fn from_raw(case: &str, tree: &TreeDescription, raw: RawExpectation) -> Result<Self, CaseError> {
        let after = match (raw.after_cleanup, raw.reason) {
            (RawAfterCleanup::InTrash, None) => AfterCleanup::InTrash,
            (RawAfterCleanup::Emptied, None) => AfterCleanup::Emptied,
            (RawAfterCleanup::Skipped, Some(reason)) => AfterCleanup::Skipped(reason),
            _ => return Err(CaseError::SkipReason { case: case.into() }),
        };
        let own = tree.paths().chain(tree.markers()).any(|path| path.ancestors().any(|folder| folder == raw.item));
        if !is_plain(&raw.item) || !own {
            return Err(CaseError::ItemNotInCase { case: case.into(), item: raw.item });
        }
        check_category(&raw.category).map_err(|source| CaseError::CategoryId {
            case: case.into(),
            category: raw.category.clone(),
            source,
        })?;
        Ok(Self { item: raw.item, category: raw.category, danger: raw.danger, after })
    }

    fn into_expectation(self, case: &str) -> Result<Expectation, CaseError> {
        let category = category_id(leak(self.category.clone())).map_err(|source| CaseError::CategoryId {
            case: case.into(),
            category: self.category,
            source,
        })?;
        Ok(Expectation { item: self.item, category, danger: self.danger, after: self.after })
    }
}

impl Expectation {
    /// The finding's item, relative to the sandbox folder.
    pub fn item(&self) -> &Path {
        &self.item
    }

    /// The category the finding belongs to.
    pub const fn category(&self) -> CategoryId {
        self.category
    }

    /// The finding's danger level: the category's, or a raised one.
    pub const fn danger(&self) -> DangerLevel {
        self.danger
    }

    /// What a cleanup of the item does.
    pub const fn after(&self) -> AfterCleanup {
        self.after
    }
}

/// `true` for a non-empty relative path of plain names only: no root, `.`, `..` or prefix.
fn is_plain(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|component| matches!(component, Component::Normal(_)))
}

/// The module part of a category id `<module>.<name>`.
fn module_of(id: &str) -> &str {
    id.split_once('.').map_or(id, |(module, _)| module)
}

/// Checks a category id `<module>.<name>` read from a case file, without keeping it.
fn check_category(id: &str) -> Result<(), IdError> {
    ModuleId::check(module_of(id))?;
    CategoryId::check(module_of(id), id)
}

/// The category id `<module>.<name>` of a checked, kept id.
fn category_id(id: &'static str) -> Result<CategoryId, IdError> {
    CategoryId::new(ModuleId::new(module_of(id))?, id)
}

/// The cases' files as one tree.
fn merge_trees<'a>(mut cases: impl Iterator<Item = &'a Case>) -> Result<TreeDescription, CaseError> {
    cases.try_fold(TreeDescription::default(), |tree, case| {
        tree.merge(case.tree.clone()).map_err(|source| CaseError::Sandbox { source })
    })
}

/// The core's ids hold `&'static str`; an id read from a case file lives for the rest of the process. Only the ids of
/// an accepted file are kept, and a dev-only tool reads a handful of them, so the leak is bounded.
fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use fresher_core::{CategoryDescriptor, CategoryId, DangerLevel, ModuleId};

    fn trial_categories() -> Vec<CategoryDescriptor> {
        let trial = ModuleId::new("trial").unwrap();
        vec![CategoryDescriptor::new(CategoryId::new(trial, "trial.caches").unwrap(), DangerLevel::Low)]
    }

    const NEGATIVE: &str = r#"
[[case]]
name = "a trial folder with no marker next to it is never offered"
kind = "negative"
[[case.file]]
path = "home/bare/fresher-trial-cache/blob.bin"
size = 1048576
"#;

    fn positive(name: &str, path: &str, item: &str, danger: &str) -> String {
        format!(
            r#"
[[case]]
name = "{name}"
kind = "positive"
[[case.marker]]
path = "home/work"
[[case.file]]
path = "{path}"
size = 52428800
fill = "data"
[case.expect]
item = "{item}"
category = "trial.caches"
danger = "{danger}"
after-cleanup = "in-trash"
"#
        )
    }

    fn file(cases: &[&str]) -> String {
        format!("feature = \"trial\"\nmodule = \"trial\"\n{}", cases.concat())
    }

    fn cache(danger: &str) -> String {
        positive(
            "a trial cache goes to the Trash",
            "home/work/fresher-trial-cache/blob.bin",
            "home/work/fresher-trial-cache",
            danger,
        )
    }

    mod negative {
        use super::{NEGATIVE, cache, file, positive, trial_categories};
        use crate::{CaseError, CaseFile};

        fn refused(text: &str) -> CaseError {
            CaseFile::parse(text).unwrap_err()
        }

        #[test]
        fn a_duplicate_name_is_refused() {
            let error = refused(&file(&[NEGATIVE, NEGATIVE.replace("bare", "other").as_str()]));

            assert!(matches!(error, CaseError::DuplicateName { case } if case.starts_with("a trial folder")));
        }

        #[test]
        fn a_positive_before_a_negative_is_refused() {
            let error = refused(&file(&[&cache("low"), NEGATIVE]));

            assert!(matches!(error, CaseError::PositiveBeforeNegative { case } if case.starts_with("a trial folder")));
        }

        #[test]
        fn a_positive_without_expect_is_refused() {
            let text = file(&[&NEGATIVE.replace("negative", "positive")]);

            assert!(matches!(refused(&text), CaseError::PositiveWithoutExpect { .. }));
        }

        #[test]
        fn a_negative_with_expect_is_refused() {
            let text = file(&[&cache("low").replace("\"positive\"", "\"negative\"")]);

            assert!(matches!(refused(&text), CaseError::NegativeWithExpect { case } if case.contains("Trash")));
        }

        #[test]
        fn a_skip_reason_without_skipped_is_refused() {
            let text = file(&[&cache("low").replace("after-cleanup", "reason = \"in-use\"\nafter-cleanup")]);

            assert!(matches!(refused(&text), CaseError::SkipReason { .. }));
        }

        #[test]
        fn skipped_without_a_reason_is_refused() {
            let text = file(&[&cache("low").replace("\"in-trash\"", "\"skipped\"")]);

            assert!(matches!(refused(&text), CaseError::SkipReason { .. }));
        }

        #[test]
        fn an_item_that_is_not_the_cases_is_refused() {
            for item in ["home/bare/fresher-trial-cache", "home/work/fresher-trial-cache/other", "", "/home/work"] {
                let text = file(&[&positive("p", "home/work/fresher-trial-cache/blob.bin", item, "low")]);

                assert!(matches!(refused(&text), CaseError::ItemNotInCase { case, .. } if case == "p"), "{item}");
            }
        }

        #[test]
        fn an_unknown_category_is_refused() {
            let parsed = CaseFile::parse(&file(&[&cache("low").replace("trial.caches", "trial.logs")])).unwrap();

            let error = parsed.check(&trial_categories()).unwrap_err();

            assert!(matches!(error, CaseError::UnknownCategory { category, .. } if category == "trial.logs"));
        }

        #[test]
        fn a_malformed_category_is_refused() {
            let text = file(&[&cache("low").replace("trial.caches", "Trial")]);

            assert!(matches!(refused(&text), CaseError::CategoryId { .. }));
        }

        #[test]
        fn a_danger_below_the_categorys_is_refused() {
            let parsed = CaseFile::parse(&file(&[&cache("safe")])).unwrap();

            let error = parsed.check(&trial_categories()).unwrap_err();

            assert!(matches!(error, CaseError::DangerBelowCategory { case, .. } if case.contains("Trash")));
        }

        #[test]
        fn two_cases_writing_one_path_are_refused() {
            let second = NEGATIVE.replace("a trial folder", "another folder");

            let error = refused(&file(&[NEGATIVE, &second]));

            assert!(matches!(error, CaseError::PathWrittenTwice { case, other, .. }
                if case.starts_with("another") && other.starts_with("a trial folder")));
        }

        #[test]
        fn an_unknown_field_is_refused() {
            let in_case = file(&[&NEGATIVE.replace("kind =", "colour = \"red\"\nkind =")]);
            let in_file = NEGATIVE.replace("size", "colour = 1\nsize");
            let in_expect = file(&[&cache("low").replace("danger =", "colour = \"red\"\ndanger =")]);
            let at_top = format!("colour = 1\n{}", file(&[NEGATIVE]));

            assert!(matches!(refused(&in_case), CaseError::Files { .. }));
            assert!(matches!(refused(&file(&[&in_file])), CaseError::Files { .. }));
            assert!(matches!(refused(&in_expect), CaseError::Parse { .. }));
            assert!(matches!(refused(&at_top), CaseError::Parse { .. }));
        }

        #[test]
        fn a_path_outside_the_sandbox_is_refused() {
            for path in ["../outside", "/etc/hosts", "home/../../outside", "./home/a", ""] {
                let text = file(&[&NEGATIVE.replace("home/bare/fresher-trial-cache/blob.bin", path)]);

                assert!(matches!(refused(&text), CaseError::PathOutsideSandbox { .. }), "{path:?}");
            }
            let marker = file(&[&cache("low").replace("path = \"home/work\"", "path = \"/tmp\"")]);
            assert!(matches!(refused(&marker), CaseError::PathOutsideSandbox { .. }));
            let hardlink = file(&[&format!(
                "{NEGATIVE}[[case.hardlink]]\npath = \"home/bare/second\"\ntarget = \"/etc/hosts\"\n"
            )]);
            assert!(
                matches!(refused(&hardlink), CaseError::PathOutsideSandbox { path, .. } if path.ends_with("hosts"))
            );
        }

        #[test]
        fn a_tree_error_names_its_case() {
            let text = file(&[&NEGATIVE.replace("size", "age = \"3w\"\nsize")]);

            assert!(matches!(refused(&text), CaseError::Tree { case, .. } if case.starts_with("a trial folder")));
        }
    }

    mod positive {
        use std::path::Path;
        use std::time::{Duration, SystemTime};

        use fresher_core::{DangerLevel, ModuleId, SkipCode};
        use fresher_testing::FIXTURE_NOW;

        use super::{NEGATIVE, cache, file, trial_categories};
        use crate::{AfterCleanup, Case, CaseFile, CaseKind};

        #[test]
        fn a_case_file_parses_with_negatives_then_positives() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE, &cache("low")])).unwrap();

            parsed.check(&trial_categories()).unwrap();
            assert_eq!(parsed.feature(), "trial");
            assert_eq!(parsed.module().map(ModuleId::as_str), Some("trial"));
            let kinds: Vec<_> = parsed.cases().iter().map(Case::kind).collect();
            assert_eq!(kinds, [CaseKind::Negative, CaseKind::Positive]);
            assert!(parsed.cases()[0].expect().is_none());
            let expect = parsed.cases()[1].expect().unwrap();
            assert_eq!(expect.item(), Path::new("home/work/fresher-trial-cache"));
            assert_eq!(expect.category().as_str(), "trial.caches");
            assert_eq!(expect.danger(), DangerLevel::Low);
            assert_eq!(expect.after(), AfterCleanup::InTrash);
        }

        #[test]
        fn a_core_feature_has_no_module_and_a_skip_has_its_reason() {
            let text = file(&[&cache("low").replace("\"in-trash\"", "\"skipped\"\nreason = \"in-use\"")])
                .replace("module = \"trial\"\n", "");

            let parsed = CaseFile::parse(&text).unwrap();

            assert_eq!(parsed.module(), None);
            assert_eq!(parsed.cases()[0].expect().unwrap().after(), AfterCleanup::Skipped(SkipCode::InUse));
        }

        // A negative case may hold a link out of the sandbox on purpose: the scan must not follow it.
        #[test]
        fn a_symlink_may_point_out_of_the_sandbox() {
            let link = format!("{NEGATIVE}[[case.symlink]]\npath = \"home/bare/out\"\ntarget = \"/etc\"\n");

            assert!(CaseFile::parse(&file(&[&link])).is_ok());
        }

        #[test]
        fn a_raised_danger_is_accepted() {
            let parsed = CaseFile::parse(&file(&[&cache("high")])).unwrap();

            parsed.check(&trial_categories()).unwrap();
            assert_eq!(parsed.cases()[0].expect().unwrap().danger(), DangerLevel::High);
        }

        #[test]
        fn the_cases_merge_into_one_tree() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE, &cache("low")])).unwrap();
            let folder = tempfile::tempdir().unwrap();

            let now = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW);
            parsed.tree().unwrap().build_at(folder.path(), now).unwrap();

            assert!(folder.path().join("home/bare/fresher-trial-cache/blob.bin").is_file());
            assert!(folder.path().join("home/work/fresher-trial-cache/blob.bin").is_file());
            assert!(folder.path().join("home/work").join(fresher_core::SANDBOX_MARKER).is_file());
        }
    }
}
