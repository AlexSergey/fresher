use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use fresher_core::{
    Access, BundleId, Capability, CategoryDescriptor, CategoryId, DangerLevel, GuardedFolder, HighLevel, IdError,
    ModuleId, OWN_FILE_ID, RunningApp, SettingKey, SkipCode, SystemVersion, TextKey, is_kebab_case, own_file,
};
use fresher_testing::{AppBundle, TreeDescription, time_at_age};
use serde::{Deserialize, Serialize};

use crate::demo::RawDemo;
use crate::largest::{LargestExpectation, RawLargest};
use crate::uninstall::{RawUninstall, UninstallExpectation};
use crate::world::{StatedTool, ToolProblem};
use crate::{CaseError, Demo};

/// The cases of one feature (`fixtures/sandboxes/<feature>.toml`), checked when parsed: names unique, negatives
/// first, every path plain and inside the sandbox, no path written by two cases, every positive's item its own. A
/// demo world (`fixtures/demos/<world>.toml`) has a `[demo]` header, no module and scenery cases only.
#[derive(Clone, Debug)]
pub struct CaseFile {
    feature: String,
    module: Option<ModuleId>,
    demo: Option<Demo>,
    /// `high = "withheld"`: the file runs with High withheld, as the app does; offered when absent.
    high: HighLevel,
    /// The file's `[settings]`: the values of the modules' settings, shared by every case of the file.
    settings: BTreeMap<String, toml::Value>,
    cases: Vec<Case>,
}

/// The folders an app of a case may be in, relative to the sandbox: directly or one folder below, as on a Mac.
const APPS_FOLDERS: [&str; 3] = ["Applications", "System/Applications", "home/Applications"];

/// The apps folder in the fake home: a case writes nothing there but apps.
const HOME_APPS_FOLDER: &str = "home/Applications";

/// One case: the files it writes and, for a positive case, what the app must do with them.
#[derive(Clone, Debug)]
pub struct Case {
    name: String,
    kind: CaseKind,
    tree: TreeDescription,
    /// The paths of the case's apps declared `running`.
    running: Vec<PathBuf>,
    /// The apps of `[[case.running]]`: open, from no apps folder of the sandbox.
    running_elsewhere: Vec<RunningApp>,
    /// The apps of `[[case.elsewhere]]`: installed outside the apps folders, known to Spotlight only.
    elsewhere: Vec<(String, BundleId)>,
    /// `spotlight = "off"`: Spotlight cannot be asked in the case's sandbox.
    spotlight_off: bool,
    /// What the guarded folders and other apps' data allow, in the order of [`Capability`]; one the case does not
    /// state is absent.
    access: Vec<(Capability, Access)>,
    /// The macOS version the case states (`macos = "15.4"`).
    macos: Option<SystemVersion>,
    /// The installer receipts of `[[case.receipt]]`: the package's file name and the age it was installed at.
    receipts: Vec<(String, String)>,
    /// The calls of `[[case.tool]]`: what a program answers in the case's sandbox.
    tools: Vec<StatedTool>,
    expect: Option<Expectation>,
    map: Option<MapExpectation>,
    largest: Option<LargestExpectation>,
    uninstall: Option<UninstallExpectation>,
    left_out: Vec<LeftOutExpectation>,
    insights: Vec<InsightExpectation>,
}

/// A negative case is never offered and stays untouched; a positive case is offered and cleaned as it expects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseKind {
    Negative,
    Positive,
    /// A part of a demo world: what it shows is not checked.
    Scenery,
}

/// What a positive case expects: the finding's item, its category and danger, and what a cleanup does with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expectation {
    item: PathBuf,
    category: CategoryId,
    danger: DangerLevel,
    after: AfterCleanup,
}

/// What a case expects the module to leave out: `items` items of `category`, left out for `reason`, one of the
/// module's declared problems. The harness adds up every case's expectations and compares them exactly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftOutExpectation {
    category: CategoryId,
    reason: TextKey,
    items: u32,
}

/// What a case expects its file's module to describe and never offer: the insight called `identity`, with `figures`
/// numbers and `parts` parts, `commands` of them carrying a command. The harness fails on an insight no case expects.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsightExpectation {
    identity: String,
    figures: u32,
    parts: u32,
    commands: u32,
}

/// Where the storage map's by-place view must show a case's files: in which part, under which folder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapExpectation {
    part: MapPartName,
    at: PathBuf,
}

/// The part of the map a case's files are in: a positive's are explained, a negative's are the user's folders.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MapPartName {
    Explained,
    YourFolders,
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
    demo: Option<RawDemo>,
    high: Option<RawHigh>,
    #[serde(default)]
    settings: BTreeMap<String, toml::Value>,
    #[serde(default, rename = "case")]
    cases: Vec<RawCase>,
}

// Its files are every key but `name`, `kind`, `spotlight`, `access`, `macos`, `expect`, `map`, `largest`, `uninstall`,
// `left-out`, `running`, `elsewhere`, `receipt`, `tool` and `insight` (`[[case.running]]`, `[[case.elsewhere]]`,
// `[[case.receipt]]`, `[[case.tool]]` and `[[case.insight]]`, taken out first), read as a `TreeDescription`, which
// refuses an unknown one; each `[[case.app]]` loses its `running` first.
#[derive(Deserialize)]
struct RawCase {
    name: String,
    kind: CaseKind,
    spotlight: Option<RawSpotlight>,
    access: Option<BTreeMap<String, RawAccess>>,
    macos: Option<String>,
    expect: Option<RawExpectation>,
    map: Option<RawMap>,
    largest: Option<RawLargest>,
    uninstall: Option<RawUninstall>,
    #[serde(default, rename = "left-out")]
    left_out: Vec<RawLeftOut>,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMap {
    part: MapPartName,
    at: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLeftOut {
    category: String,
    reason: String,
    items: u32,
}

/// A `[[case.running]]`: an app that runs from no apps folder; its bundle ids are checked once read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawRunning {
    name: String,
    #[serde(default)]
    bundle_ids: Vec<String>,
}

/// A `[[case.elsewhere]]`: an app Spotlight finds outside the apps folders; its bundle id is checked once read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
struct RawElsewhere {
    name: String,
    bundle_id: String,
}

/// A `[[case.receipt]]`: an installer receipt of the sandbox's Mac; its package and age are checked once read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReceipt {
    package: String,
    installed: String,
}

/// A `[[case.insight]]`: an insight the module describes; checked once read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawInsight {
    identity: String,
    figures: u32,
    parts: u32,
    commands: u32,
}

/// A `[[case.tool]]`: a call of a program and its answer; checked once read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTool {
    program: String,
    args: Vec<String>,
    #[serde(default)]
    status: i32,
    #[serde(default)]
    stdout: String,
}

/// `spotlight`: the only value is `off`; Spotlight is on when a case says nothing.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RawSpotlight {
    Off,
}

/// What a guarded folder allows, as a case file and `world.toml` write it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RawAccess {
    Granted,
    Denied,
    NotAsked,
}

impl RawAccess {
    /// The state it stands for.
    pub(crate) const fn access(self) -> Access {
        match self {
            Self::Granted => Access::Granted,
            Self::Denied => Access::Denied,
            Self::NotAsked => Access::NotAsked,
        }
    }

    /// The value of `state`; `None` for a state no case can write.
    pub(crate) const fn of(state: Access) -> Option<Self> {
        match state {
            Access::Granted => Some(Self::Granted),
            Access::Denied => Some(Self::Denied),
            Access::NotAsked => Some(Self::NotAsked),
            Access::Unknown | Access::NotApplicable => None,
        }
    }
}

/// The capability an `access` key names (its id: `desktop`, `documents`, `downloads`, `other-apps-data`): a guarded
/// folder or other apps' data, never every file.
pub(crate) fn stated_capability(key: &str) -> Option<Capability> {
    [
        Capability::Folder(GuardedFolder::Desktop),
        Capability::Folder(GuardedFolder::Documents),
        Capability::Folder(GuardedFolder::Downloads),
        Capability::OtherAppsData,
    ]
    .into_iter()
    .find(|capability| capability.id() == key)
}

/// The one value of the top-level `high`: any other is refused by the file's parse error.
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum RawHigh {
    Withheld,
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
        if raw.demo.is_some() && raw.module.is_some() {
            return Err(CaseError::DemoWithModule { feature: raw.feature });
        }
        check_settings(&raw.settings)?;
        let mut names = HashSet::new();
        let mut written: HashMap<PathBuf, String> = HashMap::new();
        let mut seen_positive = false;
        let mut insights: HashMap<String, String> = HashMap::new();
        let mut cases = Vec::with_capacity(raw.cases.len());
        for raw_case in raw.cases {
            match (raw.demo.is_some(), raw_case.kind) {
                (true, CaseKind::Negative | CaseKind::Positive) => {
                    return Err(CaseError::CheckedCaseInDemo { case: raw_case.name });
                }
                (false, CaseKind::Scenery) => return Err(CaseError::SceneryOutsideDemo { case: raw_case.name }),
                (true, CaseKind::Scenery) | (false, CaseKind::Negative | CaseKind::Positive) => {}
            }
            let (case, pending) = Case::from_raw(raw_case)?;
            if !names.insert(case.name.clone()) {
                return Err(CaseError::DuplicateName { case: case.name });
            }
            match case.kind {
                CaseKind::Positive => seen_positive = true,
                CaseKind::Negative if seen_positive => {
                    return Err(CaseError::PositiveBeforeNegative { case: case.name });
                }
                // A demo file has scenery only: there is no order to keep.
                CaseKind::Negative | CaseKind::Scenery => {}
            }
            for insight in &case.insights {
                if let Some(other) = insights.insert(insight.identity.clone(), case.name.clone()) {
                    let identity = insight.identity.clone();
                    return Err(CaseError::InsightTwice { case: case.name, other, identity });
                }
            }
            for path in case.tree.paths() {
                if let Some(other) = written.insert(path.to_path_buf(), case.name.clone()) {
                    return Err(CaseError::PathWrittenTwice { case: case.name, other, path: path.to_path_buf() });
                }
            }
            cases.push((case, pending));
        }
        let checked: Vec<&Case> = cases.iter().map(|(case, _)| case).collect();
        check_uninstalled_bundles(&checked)?;
        merge_trees(cases.iter().map(|(case, _)| case))?;
        // Every check passed: only now are the ids kept, so a refused file keeps nothing.
        let module = raw
            .module
            .map(|module| ModuleId::new(leak(module.clone())).map_err(|source| CaseError::Module { module, source }))
            .transpose()?;
        let demo = raw.demo.map(|demo| Demo::from_raw(&raw.feature, demo)).transpose()?;
        let cases = cases
            .into_iter()
            .map(|(mut case, pending)| {
                case.expect = pending.expect.map(|pending| pending.into_expectation(&case.name)).transpose()?;
                case.left_out = pending
                    .left_out
                    .into_iter()
                    .map(|pending| pending.into_expectation(&case.name))
                    .collect::<Result<_, _>>()?;
                Ok(case)
            })
            .collect::<Result<_, CaseError>>()?;
        let high = match raw.high {
            Some(RawHigh::Withheld) => HighLevel::Withheld,
            None => HighLevel::Offered,
        };
        Ok(Self { feature: raw.feature, module, demo, high, settings: raw.settings, cases })
    }

    /// Checks every expectation against the categories and the declared problems of the modules: the category
    /// exists and the danger is not below its level; a left-out's category exists and its reason is a problem of
    /// that category's module.
    pub fn check(&self, categories: &[CategoryDescriptor], problems: &[TextKey]) -> Result<(), CaseError> {
        for case in &self.cases {
            for left_out in &case.left_out {
                if !categories.iter().any(|descriptor| descriptor.id() == left_out.category) {
                    return Err(CaseError::UnknownCategory {
                        case: case.name.clone(),
                        category: left_out.category.to_string(),
                    });
                }
                let declared = |problem: &&TextKey| {
                    **problem == left_out.reason && problem.namespace() == module_of(left_out.category.as_str())
                };
                if !problems.iter().any(|problem| declared(&problem)) {
                    return Err(CaseError::UndeclaredReason {
                        case: case.name.clone(),
                        category: left_out.category.to_string(),
                        reason: left_out.reason.to_string(),
                    });
                }
            }
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

    /// What the file's cases expect to be left out, added up per category and reason, sorted by both.
    pub fn left_outs(&self) -> Vec<LeftOutExpectation> {
        let mut sums: Vec<LeftOutExpectation> = Vec::new();
        for expected in self.cases.iter().flat_map(|case| &case.left_out) {
            match sums.iter_mut().find(|sum| (sum.category, &sum.reason) == (expected.category, &expected.reason)) {
                Some(sum) => sum.items = sum.items.saturating_add(expected.items),
                None => sums.push(expected.clone()),
            }
        }
        sums.sort_by(|a, b| (a.category.as_str(), a.reason.as_str()).cmp(&(b.category.as_str(), b.reason.as_str())));
        sums
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

    /// Whether the autotest runs the file with High offered (no `high` key) or withheld (`high = "withheld"`).
    pub const fn high(&self) -> HighLevel {
        self.high
    }

    /// The `[demo]` header of a demo world; `None` for a test's case file.
    pub const fn demo(&self) -> Option<&Demo> {
        self.demo.as_ref()
    }

    /// The cases, negatives first.
    pub fn cases(&self) -> &[Case] {
        &self.cases
    }

    /// The file's `[settings]` by key (`<module>.<name>`): a bool, an integer or a string; empty when the file has
    /// none. Every case of the file runs with them.
    pub const fn settings(&self) -> &BTreeMap<String, toml::Value> {
        &self.settings
    }
}

/// The `[settings]` of several case files as one table; a key two files set to different values is refused, as a
/// path written twice is.
pub fn merged_settings(files: &[CaseFile]) -> Result<BTreeMap<String, toml::Value>, CaseError> {
    let mut merged = BTreeMap::new();
    for (key, value) in files.iter().flat_map(|file| &file.settings) {
        if merged.insert(key.clone(), value.clone()).is_some_and(|first| first != *value) {
            return Err(CaseError::SettingTwice { key: key.clone() });
        }
    }
    Ok(merged)
}

/// Every key is `<module>.<name>` in kebab case and every value a bool, an integer or a string.
fn check_settings(settings: &BTreeMap<String, toml::Value>) -> Result<(), CaseError> {
    for (key, value) in settings {
        if SettingKey::new(key.as_str()).is_err() {
            return Err(CaseError::SettingKey { key: key.clone() });
        }
        if !matches!(value, toml::Value::Boolean(_) | toml::Value::Integer(_) | toml::Value::String(_)) {
            return Err(CaseError::SettingValue { key: key.clone() });
        }
    }
    Ok(())
}

impl Case {
    /// The case without its expectation, which is kept apart until the whole file is accepted.
    fn from_raw(mut raw: RawCase) -> Result<(Self, Pending), CaseError> {
        let name = raw.name;
        let running = take_running(&mut raw.files)
            .map_err(|source| CaseError::Files { case: name.clone(), source: Box::new(source) })?;
        let running_elsewhere = take_running_elsewhere(&name, &mut raw.files)?;
        let elsewhere = take_elsewhere(&name, &mut raw.files)?;
        let receipts = take_receipts(&name, &mut raw.files)?;
        let tools = take_tools(&name, &mut raw.files)?;
        let insights = take_insights(&name, &mut raw.files)?;
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
        check_apps(&name, &tree)?;
        let running = tree.apps().iter().zip(running).filter(|(_, running)| *running).map(|(app, _)| app.path.clone());
        let running = running.collect();
        if raw.kind == CaseKind::Scenery
            && (raw.expect.is_some()
                || raw.map.is_some()
                || raw.uninstall.is_some()
                || !raw.left_out.is_empty()
                || !insights.is_empty())
        {
            return Err(CaseError::SceneryWithExpectation { case: name });
        }
        let uninstall =
            raw.uninstall.map(|raw_uninstall| UninstallExpectation::from_raw(&name, raw.kind, &tree, raw_uninstall));
        let uninstall = uninstall.transpose()?;
        let expect = match (raw.kind, raw.expect) {
            (CaseKind::Negative, Some(_)) => return Err(CaseError::NegativeWithExpect { case: name }),
            // A positive uninstall is checked on the Apps screen: its own expectation.
            (CaseKind::Positive, None) if uninstall.is_some() => None,
            (CaseKind::Positive, None) => return Err(CaseError::PositiveWithoutExpect { case: name }),
            (CaseKind::Negative | CaseKind::Scenery, _) => None,
            (CaseKind::Positive, Some(raw)) => Some(PendingExpectation::from_raw(&name, &tree, raw)?),
        };
        let map = raw.map.map(|map| MapExpectation::from_raw(&name, raw.kind, &tree, map)).transpose()?;
        let largest = raw.largest.map(|raw| LargestExpectation::from_raw(&name, &tree, raw)).transpose()?;
        if largest.is_none() && expect.as_ref().is_some_and(|expect| expect.category == OWN_FILE_ID) {
            return Err(CaseError::OwnFileWithoutLargest { case: name });
        }
        let left_out =
            raw.left_out.into_iter().map(|raw| PendingLeftOut::from_raw(&name, raw)).collect::<Result<_, _>>()?;
        let spotlight_off = matches!(raw.spotlight, Some(RawSpotlight::Off));
        let access = raw
            .access
            .unwrap_or_default()
            .into_iter()
            .map(|(key, value)| match stated_capability(&key) {
                Some(capability) => Ok((capability, value.access())),
                None => Err(CaseError::AccessKey { case: name.clone(), key }),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let macos = raw
            .macos
            .map(|value| SystemVersion::parse(&value).ok_or(CaseError::MacosVersion { case: name.clone(), value }))
            .transpose()?;
        let case = Self {
            name,
            kind: raw.kind,
            tree,
            running,
            running_elsewhere,
            elsewhere,
            spotlight_off,
            access,
            macos,
            receipts,
            tools,
            expect: None,
            map,
            largest,
            uninstall,
            left_out: vec![],
            insights,
        };
        Ok((case, Pending { expect, left_out }))
    }

    /// The case's name, unique in its file: a failure names the case by it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the case is negative (never offered), positive (offered and cleaned as expected) or scenery (a part of
    /// a demo world, not checked).
    pub const fn kind(&self) -> CaseKind {
        self.kind
    }

    /// The files the case writes, relative to the sandbox folder.
    pub const fn tree(&self) -> &TreeDescription {
        &self.tree
    }

    /// The case's apps declared `running`: open while the app scans.
    pub fn running_apps(&self) -> impl Iterator<Item = &AppBundle> {
        self.tree.apps().iter().filter(|app| self.running.contains(&app.path))
    }

    /// The case's `[[case.running]]`: apps open while the app scans that are installed in no apps folder.
    pub fn running_elsewhere(&self) -> &[RunningApp] {
        &self.running_elsewhere
    }

    /// The case's `[[case.elsewhere]]`: apps Spotlight finds outside the apps folders, by name and bundle id.
    pub fn elsewhere(&self) -> &[(String, BundleId)] {
        &self.elsewhere
    }

    /// Whether the case says `spotlight = "off"`: then Spotlight cannot be asked in the whole sandbox.
    pub const fn spotlight_off(&self) -> bool {
        self.spotlight_off
    }

    /// What the case states the guarded folders and other apps' data allow; empty when it states nothing.
    pub fn access(&self) -> &[(Capability, Access)] {
        &self.access
    }

    /// The macOS version the case states; `None` when it states none.
    pub const fn macos(&self) -> Option<SystemVersion> {
        self.macos
    }

    /// The case's `[[case.receipt]]`: each package's file name and the age it was installed at (`"10d"`).
    pub fn receipts(&self) -> &[(String, String)] {
        &self.receipts
    }

    /// The case's `[[case.tool]]`: the calls it answers, in order.
    pub(crate) fn tools(&self) -> &[StatedTool] {
        &self.tools
    }

    /// The expectation of a positive case; a negative case has none.
    pub const fn expect(&self) -> Option<&Expectation> {
        self.expect.as_ref()
    }

    /// What the case expects the module to leave out.
    pub fn left_out(&self) -> &[LeftOutExpectation] {
        &self.left_out
    }

    /// Where the storage map must show the case's files, when the case says.
    pub const fn map(&self) -> Option<&MapExpectation> {
        self.map.as_ref()
    }

    /// What the Largest screen must show of the case's files, when the case says.
    pub const fn largest(&self) -> Option<&LargestExpectation> {
        self.largest.as_ref()
    }

    /// What the Apps screen must show of one of the case's apps, and what its uninstall does, when the case says.
    pub const fn uninstall(&self) -> Option<&UninstallExpectation> {
        self.uninstall.as_ref()
    }

    /// The insights the case expects its file's module to describe.
    pub fn insights(&self) -> &[InsightExpectation] {
        &self.insights
    }
}

impl InsightExpectation {
    /// The insight's identity, unique in its case file.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// How many numbers it shows.
    pub const fn figures(&self) -> u32 {
        self.figures
    }

    /// How many parts it lists.
    pub const fn parts(&self) -> u32 {
        self.parts
    }

    /// How many of its parts carry a command.
    pub const fn commands(&self) -> u32 {
        self.commands
    }
}

impl MapExpectation {
    /// Checks the part against the case's kind and resolves `at`: one of the case's paths or their folders, by
    /// default the folder of its first file.
    fn from_raw(case: &str, kind: CaseKind, tree: &TreeDescription, raw: RawMap) -> Result<Self, CaseError> {
        match (kind, raw.part) {
            (CaseKind::Negative, MapPartName::YourFolders) | (CaseKind::Positive, MapPartName::Explained) => {}
            (CaseKind::Negative | CaseKind::Positive | CaseKind::Scenery, part) => {
                return Err(CaseError::MapOnTheWrongKind { case: case.into(), part });
            }
        }
        let at = match raw.at {
            Some(at) => at,
            None => tree
                .files()
                .chain(tree.paths())
                .next()
                .and_then(Path::parent)
                .filter(|folder| !folder.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .ok_or_else(|| CaseError::MapWithoutFolder { case: case.into() })?,
        };
        let own = tree.paths().chain(tree.markers()).any(|path| path.ancestors().any(|folder| folder == at));
        if !is_plain(&at) || !own {
            return Err(CaseError::MapAtOutsideTheCase { case: case.into(), at });
        }
        Ok(Self { part: raw.part, at })
    }

    /// The part of the map the case's files are in.
    pub const fn part(&self) -> MapPartName {
        self.part
    }

    /// The folder the by-place view shows the case's files under, relative to the sandbox folder.
    pub fn at(&self) -> &Path {
        &self.at
    }
}

/// What a case holds that is kept apart until the whole file is accepted.
struct Pending {
    expect: Option<PendingExpectation>,
    left_out: Vec<PendingLeftOut>,
}

/// A checked left-out expectation whose category id is not kept yet.
struct PendingLeftOut {
    category: String,
    reason: TextKey,
    items: u32,
}

impl PendingLeftOut {
    fn from_raw(case: &str, raw: RawLeftOut) -> Result<Self, CaseError> {
        if raw.items == 0 {
            return Err(CaseError::LeftOutWithoutItems { case: case.into(), category: raw.category });
        }
        check_category(&raw.category).map_err(|source| CaseError::CategoryId {
            case: case.into(),
            category: raw.category.clone(),
            source,
        })?;
        let reason = TextKey::new(raw.reason.as_str())
            .map_err(|_| CaseError::LeftOutReason { case: case.into(), reason: raw.reason })?;
        Ok(Self { category: raw.category, reason, items: raw.items })
    }

    fn into_expectation(self, case: &str) -> Result<LeftOutExpectation, CaseError> {
        let category = category_id(leak(self.category.clone())).map_err(|source| CaseError::CategoryId {
            case: case.into(),
            category: self.category,
            source,
        })?;
        Ok(LeftOutExpectation { category, reason: self.reason, items: self.items })
    }
}

impl LeftOutExpectation {
    /// The category whose items are left out.
    pub const fn category(&self) -> CategoryId {
        self.category
    }

    /// Why: one of the module's declared problems.
    pub const fn reason(&self) -> &TextKey {
        &self.reason
    }

    /// How many items.
    pub const fn items(&self) -> u32 {
        self.items
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

/// Removes `running` from every `[[case.app]]` of the case's files and returns it, in order, `false` when absent.
fn take_running(files: &mut toml::Table) -> Result<Vec<bool>, toml::de::Error> {
    let Some(toml::Value::Array(apps)) = files.get_mut("app") else { return Ok(Vec::new()) };
    apps.iter_mut()
        .map(|app| match app.as_table_mut().and_then(|table| table.remove("running")) {
            Some(running) => running.try_into(),
            None => Ok(false),
        })
        .collect()
}

/// Removes `[[case.running]]` from the case's files and checks each: a name and at least one bundle id, every id one
/// the app would read.
fn take_running_elsewhere(case: &str, files: &mut toml::Table) -> Result<Vec<RunningApp>, CaseError> {
    let Some(entries) = files.remove("running") else { return Ok(Vec::new()) };
    let entries: Vec<RawRunning> =
        entries.try_into().map_err(|source| CaseError::Files { case: case.into(), source: Box::new(source) })?;
    entries
        .into_iter()
        .map(|entry| {
            if entry.bundle_ids.is_empty() {
                return Err(CaseError::RunningWithoutBundleId { case: case.into(), name: entry.name });
            }
            let bundle_ids = entry
                .bundle_ids
                .into_iter()
                .map(|id| BundleId::new(id.as_str()).map_err(|_| CaseError::RunningBundleId { case: case.into(), id }))
                .collect::<Result<_, _>>()?;
            Ok(RunningApp { name: entry.name, bundle_ids })
        })
        .collect()
}

/// Removes `[[case.elsewhere]]` from the case's files and checks each: a name that is one plain file name (it names
/// the app's bundle, `<name>.app`) and a bundle id the app would read.
fn take_elsewhere(case: &str, files: &mut toml::Table) -> Result<Vec<(String, BundleId)>, CaseError> {
    let Some(entries) = files.remove("elsewhere") else { return Ok(Vec::new()) };
    let entries: Vec<RawElsewhere> =
        entries.try_into().map_err(|source| CaseError::Files { case: case.into(), source: Box::new(source) })?;
    entries
        .into_iter()
        .map(|entry| {
            if !is_bundle_name(&entry.name) {
                return Err(CaseError::ElsewhereName { case: case.into(), name: entry.name });
            }
            let id = BundleId::new(entry.bundle_id.as_str())
                .map_err(|_| CaseError::ElsewhereBundleId { case: case.into(), id: entry.bundle_id.clone() })?;
            Ok((entry.name, id))
        })
        .collect()
}

/// `true` for a package file name: one plain file name with something before `.pkg`.
pub(crate) fn is_pkg_name(name: &str) -> bool {
    is_bundle_name(name) && name.strip_suffix(".pkg").is_some_and(|stem| !stem.is_empty())
}

/// Removes `[[case.receipt]]` from the case's files and checks each: a package that is one plain file name ending in
/// `.pkg`, and an age it was installed at.
fn take_receipts(case: &str, files: &mut toml::Table) -> Result<Vec<(String, String)>, CaseError> {
    let Some(entries) = files.remove("receipt") else { return Ok(Vec::new()) };
    let entries: Vec<RawReceipt> =
        entries.try_into().map_err(|source| CaseError::Files { case: case.into(), source: Box::new(source) })?;
    entries
        .into_iter()
        .map(|entry| {
            if !is_pkg_name(&entry.package) {
                return Err(CaseError::ReceiptPackage { case: case.into(), package: entry.package });
            }
            if time_at_age(&entry.installed, SystemTime::UNIX_EPOCH + Duration::from_secs(1 << 40)).is_none() {
                let (package, installed) = (entry.package, entry.installed);
                return Err(CaseError::ReceiptInstalled { case: case.into(), package, installed });
            }
            Ok((entry.package, entry.installed))
        })
        .collect()
}

/// Removes `[[case.tool]]` from the case's files and checks each: a program relative to the sandbox whose every
/// component is a plain name, arguments without a NUL, an output of at most 64 KiB.
fn take_tools(case: &str, files: &mut toml::Table) -> Result<Vec<StatedTool>, CaseError> {
    let Some(entries) = files.remove("tool") else { return Ok(Vec::new()) };
    let entries: Vec<RawTool> =
        entries.try_into().map_err(|source| CaseError::Files { case: case.into(), source: Box::new(source) })?;
    entries
        .into_iter()
        .map(|entry| {
            StatedTool::new(&entry.program, entry.args, entry.status, entry.stdout).map_err(|problem| {
                let (case, program) = (case.into(), entry.program);
                match problem {
                    ToolProblem::Program => CaseError::ToolProgram { case, program },
                    ToolProblem::Argument => CaseError::ToolArgument { case, program },
                    ToolProblem::Output => CaseError::ToolOutput { case, program },
                }
            })
        })
        .collect()
}

/// Removes `[[case.insight]]` from the case's files and checks each: an identity in kebab case, no more commands than
/// parts.
fn take_insights(case: &str, files: &mut toml::Table) -> Result<Vec<InsightExpectation>, CaseError> {
    let Some(entries) = files.remove("insight") else { return Ok(Vec::new()) };
    let entries: Vec<RawInsight> =
        entries.try_into().map_err(|source| CaseError::Files { case: case.into(), source: Box::new(source) })?;
    entries
        .into_iter()
        .map(|entry| {
            let RawInsight { identity, figures, parts, commands } = entry;
            if !is_kebab_case(&identity) {
                return Err(CaseError::InsightIdentity { case: case.into(), identity });
            }
            if commands > parts {
                return Err(CaseError::InsightCommands { case: case.into(), identity, commands, parts });
            }
            Ok(InsightExpectation { identity, figures, parts, commands })
        })
        .collect()
}

/// `true` for a name that is one plain file name: not empty, no `/`, not `.` or `..`.
pub(crate) fn is_bundle_name(name: &str) -> bool {
    is_plain(Path::new(name)) && Path::new(name).components().count() == 1 && !name.contains('/')
}

/// Refuses an app outside the apps folders (or more than one folder below one), and anything but an app the case
/// writes into the fake home's apps folder.
fn check_apps(case: &str, tree: &TreeDescription) -> Result<(), CaseError> {
    for app in tree.apps() {
        let in_apps_folder = APPS_FOLDERS.iter().any(|folder| {
            app.path.strip_prefix(folder).is_ok_and(|inside| (1..=2).contains(&inside.components().count()))
        });
        if !in_apps_folder {
            return Err(CaseError::AppOutsideAppsFolders { case: case.into(), path: app.path.clone() });
        }
    }
    let not_an_app = tree.paths().chain(tree.markers()).find(|path| {
        path.strip_prefix(HOME_APPS_FOLDER).is_ok_and(|inside| !inside.as_os_str().is_empty())
            && !tree.apps().iter().any(|app| path.starts_with(&app.path))
    });
    match not_an_app {
        Some(path) => Err(CaseError::NotAnAppInHomeApps { case: case.into(), path: path.to_path_buf() }),
        None => Ok(()),
    }
}

/// `true` for a non-empty relative path of plain names only: no root, `.`, `..` or prefix.
pub(crate) fn is_plain(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|component| matches!(component, Component::Normal(_)))
}

/// The module part of a category id `<module>.<name>`.
fn module_of(id: &str) -> &str {
    id.split_once('.').map_or(id, |(module, _)| module)
}

/// Checks a category id `<module>.<name>` read from a case file, without keeping it: a module's, or the core's own
/// `core.own-file` (a row picked on the Largest screen).
pub(crate) fn check_category(id: &str) -> Result<(), IdError> {
    if id == OWN_FILE_ID {
        return Ok(());
    }
    ModuleId::check(module_of(id))?;
    CategoryId::check(module_of(id), id)
}

/// The category id `<module>.<name>` of a checked, kept id.
pub(crate) fn category_id(id: &'static str) -> Result<CategoryId, IdError> {
    if id == OWN_FILE_ID {
        return Ok(own_file().id());
    }
    CategoryId::new(ModuleId::new(module_of(id))?, id)
}

/// The bundle a positive case uninstalls is its own item: no other case writes at or below it.
fn check_uninstalled_bundles(cases: &[&Case]) -> Result<(), CaseError> {
    for case in cases {
        let Some(uninstall) = case.uninstall.as_ref().filter(|uninstall| uninstall.is_chosen()) else { continue };
        let bundle = uninstall.app();
        for other in cases.iter().filter(|other| other.name != case.name) {
            if let Some(path) = other.tree.paths().chain(other.tree.markers()).find(|path| path.starts_with(bundle)) {
                let (case, other, path) = (case.name.clone(), other.name.clone(), path.to_path_buf());
                return Err(CaseError::UninstallBundleShared { case, other, path });
            }
        }
    }
    Ok(())
}

/// The cases' files as one tree.
fn merge_trees<'a>(mut cases: impl Iterator<Item = &'a Case>) -> Result<TreeDescription, CaseError> {
    cases.try_fold(TreeDescription::default(), |tree, case| {
        tree.merge(case.tree.clone()).map_err(|source| CaseError::Sandbox { source })
    })
}

/// The core's ids hold `&'static str`; an id read from a case file lives for the rest of the process. Only the ids of
/// an accepted file are kept, and a dev-only tool reads a handful of them, so the leak is bounded.
pub(crate) fn leak(text: String) -> &'static str {
    Box::leak(text.into_boxed_str())
}

#[cfg(test)]
mod tests {
    use fresher_core::{CategoryDescriptor, CategoryId, DangerLevel, ModuleId, TextKey};

    fn trial_categories() -> Vec<CategoryDescriptor> {
        let trial = ModuleId::new("trial").unwrap();
        vec![CategoryDescriptor::new(CategoryId::new(trial, "trial.caches").unwrap(), DangerLevel::Low)]
    }

    const NEGATIVE: &str = r#"
[[case]]
name = "a trial folder with no marker next to it is never offered"
kind = "negative"
[[case.file]]
path = "home/bare/Sample app cache/blob.bin"
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

    /// A negative case that installs an app at `path`, with `extra` lines in its `[[case.app]]`.
    fn with_app(path: &str, extra: &str) -> String {
        format!(
            "[[case]]\nname = \"an app\"\nkind = \"negative\"\n[[case.app]]\npath = \"{path}\"\n\
             bundle-id = \"com.example.Tool\"\nname = \"Tool\"\n{extra}"
        )
    }

    fn file(cases: &[&str]) -> String {
        format!("feature = \"trial\"\nmodule = \"trial\"\n{}", cases.concat())
    }

    fn cache(danger: &str) -> String {
        positive(
            "a trial cache goes to the Trash",
            "home/work/Sample app cache/blob.bin",
            "home/work/Sample app cache",
            danger,
        )
    }

    fn left_out(category: &str, reason: &str, items: u32) -> String {
        format!("[[case.left-out]]\ncategory = \"{category}\"\nreason = \"{reason}\"\nitems = {items}\n")
    }

    const DOCKER: &str = "Applications/OrbStack.app/Contents/MacOS/xbin/docker";

    /// A `[[case.tool]]` of `program` with `extra` lines (its `args` included).
    fn tool(program: &str, extra: &str) -> String {
        format!("[[case.tool]]\nprogram = \"{program}\"\n{extra}")
    }

    /// A `[[case.insight]]` of `identity` with its counts.
    fn insight(identity: &str, figures: u32, parts: u32, commands: u32) -> String {
        format!(
            "[[case.insight]]\nidentity = \"{identity}\"\nfigures = {figures}\nparts = {parts}\ncommands = {commands}\n"
        )
    }

    fn trial_problems() -> Vec<TextKey> {
        vec![TextKey::new("trial.error.failed").unwrap()]
    }

    mod negative {
        use super::{
            DOCKER, NEGATIVE, cache, file, insight, left_out, positive, tool, trial_categories, trial_problems,
            with_app,
        };
        use crate::{CaseError, CaseFile, MapPartName, merged_settings};

        fn refused(text: &str) -> CaseError {
            CaseFile::parse(text).unwrap_err()
        }

        #[test]
        fn a_high_key_other_than_withheld_is_refused() {
            for value in ["\"offered\"", "\"Withheld\"", "\"\"", "true"] {
                let text = format!("feature = \"trial\"\nmodule = \"trial\"\nhigh = {value}\n{NEGATIVE}");

                let error = refused(&text);

                assert!(matches!(&error, CaseError::Parse { .. }), "{value}: {error}");
                assert!(error.to_string().contains("high"), "{value}: {error}");
            }
        }

        #[test]
        fn a_setting_key_off_the_shape_is_refused() {
            for key in ["demo", "Demo.count", "demo.Count", "demo..count", "demo.count_two", ".count", "demo.count."] {
                let text = format!("{}[settings]\n\"{key}\" = 3\n", file(&[NEGATIVE]));

                let error = refused(&text);

                assert!(matches!(&error, CaseError::SettingKey { key: found } if found == key), "{key}: {error}");
            }
        }

        #[test]
        fn a_setting_value_that_is_no_bool_integer_or_string_is_refused() {
            for value in ["1.5", "[1]", "{ a = 1 }", "1979-05-27"] {
                let text = format!("{}[settings]\n\"demo.count\" = {value}\n", file(&[NEGATIVE]));

                let error = refused(&text);

                assert!(matches!(&error, CaseError::SettingValue { key } if key == "demo.count"), "{value}: {error}");
            }
        }

        #[test]
        fn a_setting_set_twice_in_all_is_refused() {
            let set =
                |value: &str| CaseFile::parse(&format!("{}[settings]\n\"demo.count\" = {value}\n", file(&[NEGATIVE])));
            let (first, second) = (set("3").unwrap(), set("4").unwrap());

            let error = merged_settings(&[first, second]).unwrap_err();

            assert!(matches!(&error, CaseError::SettingTwice { key } if key == "demo.count"), "{error}");
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
        fn an_own_file_expectation_without_largest_is_refused() {
            let text = file(&[&cache("high").replace("trial.caches", "core.own-file")]);

            assert!(matches!(refused(&text), CaseError::OwnFileWithoutLargest { case } if case.contains("Trash")));
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
            for item in ["home/bare/Sample app cache", "home/work/Sample app cache/other", "", "/home/work"] {
                let text = file(&[&positive("p", "home/work/Sample app cache/blob.bin", item, "low")]);

                assert!(matches!(refused(&text), CaseError::ItemNotInCase { case, .. } if case == "p"), "{item}");
            }
        }

        #[test]
        fn an_unknown_category_is_refused() {
            let parsed = CaseFile::parse(&file(&[&cache("low").replace("trial.caches", "trial.logs")])).unwrap();

            let error = parsed.check(&trial_categories(), &[]).unwrap_err();

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

            let error = parsed.check(&trial_categories(), &[]).unwrap_err();

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
                let text = file(&[&NEGATIVE.replace("home/bare/Sample app cache/blob.bin", path)]);

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
        fn explained_on_a_negative_is_refused() {
            let text = file(&[&format!("{NEGATIVE}[case.map]\npart = \"explained\"\n")]);

            assert!(matches!(refused(&text), CaseError::MapOnTheWrongKind { case, part: MapPartName::Explained }
                if case.starts_with("a trial folder")));
        }

        #[test]
        fn your_folders_on_a_positive_is_refused() {
            let text = file(&[&format!("{}[case.map]\npart = \"your-folders\"\n", cache("low"))]);

            assert!(matches!(refused(&text), CaseError::MapOnTheWrongKind { case, part: MapPartName::YourFolders }
                if case.contains("Trash")));
        }

        #[test]
        fn a_map_at_outside_the_case_is_refused() {
            for at in ["home/work", "home/bare/other", "home/bare/Sample app cache/blob.bin/x", "", "/home/bare"] {
                let text = file(&[&format!("{NEGATIVE}[case.map]\npart = \"your-folders\"\nat = \"{at}\"\n")]);

                assert!(
                    matches!(refused(&text), CaseError::MapAtOutsideTheCase { case, .. }
                        if case.starts_with("a trial folder")),
                    "{at:?}"
                );
            }
        }

        #[test]
        fn a_map_of_a_case_whose_first_file_is_at_the_root_needs_an_at() {
            let text = file(&[&format!(
                "{}[case.map]\npart = \"your-folders\"\n",
                NEGATIVE.replace("home/bare/Sample app cache/blob.bin", "loose.bin")
            )]);

            assert!(
                matches!(refused(&text), CaseError::MapWithoutFolder { case } if case.starts_with("a trial folder"))
            );
        }

        #[test]
        fn an_unknown_map_field_is_refused() {
            let text = file(&[&format!("{NEGATIVE}[case.map]\npart = \"your-folders\"\ncolour = \"red\"\n")]);
            let unknown_part = file(&[&format!("{NEGATIVE}[case.map]\npart = \"free\"\n")]);

            assert!(matches!(refused(&text), CaseError::Parse { .. }));
            assert!(matches!(refused(&unknown_part), CaseError::Parse { .. }));
        }

        #[test]
        fn an_app_outside_the_apps_folders_is_refused() {
            for path in ["home/Tool.app", "Applications/A/B/Tool.app", "System/Tool.app", "home/Library/Tool.app"] {
                let error = refused(&file(&[&with_app(path, "")]));

                assert!(
                    matches!(&error, CaseError::AppOutsideAppsFolders { case, .. } if case == "an app"),
                    "{path}: {error}"
                );
            }
        }

        #[test]
        fn an_app_that_is_not_a_bundle_or_not_plain_is_refused() {
            for path in ["Applications/Tool", "Applications/../Tool.app", "/Applications/Tool.app"] {
                let error = refused(&file(&[&with_app(path, "")]));

                assert!(matches!(&error, CaseError::Tree { case, .. } if case == "an app"), "{path}: {error}");
            }
            let bad_id = with_app("Applications/Tool.app", "").replace("com.example.Tool", "not an id");
            assert!(matches!(refused(&file(&[&bad_id])), CaseError::Tree { .. }));
        }

        #[test]
        fn two_apps_at_one_path_are_refused() {
            let twice = format!(
                "{}{}",
                with_app("Applications/Tool.app", ""),
                "[[case.app]]\npath = \"Applications/Tool.app\"\nbundle-id = \"a.b\"\nname = \"B\"\n"
            );
            let other = with_app("Applications/Tool.app", "").replace("an app", "another app");

            assert!(matches!(refused(&file(&[&twice])), CaseError::Tree { .. }));
            assert!(matches!(
                refused(&file(&[&with_app("Applications/Tool.app", ""), &other])),
                CaseError::PathWrittenTwice { .. }
            ));
        }

        #[test]
        fn a_file_in_the_home_apps_folder_that_is_no_app_is_refused() {
            let loose = format!(
                "{}[[case.file]]\npath = \"home/Applications/notes.txt\"\n",
                with_app("home/Applications/Tool.app", "")
            );

            let error = refused(&file(&[&loose]));

            assert!(matches!(error, CaseError::NotAnAppInHomeApps { path, .. } if path.ends_with("notes.txt")));
        }

        #[test]
        fn a_file_inside_a_declared_app_is_refused() {
            for app in ["Applications/Tool.app", "home/Applications/Tool.app"] {
                let inside = format!("{}[[case.file]]\npath = \"{app}/Contents/Info.plist\"\n", with_app(app, ""));

                let error = refused(&file(&[&inside]));

                assert!(
                    matches!(error, CaseError::Tree { source: fresher_testing::FixtureError::InsideApp { .. }, .. }),
                    "{error}"
                );
            }
        }

        #[test]
        fn an_apps_last_use_manager_or_owner_off_the_shape_is_refused() {
            use fresher_testing::FixtureError;

            let last_used = refused(&file(&[&with_app("Applications/Tool.app", "last-used = \"long ago\"\n")]));
            let manager = refused(&file(&[&with_app("Applications/Tool.app", "installed-by = \"macports:tool\"\n")]));
            let owner = refused(&file(&[&with_app("Applications/Tool.app", "owner = \"root\"\n")]));

            assert!(
                matches!(last_used, CaseError::Tree { source: FixtureError::AppLastUsed { .. }, .. }),
                "{last_used}"
            );
            assert!(
                matches!(manager, CaseError::Tree { source: FixtureError::AppInstalledBy { .. }, .. }),
                "{manager}"
            );
            assert!(matches!(owner, CaseError::Files { .. }), "{owner}");
        }

        #[test]
        fn an_app_with_a_control_character_in_its_name_is_refused() {
            let named = "[[case]]\nname = \"an app\"\nkind = \"negative\"\n[[case.app]]\n\
                         path = \"Applications/Tool.app\"\nbundle-id = \"com.example.Tool\"\nname = \"Tool\\u0007\"\n";

            let error = refused(&file(&[named]));

            assert!(
                matches!(error, CaseError::Tree { source: fresher_testing::FixtureError::AppTextControl { .. }, .. }),
                "{error}"
            );
        }

        #[test]
        fn an_app_with_an_unknown_field_or_a_running_that_is_no_bool_is_refused() {
            for extra in ["colour = \"red\"\n", "running = \"yes\"\n"] {
                let error = refused(&file(&[&with_app("Applications/Tool.app", extra)]));

                assert!(matches!(error, CaseError::Files { .. }), "{extra}");
            }
        }

        #[test]
        fn a_running_entry_without_a_bundle_id_is_refused() {
            for ids in ["", "bundle-ids = []\n"] {
                let text = file(&[&format!("{NEGATIVE}[[case.running]]\nname = \"CLion\"\n{ids}")]);

                assert!(
                    matches!(refused(&text), CaseError::RunningWithoutBundleId { name, .. } if name == "CLion"),
                    "{ids:?}"
                );
            }
        }

        #[test]
        fn a_running_entry_with_an_invalid_bundle_id_is_refused() {
            let text = file(&[&format!(
                "{NEGATIVE}[[case.running]]\nname = \"CLion\"\nbundle-ids = [\"a.b\", \"not an id\"]\n"
            )]);

            assert!(matches!(refused(&text), CaseError::RunningBundleId { id, .. } if id == "not an id"));
        }

        #[test]
        fn a_running_entry_with_an_unknown_field_is_refused() {
            let text = file(&[&format!(
                "{NEGATIVE}[[case.running]]\nname = \"CLion\"\nbundle-ids = [\"a.b\"]\ncolour = \"red\"\n"
            )]);

            assert!(matches!(refused(&text), CaseError::Files { .. }));
        }

        #[test]
        fn an_elsewhere_app_with_an_unknown_or_a_missing_field_is_refused() {
            for entry in [
                "name = \"Far\"\nbundle-id = \"com.example.Far\"\ncolour = \"red\"\n",
                "name = \"Far\"\n",
                "bundle-id = \"com.example.Far\"\n",
            ] {
                let text = file(&[&format!("{NEGATIVE}[[case.elsewhere]]\n{entry}")]);

                assert!(matches!(refused(&text), CaseError::Files { .. }), "{entry}");
            }
        }

        #[test]
        fn an_elsewhere_app_with_an_invalid_bundle_id_is_refused() {
            let text = file(&[&format!("{NEGATIVE}[[case.elsewhere]]\nname = \"Far\"\nbundle-id = \"not an id\"\n")]);

            assert!(matches!(refused(&text), CaseError::ElsewhereBundleId { id, .. } if id == "not an id"));
        }

        #[test]
        fn an_elsewhere_app_with_an_empty_or_a_path_name_is_refused() {
            for name in ["", "a/b", "..", ".", "Far/"] {
                let text = file(&[&format!(
                    "{NEGATIVE}[[case.elsewhere]]\nname = \"{name}\"\nbundle-id = \"com.example.Far\"\n"
                )]);

                assert!(
                    matches!(refused(&text), CaseError::ElsewhereName { name: found, .. } if found == name),
                    "{name:?}"
                );
            }
        }

        #[test]
        fn a_tool_with_an_absolute_program_is_refused() {
            let text = file(&[&format!("{NEGATIVE}{}", tool(&format!("/{DOCKER}"), "args = []\n"))]);

            let error = refused(&text);

            assert!(
                matches!(&error, CaseError::ToolProgram { case, program }
                    if case.starts_with("a trial folder") && *program == format!("/{DOCKER}")),
                "{error}"
            );
        }

        #[test]
        fn a_tool_with_a_dotted_program_is_refused() {
            for program in ["", ".", "..", "a/./docker", "a/../docker", "a//docker", "a/docker/", "./docker"] {
                let text = file(&[&format!("{NEGATIVE}{}", tool(program, "args = []\n"))]);

                let error = refused(&text);

                assert!(matches!(error, CaseError::ToolProgram { .. }), "{program:?}: {error}");
            }
        }

        #[test]
        fn a_tool_argument_with_a_nul_is_refused() {
            let text = file(&[&format!("{NEGATIVE}{}", tool(DOCKER, "args = [\"system\", \"d\\u0000f\"]\n"))]);

            let error = refused(&text);

            assert!(matches!(&error, CaseError::ToolArgument { program, .. } if program == DOCKER), "{error}");
        }

        #[test]
        fn a_tool_output_over_64_kib_is_refused() {
            let output = "x".repeat(64 * 1024 + 1);
            let text = file(&[&format!("{NEGATIVE}{}", tool(DOCKER, &format!("args = []\nstdout = \"{output}\"\n")))]);

            let error = refused(&text);

            assert!(matches!(&error, CaseError::ToolOutput { program, .. } if program == DOCKER), "{error}");
            let most = "x".repeat(64 * 1024);
            let text = file(&[&format!("{NEGATIVE}{}", tool(DOCKER, &format!("args = []\nstdout = \"{most}\"\n")))]);
            assert!(CaseFile::parse(&text).is_ok());
        }

        #[test]
        fn a_tool_with_a_missing_unknown_or_mistyped_field_is_refused() {
            for extra in ["args = []\nstderr = \"no\"\n", "args = []\nstatus = \"0\"\n", "stdout = \"\"\n"] {
                let text = file(&[&format!("{NEGATIVE}{}", tool(DOCKER, extra))]);

                let error = refused(&text);

                assert!(matches!(error, CaseError::Files { .. }), "{extra}: {error}");
            }
        }

        #[test]
        fn a_spotlight_value_other_than_off_is_refused() {
            for value in ["\"on\"", "\"Off\"", "false"] {
                let text = file(&[&NEGATIVE.replace("kind =", &format!("spotlight = {value}\nkind ="))]);

                assert!(matches!(refused(&text), CaseError::Parse { .. }), "{value}");
            }
        }

        #[test]
        fn all_files_stays_refused_in_access() {
            for key in ["all-files", "removable-volume", "x"] {
                let text = file(&[&NEGATIVE.replace("kind =", &format!("access = {{ {key} = \"granted\" }}\nkind ="))]);

                let error = refused(&text);

                assert!(
                    matches!(&error, CaseError::AccessKey { case, key: found }
                        if case.starts_with("a trial folder") && found == key),
                    "{key}: {error}"
                );
                assert!(error.to_string().contains("only the guarded folders desktop, documents and downloads, and"));
            }
        }

        #[test]
        fn an_insight_identity_twice_is_refused() {
            let other = NEGATIVE.replace("bare", "other").replace("a trial folder", "another folder");
            let first = format!("{NEGATIVE}{}", insight("orbstack", 3, 4, 3));
            let second = format!("{other}{}", insight("orbstack", 3, 0, 0));
            let within = format!("{NEGATIVE}{}{}", insight("orbstack", 3, 4, 3), insight("orbstack", 1, 0, 0));

            for text in [file(&[&first, &second]), file(&[&within])] {
                let error = refused(&text);

                assert!(
                    matches!(&error, CaseError::InsightTwice { identity, .. } if identity == "orbstack"),
                    "{error}"
                );
            }
        }

        #[test]
        fn more_commands_than_parts_is_refused() {
            let text = file(&[&format!("{NEGATIVE}{}", insight("orbstack", 3, 2, 3))]);

            let error = refused(&text);

            assert!(matches!(error, CaseError::InsightCommands { commands: 3, parts: 2, .. }), "{error}");
        }

        #[test]
        fn an_insight_off_the_shape_is_refused() {
            for identity in ["OrbStack", "docker_desktop", "", "-a"] {
                let error = refused(&file(&[&format!("{NEGATIVE}{}", insight(identity, 1, 0, 0))]));

                assert!(matches!(error, CaseError::InsightIdentity { .. }), "{identity}: {error}");
            }
            for extra in ["colour = 1\n", ""] {
                let raw = format!("[[case.insight]]\nidentity = \"orbstack\"\nfigures = 1\nparts = 0\n{extra}");
                let raw = if extra.is_empty() { raw } else { format!("{raw}commands = 0\n") };

                let error = refused(&file(&[&format!("{NEGATIVE}{raw}")]));

                assert!(matches!(error, CaseError::Files { .. }), "{extra}: {error}");
            }
        }

        #[test]
        fn a_macos_version_that_does_not_parse_is_refused() {
            for value in ["\"\"", "\"x\"", "\"15.x\"", "15"] {
                let text = file(&[&NEGATIVE.replace("kind =", &format!("macos = {value}\nkind ="))]);

                let error = refused(&text);

                assert!(matches!(&error, CaseError::MacosVersion { .. } | CaseError::Parse { .. }), "{value}: {error}");
            }
        }

        #[test]
        fn a_receipt_that_is_not_a_plain_pkg_name_is_refused() {
            for package in ["a/b.pkg", "x.dmg", ".pkg", "..", "Tool.pkg/"] {
                let receipt = format!("[[case.receipt]]\npackage = \"{package}\"\ninstalled = \"10d\"\n");
                let error = refused(&file(&[&format!("{NEGATIVE}{receipt}")]));

                assert!(matches!(error, CaseError::ReceiptPackage { .. }), "{package}: {error}");
            }
            let without = "[[case.receipt]]\npackage = \"Tool.pkg\"\n";
            assert!(matches!(refused(&file(&[&format!("{NEGATIVE}{without}")])), CaseError::Files { .. }));
            let late = "[[case.receipt]]\npackage = \"Tool.pkg\"\ninstalled = \"soon\"\n";
            let error = refused(&file(&[&format!("{NEGATIVE}{late}")]));
            assert!(matches!(error, CaseError::ReceiptInstalled { .. }), "{error}");
        }

        #[test]
        fn an_access_value_other_than_the_three_is_refused() {
            for value in ["\"yes\"", "true"] {
                let text = file(&[&NEGATIVE.replace("kind =", &format!("access = {{ downloads = {value} }}\nkind ="))]);

                assert!(matches!(refused(&text), CaseError::Parse { .. }), "{value}");
            }
        }

        #[test]
        fn a_left_out_with_no_items_is_refused() {
            let text = file(&[&format!("{NEGATIVE}{}", left_out("trial.caches", "trial.error.failed", 0))]);

            assert!(
                matches!(refused(&text), CaseError::LeftOutWithoutItems { category, .. } if category == "trial.caches")
            );
        }

        #[test]
        fn a_left_out_with_a_malformed_category_or_reason_is_refused() {
            let category = file(&[&format!("{NEGATIVE}{}", left_out("Trial", "trial.error.failed", 1))]);
            let reason = file(&[&format!("{NEGATIVE}{}", left_out("trial.caches", "Failed", 1))]);

            assert!(matches!(refused(&category), CaseError::CategoryId { .. }));
            assert!(matches!(refused(&reason), CaseError::LeftOutReason { reason, .. } if reason == "Failed"));
        }

        #[test]
        fn a_left_out_with_an_unknown_field_is_refused() {
            let text = file(&[&format!("{NEGATIVE}{}colour = 1\n", left_out("trial.caches", "trial.error.failed", 1))]);

            assert!(matches!(refused(&text), CaseError::Parse { .. }));
        }

        #[test]
        fn a_left_out_of_an_unknown_category_is_refused() {
            let text = file(&[&format!("{NEGATIVE}{}", left_out("trial.logs", "trial.error.failed", 1))]);

            let error = CaseFile::parse(&text).unwrap().check(&trial_categories(), &trial_problems()).unwrap_err();

            assert!(matches!(error, CaseError::UnknownCategory { category, .. } if category == "trial.logs"));
        }

        #[test]
        fn an_undeclared_left_out_reason_is_refused() {
            for reason in ["trial.error.other", "demo.error.failed"] {
                let text = file(&[&format!("{NEGATIVE}{}", left_out("trial.caches", reason, 1))]);

                let error = CaseFile::parse(&text).unwrap().check(&trial_categories(), &trial_problems()).unwrap_err();

                assert!(
                    matches!(error, CaseError::UndeclaredReason { reason: found, .. } if found == reason),
                    "{reason}"
                );
            }
        }

        #[test]
        fn a_problem_of_another_module_is_not_declared_for_the_category() {
            let text = file(&[&format!("{NEGATIVE}{}", left_out("trial.caches", "demo.error.failed", 1))]);
            let problems = vec![fresher_core::TextKey::new("demo.error.failed").unwrap()];

            let error = CaseFile::parse(&text).unwrap().check(&trial_categories(), &problems).unwrap_err();

            assert!(matches!(error, CaseError::UndeclaredReason { .. }));
        }

        #[test]
        fn a_core_category_other_than_own_file_is_refused() {
            for category in ["core.own-files", "core.caches"] {
                let text = file(&[&cache("high").replace("trial.caches", category)]);

                assert!(
                    matches!(refused(&text), CaseError::Module { .. } | CaseError::CategoryId { .. }),
                    "{category}"
                );
            }
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

        use fresher_core::{
            Access, Capability, DangerLevel, GuardedFolder, HighLevel, ModuleId, SkipCode, SystemVersion,
        };
        use fresher_testing::FIXTURE_NOW;

        use super::{
            DOCKER, NEGATIVE, cache, file, insight, left_out, tool, trial_categories, trial_problems, with_app,
        };
        use crate::world::StatedTool;
        use crate::{AfterCleanup, Case, CaseFile, CaseKind, MapPartName, merged_settings};

        #[test]
        fn a_file_without_high_offers_it() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE, &cache("low")])).unwrap();

            assert_eq!(parsed.high(), HighLevel::Offered);
        }

        #[test]
        fn high_withheld_is_read() {
            let text = format!("feature = \"trial\"\nmodule = \"trial\"\nhigh = \"withheld\"\n{NEGATIVE}");

            let parsed = CaseFile::parse(&text).unwrap();

            assert_eq!(parsed.high(), HighLevel::Withheld);
        }

        #[test]
        fn settings_are_read_for_every_case_of_the_file() {
            let text = format!(
                "{}[settings]\n\"demo.count\" = 3\n\"unused-toolchains.node-versions\" = true\n\"demo.mode\" = \"few\"\n",
                file(&[NEGATIVE, &cache("low")])
            );

            let parsed = CaseFile::parse(&text).unwrap();

            let settings: Vec<_> = parsed.settings().iter().map(|(key, value)| (key.as_str(), value.clone())).collect();
            assert_eq!(
                settings,
                [
                    ("demo.count", toml::Value::Integer(3)),
                    ("demo.mode", toml::Value::String("few".into())),
                    ("unused-toolchains.node-versions", toml::Value::Boolean(true)),
                ]
            );
            assert_eq!(parsed.cases().len(), 2);
        }

        #[test]
        fn a_case_files_text_comes_with_the_tree_description() {
            let case = "[[case]]\nname = \"a pin\"\nkind = \"negative\"\n[[case.file]]\npath = \"home/p/.nvmrc\"\ntext = \"20\\n\"\n";

            let parsed = CaseFile::parse(&file(&[case])).unwrap();

            assert_eq!(parsed.cases()[0].tree().files().count(), 1);
        }

        #[test]
        fn a_file_without_settings_has_none() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE])).unwrap();

            assert!(parsed.settings().is_empty());
        }

        #[test]
        fn merged_files_that_agree_on_a_setting_keep_it_once() {
            let text = format!("{}[settings]\n\"demo.count\" = 3\n", file(&[NEGATIVE]));
            let (first, second) = (CaseFile::parse(&text).unwrap(), CaseFile::parse(&text).unwrap());

            let merged = merged_settings(&[first, second]).unwrap();

            assert_eq!(merged.len(), 1);
        }

        #[test]
        fn left_out_expectations_add_up_per_category_and_reason() {
            let other = NEGATIVE.replace("bare", "other").replace("a trial folder", "another folder");
            let first = format!(
                "{NEGATIVE}{}{}",
                left_out("trial.caches", "trial.error.failed", 2),
                left_out("trial.kept", "trial.error.failed", 1)
            );
            let second = format!("{other}{}", left_out("trial.caches", "trial.error.failed", 3));
            let parsed = CaseFile::parse(&file(&[&first, &second])).unwrap();

            let all = parsed.left_outs();
            let sums: Vec<_> =
                all.iter().map(|sum| (sum.category().as_str(), sum.reason().as_str(), sum.items())).collect();

            assert_eq!(sums, [("trial.caches", "trial.error.failed", 5), ("trial.kept", "trial.error.failed", 1)]);
            assert_eq!(parsed.cases()[0].left_out().len(), 2);
        }

        #[test]
        fn a_declared_left_out_passes_the_check() {
            let text = file(&[&format!("{NEGATIVE}{}", left_out("trial.caches", "trial.error.failed", 1))]);

            CaseFile::parse(&text).unwrap().check(&trial_categories(), &trial_problems()).unwrap();
        }

        #[test]
        fn a_file_with_no_left_out_has_none() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE, &cache("low")])).unwrap();

            assert_eq!(parsed.left_outs(), []);
            assert!(parsed.cases().iter().all(|case| case.left_out().is_empty()));
        }

        #[test]
        fn a_case_file_parses_with_negatives_then_positives() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE, &cache("low")])).unwrap();

            parsed.check(&trial_categories(), &[]).unwrap();
            assert_eq!(parsed.feature(), "trial");
            assert_eq!(parsed.module().map(ModuleId::as_str), Some("trial"));
            let kinds: Vec<_> = parsed.cases().iter().map(Case::kind).collect();
            assert_eq!(kinds, [CaseKind::Negative, CaseKind::Positive]);
            assert!(parsed.cases()[0].expect().is_none());
            let expect = parsed.cases()[1].expect().unwrap();
            assert_eq!(expect.item(), Path::new("home/work/Sample app cache"));
            assert_eq!(expect.category().as_str(), "trial.caches");
            assert_eq!(expect.danger(), DangerLevel::Low);
            assert_eq!(expect.after(), AfterCleanup::InTrash);
        }

        #[test]
        fn a_case_with_a_map_parses() {
            let negative = format!("{NEGATIVE}[case.map]\npart = \"your-folders\"\nat = \"home/bare\"\n");
            let positive = format!("{}[case.map]\npart = \"explained\"\n", cache("low"));

            let plain = NEGATIVE.replace("bare", "plain").replace("a trial folder", "another folder");

            let parsed = CaseFile::parse(&file(&[&negative, &plain, &positive])).unwrap();

            let negative = parsed.cases()[0].map().unwrap();
            assert_eq!((negative.part(), negative.at()), (MapPartName::YourFolders, Path::new("home/bare")));
            assert!(parsed.cases()[1].map().is_none());
            // By default, the folder of the case's first file.
            let positive = parsed.cases()[2].map().unwrap();
            assert_eq!(
                (positive.part(), positive.at()),
                (MapPartName::Explained, Path::new("home/work/Sample app cache"))
            );
        }

        #[test]
        fn a_core_feature_has_no_module_and_a_skip_has_its_reason() {
            let text = file(&[&cache("low").replace("\"in-trash\"", "\"skipped\"\nreason = \"in-use\"")])
                .replace("module = \"trial\"\n", "");

            let parsed = CaseFile::parse(&text).unwrap();

            assert_eq!(parsed.module(), None);
            assert_eq!(parsed.cases()[0].expect().unwrap().after(), AfterCleanup::Skipped(SkipCode::InUse));
        }

        #[test]
        fn a_tool_reads_with_its_defaults() {
            let text = file(&[&format!("{NEGATIVE}{}", tool(DOCKER, "args = [\"system\", \"df\"]\n"))]);

            let parsed = CaseFile::parse(&text).unwrap();

            assert_eq!(
                parsed.cases()[0].tools(),
                [StatedTool {
                    program: Path::new(DOCKER).to_path_buf(),
                    args: vec!["system".into(), "df".into()],
                    status: 0,
                    stdout: String::new(),
                }]
            );
        }

        // A negative case may hold a link out of the sandbox on purpose: the scan must not follow it.
        #[test]
        fn a_symlink_may_point_out_of_the_sandbox() {
            let link = format!("{NEGATIVE}[[case.symlink]]\npath = \"home/bare/out\"\ntarget = \"/etc\"\n");

            assert!(CaseFile::parse(&file(&[&link])).is_ok());
        }

        #[test]
        fn apps_parse_in_every_apps_folder_with_their_running() {
            let extra = "[[case.app]]\npath = \"System/Applications/Utilities/Clock.app\"\nbundle-id = \"a.clock\"\n\
                         name = \"Clock\"\nrunning = false\n[[case.app]]\npath = \"home/Applications/Mine.app\"\n\
                         bundle-id = \"a.mine\"\nname = \"Mine\"\nversion = \"1\"\nbuild = \"2\"\n";
            let text = file(&[&with_app("Applications/Tool.app", &format!("running = true\n{extra}"))]);

            let parsed = CaseFile::parse(&text).unwrap();

            let case = &parsed.cases()[0];
            let apps: Vec<&str> = case.tree().apps().iter().map(|app| app.name.as_str()).collect();
            assert_eq!(apps, ["Tool", "Clock", "Mine"]);
            let running: Vec<&str> = case.running_apps().map(|app| app.name.as_str()).collect();
            assert_eq!(running, ["Tool"]);
            assert_eq!(case.tree().apps()[2].version.as_deref(), Some("1"));
        }

        #[test]
        fn running_entries_parse_with_their_bundle_ids() {
            let text = file(&[&format!(
                "{NEGATIVE}[[case.running]]\nname = \"CLion\"\nbundle-ids = [\"com.jetbrains.CLion\", \"a.helper\"]\n"
            )]);

            let parsed = CaseFile::parse(&text).unwrap();

            let running = parsed.cases()[0].running_elsewhere();
            assert_eq!(running.len(), 1);
            assert_eq!(running[0].name, "CLion");
            let ids: Vec<&str> = running[0].bundle_ids.iter().map(fresher_core::BundleId::as_str).collect();
            assert_eq!(ids, ["com.jetbrains.CLion", "a.helper"]);
            assert_eq!(parsed.cases()[0].running_apps().count(), 0);
        }

        #[test]
        fn elsewhere_apps_and_spotlight_off_parse() {
            let text = file(&[&format!(
                "{}[[case.elsewhere]]\nname = \"Far\"\nbundle-id = \"com.example.Far\"\n",
                NEGATIVE.replace("kind =", "spotlight = \"off\"\nkind =")
            )]);

            let parsed = CaseFile::parse(&text).unwrap();

            let case = &parsed.cases()[0];
            let elsewhere: Vec<(&str, &str)> =
                case.elsewhere().iter().map(|(name, id)| (name.as_str(), id.as_str())).collect();
            assert_eq!(elsewhere, [("Far", "com.example.Far")]);
            assert!(case.spotlight_off());
        }

        #[test]
        fn a_case_that_says_nothing_has_spotlight_on_and_nothing_elsewhere() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE])).unwrap();

            assert_eq!(parsed.cases()[0].elsewhere(), []);
            assert!(!parsed.cases()[0].spotlight_off());
        }

        #[test]
        fn access_parses_per_folder() {
            let access = "access = { downloads = \"granted\", desktop = \"denied\", documents = \"not-asked\" }\n";
            let text = file(&[&NEGATIVE.replace("kind =", &format!("{access}kind ="))]);

            let parsed = CaseFile::parse(&text).unwrap();

            assert_eq!(
                parsed.cases()[0].access(),
                [
                    (Capability::Folder(GuardedFolder::Desktop), Access::Denied),
                    (Capability::Folder(GuardedFolder::Documents), Access::NotAsked),
                    (Capability::Folder(GuardedFolder::Downloads), Access::Granted),
                ]
            );
        }

        #[test]
        fn other_apps_data_is_read_in_access() {
            for (value, state) in
                [("granted", Access::Granted), ("denied", Access::Denied), ("not-asked", Access::NotAsked)]
            {
                let access = format!("access = {{ other-apps-data = \"{value}\", downloads = \"granted\" }}\n");
                let text = file(&[&NEGATIVE.replace("kind =", &format!("{access}kind ="))]);

                let parsed = CaseFile::parse(&text).unwrap();

                assert_eq!(
                    parsed.cases()[0].access(),
                    [
                        (Capability::Folder(GuardedFolder::Downloads), Access::Granted),
                        (Capability::OtherAppsData, state)
                    ]
                );
            }
        }

        #[test]
        fn an_insight_expectation_reads() {
            let other = cache("low");
            let negative = format!("{NEGATIVE}{}{}", insight("docker-desktop", 3, 0, 0), insight("orbstack", 3, 4, 3));
            let positive = format!("{other}{}", insight("trial-two", 1, 2, 0));

            let parsed = CaseFile::parse(&file(&[&negative, &positive])).unwrap();

            let read = |case: usize| -> Vec<(&str, u32, u32, u32)> {
                parsed.cases()[case]
                    .insights()
                    .iter()
                    .map(|insight| (insight.identity(), insight.figures(), insight.parts(), insight.commands()))
                    .collect()
            };
            assert_eq!(read(0), [("docker-desktop", 3, 0, 0), ("orbstack", 3, 4, 3)]);
            assert_eq!(read(1), [("trial-two", 1, 2, 0)]);
            assert_eq!(parsed.cases()[0].tree().files().count(), 1, "an insight writes no file");
        }

        #[test]
        fn macos_and_receipts_parse() {
            let receipt = "[[case.receipt]]\npackage = \"Tool-3.1.pkg\"\ninstalled = \"10d\"\n\
                           [[case.receipt]]\npackage = \"Other.pkg\"\ninstalled = \"-2h\"\n";
            let text = file(&[&format!("{}{receipt}", NEGATIVE.replace("kind =", "macos = \"15.4\"\nkind ="))]);

            let parsed = CaseFile::parse(&text).unwrap();

            assert_eq!(parsed.cases()[0].macos(), Some(SystemVersion { major: 15, minor: 4 }));
            let expected = [("Tool-3.1.pkg".to_owned(), "10d".to_owned()), ("Other.pkg".to_owned(), "-2h".to_owned())];
            assert_eq!(parsed.cases()[0].receipts(), expected);
        }

        #[test]
        fn a_case_that_states_no_installs_has_none() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE])).unwrap();

            assert_eq!(parsed.cases()[0].macos(), None);
            assert_eq!(parsed.cases()[0].receipts(), []);
        }

        #[test]
        fn a_case_that_states_no_access_has_none() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE])).unwrap();

            assert_eq!(parsed.cases()[0].access(), []);
        }

        #[test]
        fn the_core_s_own_file_category_is_read() {
            let own = cache("high").replace("trial.caches", "core.own-file");
            let parsed = CaseFile::parse(&file(&[&format!("{own}[case.largest]\nshown = \"offered\"\n")])).unwrap();

            parsed.check(&[fresher_core::own_file()], &[]).unwrap();
            assert_eq!(parsed.cases()[0].expect().unwrap().category(), fresher_core::own_file().id());
        }

        #[test]
        fn a_raised_danger_is_accepted() {
            let parsed = CaseFile::parse(&file(&[&cache("high")])).unwrap();

            parsed.check(&trial_categories(), &[]).unwrap();
            assert_eq!(parsed.cases()[0].expect().unwrap().danger(), DangerLevel::High);
        }

        #[test]
        fn the_cases_merge_into_one_tree() {
            let parsed = CaseFile::parse(&file(&[NEGATIVE, &cache("low")])).unwrap();
            let folder = tempfile::tempdir().unwrap();

            let now = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW);
            parsed.tree().unwrap().build_at(folder.path(), now).unwrap();

            assert!(folder.path().join("home/bare/Sample app cache/blob.bin").is_file());
            assert!(folder.path().join("home/work/Sample app cache/blob.bin").is_file());
            assert!(folder.path().join("home/work").join(fresher_core::SANDBOX_MARKER).is_file());
        }
    }
}
