//! The contract every feature module implements and the engine drives.

use std::collections::BTreeSet;

use crate::ids::is_kebab_case;
use crate::settings::schema::ENABLED;
use crate::{
    CancelToken, Capability, CategoryDescriptor, CategoryId, Clock, CommandRunner, Insight, Interest, LeftOutItem,
    LeftOutKind, Match, ModuleGroup, ModuleId, ModuleSettings, NewFinding, PlaceSeen, SettingDescriptor, SystemFacts,
    Text, TextKey, ToolCommand, ToolId,
};

/// A feature of the app (Docker, `node_modules`, duplicates): it states what it wants to see during the walk and
/// turns what it is shown into findings. A module never walks the disk and never removes anything itself.
pub trait Module: Send + Sync {
    fn descriptor(&self) -> &ModuleDescriptor;

    /// What the module wants the walk to report; empty for a module that asks an external tool in `finish`.
    fn interests(&self) -> &[Interest];

    /// Called for every entry that matched one of the module's interests (`matched.interest` indexes `interests()`).
    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError>;

    /// Called once after the walk.
    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError>;
}

/// Who a module is: its id, its group, its categories, its settings, its texts, the checks of the tools it needs and
/// the problems it may report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDescriptor {
    id: ModuleId,
    group: ModuleGroup,
    categories: Vec<CategoryDescriptor>,
    settings: Vec<SettingDescriptor>,
    locales: Locales,
    tool_checks: Vec<ToolCommand>,
    problems: Vec<TextKey>,
    kept: Vec<TextKey>,
    choices: Vec<ModuleChoice>,
}

/// A way to pick a module's categories in one step ("Remove everything", "Keep settings"): a name and the categories
/// it selects. Its texts are `<module>.choice.<name>.label` and `.description`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleChoice {
    name: String,
    categories: Vec<CategoryId>,
}

impl ModuleChoice {
    /// A choice called `name` (kebab case) that selects `categories`; [`ModuleDescriptor::with_choices`] checks both.
    pub fn new(name: impl Into<String>, categories: Vec<CategoryId>) -> Self {
        Self { name: name.into(), categories }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// The categories the choice selects.
    pub fn categories(&self) -> &[CategoryId] {
        &self.categories
    }

    /// The choice's label: `<module>.choice.<name>.label`. A valid key only once the descriptor's `with_choices` has
    /// checked the choice's name.
    pub fn label_key(&self, module: ModuleId) -> TextKey {
        TextKey::from_valid(format!("{module}.choice.{}.label", self.name))
    }

    /// What the choice does, in a sentence: `<module>.choice.<name>.description`. A valid key only once the
    /// descriptor's `with_choices` has checked the choice's name.
    pub fn description_key(&self, module: ModuleId) -> TextKey {
        TextKey::from_valid(format!("{module}.choice.{}.description", self.name))
    }
}

/// A module's dictionaries, one flat JSON object per language, embedded with `include_str!`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Locales {
    pub en: &'static str,
}

/// Why a module descriptor was refused.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DescriptorError {
    #[error("category `{id}` is listed twice")]
    DuplicateCategory { id: String },
    #[error("category `{id}` is not in the namespace of module `{module}`")]
    ForeignCategory { id: String, module: String },
    #[error("setting `{key}` is listed twice")]
    DuplicateSetting { key: String },
    #[error("setting `{key}` is not in the namespace of module `{module}`")]
    ForeignSetting { key: String, module: String },
    #[error("setting `{key}` is added by the engine for every module")]
    ReservedSetting { key: String },
    #[error("tool `{tool}` has a check but no category needs it")]
    UnusedToolCheck { tool: String },
    #[error("tool `{tool}` is needed but has no check")]
    MissingToolCheck { tool: String },
    #[error("tool `{tool}` has two checks")]
    DuplicateToolCheck { tool: String },
    #[error("problem `{key}` is not in the namespace of module `{module}`")]
    ForeignProblem { key: String, module: String },
    #[error("problem `{key}` is listed twice")]
    DuplicateProblem { key: String },
    #[error("kept reason `{key}` is not a declared problem, or is listed twice")]
    KeptReason { key: String },
    #[error("choice `{name}` is not a kebab-case name, or is listed twice")]
    ChoiceName { name: String },
    #[error("choice `{name}` selects no category")]
    EmptyChoice { name: String },
    #[error("choice `{name}` selects category `{category}`, which is not the module's")]
    ChoiceCategory { name: String, category: String },
}

impl ModuleDescriptor {
    /// A descriptor whose categories and settings are unique and all in the module's namespace. The settings do not
    /// include `<module>.enabled`: the schema adds it.
    pub fn new(
        id: ModuleId,
        group: ModuleGroup,
        categories: Vec<CategoryDescriptor>,
        settings: Vec<SettingDescriptor>,
        locales: Locales,
    ) -> Result<Self, DescriptorError> {
        let mut seen = BTreeSet::new();
        for category in &categories {
            if category.id().module() != id {
                return Err(DescriptorError::ForeignCategory {
                    id: category.id().as_str().into(),
                    module: id.as_str().into(),
                });
            }
            if !seen.insert(category.id()) {
                return Err(DescriptorError::DuplicateCategory { id: category.id().as_str().into() });
            }
        }
        check_settings(id, &settings)?;
        Ok(Self {
            id,
            group,
            categories,
            settings,
            locales,
            tool_checks: Vec::new(),
            problems: Vec::new(),
            kept: Vec::new(),
            choices: Vec::new(),
        })
    }

    /// This descriptor with the command that tells whether a tool it needs is ready: success means ready. Refused
    /// when no category has a `Tool` need of that tool, or the tool already has a check.
    pub fn with_tool_check(mut self, check: ToolCommand) -> Result<Self, DescriptorError> {
        let tool = check.tool();
        if !self.needed_tools().any(|needed| needed == tool) {
            return Err(DescriptorError::UnusedToolCheck { tool: tool.as_str().into() });
        }
        if self.tool_checks.iter().any(|known| known.tool() == tool) {
            return Err(DescriptorError::DuplicateToolCheck { tool: tool.as_str().into() });
        }
        self.tool_checks.push(check);
        Ok(self)
    }

    /// This descriptor with the keys of the texts the module may report as a failure (`ModuleError::Failed`) or as the
    /// reason of a left-out (`FindingSink::left_out`). Refused when a key is outside the module's namespace or listed
    /// twice.
    pub fn with_problems(mut self, keys: Vec<TextKey>) -> Result<Self, DescriptorError> {
        let mut seen = BTreeSet::new();
        for key in &keys {
            if key.namespace() != self.id.as_str() {
                return Err(DescriptorError::ForeignProblem {
                    key: key.as_str().into(),
                    module: self.id.as_str().into(),
                });
            }
            if !seen.insert(key) {
                return Err(DescriptorError::DuplicateProblem { key: key.as_str().into() });
            }
        }
        self.problems = keys;
        Ok(self)
    }

    /// This descriptor with the left-out reasons that keep an item for good (`LeftOutKind::Kept`): it is in use, or
    /// never a candidate by its nature. Every other left-out reason keeps an item for now. Refused when a key is not
    /// one of the declared problems (`with_problems` comes first) or is listed twice.
    pub fn with_kept(mut self, keys: Vec<TextKey>) -> Result<Self, DescriptorError> {
        let mut seen = BTreeSet::new();
        for key in &keys {
            if !self.problems.contains(key) || !seen.insert(key) {
                return Err(DescriptorError::KeptReason { key: key.as_str().into() });
            }
        }
        self.kept = keys;
        Ok(self)
    }

    /// This descriptor with the choices of its page, in the order given. Refused when a name is not kebab case or is
    /// listed twice, a choice selects no category, or a category is not the module's.
    pub fn with_choices(mut self, choices: Vec<ModuleChoice>) -> Result<Self, DescriptorError> {
        let mut seen = BTreeSet::new();
        for choice in &choices {
            if !is_kebab_case(&choice.name) || !seen.insert(choice.name.as_str()) {
                return Err(DescriptorError::ChoiceName { name: choice.name.clone() });
            }
            if choice.categories.is_empty() {
                return Err(DescriptorError::EmptyChoice { name: choice.name.clone() });
            }
            if let Some(foreign) = choice.categories.iter().find(|id| !self.categories.iter().any(|c| c.id() == **id)) {
                return Err(DescriptorError::ChoiceCategory {
                    name: choice.name.clone(),
                    category: foreign.as_str().into(),
                });
            }
        }
        self.choices = choices;
        Ok(self)
    }

    /// Refused when a category needs a tool that has no check; the registry calls it once the descriptor is built.
    pub fn check_tools(&self) -> Result<(), DescriptorError> {
        match self.needed_tools().find(|tool| !self.tool_checks.iter().any(|check| check.tool() == *tool)) {
            Some(tool) => Err(DescriptorError::MissingToolCheck { tool: tool.as_str().into() }),
            None => Ok(()),
        }
    }

    pub const fn id(&self) -> ModuleId {
        self.id
    }

    /// The group the screens show the module under.
    pub const fn group(&self) -> ModuleGroup {
        self.group
    }

    pub fn categories(&self) -> &[CategoryDescriptor] {
        &self.categories
    }

    pub fn settings(&self) -> &[SettingDescriptor] {
        &self.settings
    }

    pub const fn locales(&self) -> Locales {
        self.locales
    }

    /// The checks of the tools its categories need.
    pub fn tool_checks(&self) -> &[ToolCommand] {
        &self.tool_checks
    }

    /// The keys the module may report as a failure or a left-out reason, in the order it declared them.
    pub fn problems(&self) -> &[TextKey] {
        &self.problems
    }

    /// The left-out reasons that keep an item for good, in the order the module declared them.
    pub fn kept(&self) -> &[TextKey] {
        &self.kept
    }

    /// Whether a left-out for `reason` is kept for good (one of `with_kept`) or for now (any other).
    pub fn left_out_kind(&self, reason: &TextKey) -> LeftOutKind {
        if self.kept.contains(reason) { LeftOutKind::Kept } else { LeftOutKind::ForNow }
    }

    /// The choices of the module's page, in the order it declared them; empty for a module without.
    pub fn choices(&self) -> &[ModuleChoice] {
        &self.choices
    }

    fn needed_tools(&self) -> impl Iterator<Item = ToolId> {
        self.categories.iter().flat_map(CategoryDescriptor::needs).filter_map(|need| match need.capability {
            Capability::Tool(tool) => Some(tool),
            _ => None,
        })
    }
}

fn check_settings(id: ModuleId, settings: &[SettingDescriptor]) -> Result<(), DescriptorError> {
    let mut seen = BTreeSet::new();
    let enabled = format!("{id}.{ENABLED}");
    for setting in settings {
        let key = setting.key();
        if key.namespace() != id.as_str() {
            return Err(DescriptorError::ForeignSetting { key: key.as_str().into(), module: id.as_str().into() });
        }
        if key.as_str() == enabled {
            return Err(DescriptorError::ReservedSetting { key: key.as_str().into() });
        }
        if !seen.insert(key) {
            return Err(DescriptorError::DuplicateSetting { key: key.as_str().into() });
        }
    }
    Ok(())
}

/// Why a module could not finish its work. A module's failure is reported for that module; the others go on.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum ModuleError {
    #[error("the module stopped: the operation was cancelled")]
    Cancelled,
    /// `reason` is one of the problems the module declares (`ModuleDescriptor::with_problems`); a technical detail,
    /// such as a tool's output, goes under the arg `DETAIL_ARG`, which no sentence names.
    #[error("the module failed: {}", .reason.key())]
    Failed { reason: Text },
}

/// Where a module puts its findings, what it left out and what it describes.
pub trait FindingSink {
    fn push(&mut self, finding: NewFinding);

    /// An item the module recognized and chose not to offer; never dropped silently by a module.
    fn left_out(&mut self, left_out: LeftOutItem);

    /// Space the module describes and never offers; a sink that keeps findings only drops it.
    fn describe(&mut self, _insight: Insight) {}
}

/// For tests that look at findings only: it keeps the findings and drops every left-out.
impl FindingSink for Vec<NewFinding> {
    fn push(&mut self, finding: NewFinding) {
        Vec::push(self, finding);
    }

    fn left_out(&mut self, _left_out: LeftOutItem) {}
}

/// The ports a module may use, built by the engine (or the test harness) for each call.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct ModuleContext<'a> {
    clock: &'a dyn Clock,
    commands: &'a dyn CommandRunner,
    system: &'a SystemFacts,
    settings: &'a ModuleSettings,
    cancel: &'a CancelToken,
    walk_complete: bool,
    /// What the walk saw of the place of each of the module's interests, by its index; empty until `finish`.
    places: &'a [PlaceSeen],
}

impl<'a> ModuleContext<'a> {
    pub fn new(
        clock: &'a dyn Clock,
        commands: &'a dyn CommandRunner,
        system: &'a SystemFacts,
        settings: &'a ModuleSettings,
        cancel: &'a CancelToken,
    ) -> Self {
        Self { clock, commands, system, settings, cancel, walk_complete: false, places: &[] }
    }

    /// The same context for `finish` after a walk that read every folder.
    #[must_use]
    pub const fn complete_walk(self) -> Self {
        Self { walk_complete: true, ..self }
    }

    /// Whether the walk read every folder: known only in `finish`, so `false` during the walk, and after one that
    /// could not read a folder or stopped early. A module that decides from every match together (a folder it did
    /// not see may be the one that changed) must not offer then.
    pub const fn is_walk_complete(&self) -> bool {
        self.walk_complete
    }

    /// The same context for `finish`, with what the walk saw of the place of each of the module's interests (by its
    /// index in the module's own `interests()`).
    #[must_use]
    pub const fn with_places(self, places: &'a [PlaceSeen]) -> Self {
        Self { places, ..self }
    }

    /// What the walk saw of the place of the module's interest `interest`: `Unknown` during the walk and for an
    /// interest without one place. A module that decides from every match of a place checks this
    /// rather than [`Self::is_walk_complete`], which any folder the walk could not read makes `false`.
    pub fn place_seen(&self, interest: usize) -> PlaceSeen {
        self.places.get(interest).copied().unwrap_or(PlaceSeen::Unknown)
    }

    pub fn clock(&self) -> &'a dyn Clock {
        self.clock
    }

    pub fn commands(&self) -> &'a dyn CommandRunner {
        self.commands
    }

    /// What the scan knows of the system: known folders, installed and running apps, projects. Shared by every
    /// module and gathered at most once per scan.
    pub fn system(&self) -> &'a SystemFacts {
        self.system
    }

    /// The module's own settings.
    pub fn settings(&self) -> &'a ModuleSettings {
        self.settings
    }

    pub fn cancel(&self) -> &'a CancelToken {
        self.cancel
    }
}

impl std::fmt::Debug for ModuleContext<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModuleContext")
            .field("settings", self.settings)
            .field("cancel", self.cancel)
            .field("walk_complete", &self.walk_complete)
            .field("places", &self.places.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use crate::{
        CategoryDescriptor, CategoryId, Claim, Clock, CommandOutput, CommandRunError, CommandRunner, DangerLevel,
        EntryKind, FindingSink, Interest, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError,
        ModuleId, NewFinding, Pattern, Removal, Sizes, Subject, Text, TextKey,
    };

    const LOCALES: Locales = Locales { en: "{}" };

    fn demo() -> ModuleId {
        ModuleId::new("demo").unwrap()
    }

    fn category(id: &'static str) -> CategoryDescriptor {
        CategoryDescriptor::new(CategoryId::new(demo(), id).unwrap(), DangerLevel::Low)
    }

    pub(super) fn choice(name: &str, categories: &[&'static str]) -> crate::ModuleChoice {
        let ids = categories.iter().map(|id| CategoryId::new(demo(), id).unwrap()).collect();
        crate::ModuleChoice::new(name, ids)
    }

    pub(super) fn with_two_categories() -> ModuleDescriptor {
        let categories = vec![category("demo.caches"), category("demo.logs")];
        ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, categories, vec![], LOCALES).unwrap()
    }

    struct StoppedClock;

    impl Clock for StoppedClock {
        fn now(&self) -> SystemTime {
            SystemTime::UNIX_EPOCH
        }
    }

    struct NoCommands;

    impl CommandRunner for NoCommands {
        fn run(
            &self,
            program: &str,
            _args: &[&str],
            _cancel: &crate::CancelToken,
        ) -> Result<CommandOutput, CommandRunError> {
            Err(CommandRunError::NotInstalled { program: program.into() })
        }
    }

    /// Turns every match into a finding, and fails in `finish` when the operation was cancelled.
    struct EchoModule {
        descriptor: ModuleDescriptor,
        interests: Vec<Interest>,
    }

    impl EchoModule {
        fn new() -> Self {
            let descriptor = ModuleDescriptor::new(
                demo(),
                crate::ModuleGroup::Debug,
                vec![category("demo.caches")],
                vec![],
                LOCALES,
            )
            .unwrap();
            let interest = Interest::new(Pattern::FolderName("Caches".into()), Claim::Whole, vec![]).unwrap();
            Self { descriptor, interests: vec![interest] }
        }
    }

    impl Module for EchoModule {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn interests(&self) -> &[Interest] {
            &self.interests
        }

        fn on_match(
            &self,
            matched: &Match,
            _cx: &ModuleContext<'_>,
            out: &mut dyn FindingSink,
        ) -> Result<(), ModuleError> {
            let text = |key: &str| Text::new(TextKey::new(key).unwrap());
            out.push(NewFinding::new(
                &self.descriptor.categories()[0],
                matched.path.to_string_lossy(),
                Subject::Path(matched.path.clone()),
                matched.sizes,
                text("demo.item.title"),
                text("demo.item.reason"),
                Removal::MoveToTrash(matched.path.clone()),
            ));
            Ok(())
        }

        fn finish(&self, cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
            if cx.cancel().is_cancelled() { Err(ModuleError::Cancelled) } else { Ok(()) }
        }
    }

    fn matched() -> Match {
        Match {
            interest: 0,
            path: "/home/Library/Caches".into(),
            kind: EntryKind::Folder,
            sizes: Sizes::ZERO,
            modified: None,
            changed: None,
            accessed: None,
            inside: None,
            siblings: vec![],
            holds: Vec::new(),
            marker: None,
            facts: crate::EntryFacts::REMOVABLE,
        }
    }

    mod negative {
        use super::{EchoModule, LOCALES, NoCommands, StoppedClock, category, choice, demo, with_two_categories};
        use crate::{
            CancelToken, CategoryDescriptor, CategoryId, DangerLevel, DescriptorError, Module, ModuleContext,
            ModuleDescriptor, ModuleError, ModuleId,
        };

        #[test]
        fn rejects_a_descriptor_with_duplicate_category_ids() {
            let categories = vec![category("demo.caches"), category("demo.logs"), category("demo.caches")];

            let error = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, categories, vec![], LOCALES);

            assert_eq!(error, Err(DescriptorError::DuplicateCategory { id: "demo.caches".into() }));
        }

        #[test]
        fn rejects_a_category_of_another_module() {
            let docker = ModuleId::new("docker").unwrap();
            let image = CategoryDescriptor::new(CategoryId::new(docker, "docker.images").unwrap(), DangerLevel::Low);

            let error = ModuleDescriptor::new(
                demo(),
                crate::ModuleGroup::Debug,
                vec![category("demo.caches"), image],
                vec![],
                LOCALES,
            );

            assert_eq!(
                error,
                Err(DescriptorError::ForeignCategory { id: "docker.images".into(), module: "demo".into() })
            );
        }

        fn setting(key: &str) -> crate::SettingDescriptor {
            let key = crate::SettingKey::new(key).unwrap();
            crate::SettingDescriptor::new(key, crate::SettingKind::Switch, crate::SettingValue::Switch(true)).unwrap()
        }

        #[test]
        fn rejects_a_module_setting_outside_its_namespace() {
            let error = ModuleDescriptor::new(
                demo(),
                crate::ModuleGroup::Debug,
                vec![],
                vec![setting("core.scan.deep")],
                LOCALES,
            );

            assert_eq!(
                error,
                Err(DescriptorError::ForeignSetting { key: "core.scan.deep".into(), module: "demo".into() })
            );
        }

        #[test]
        fn rejects_the_enabled_switch_and_a_duplicate_setting() {
            let reserved = ModuleDescriptor::new(
                demo(),
                crate::ModuleGroup::Debug,
                vec![],
                vec![setting("demo.enabled")],
                LOCALES,
            );
            let twice = ModuleDescriptor::new(
                demo(),
                crate::ModuleGroup::Debug,
                vec![],
                vec![setting("demo.deep"), setting("demo.deep")],
                LOCALES,
            );

            assert_eq!(reserved, Err(DescriptorError::ReservedSetting { key: "demo.enabled".into() }));
            assert_eq!(twice, Err(DescriptorError::DuplicateSetting { key: "demo.deep".into() }));
        }

        fn docker_check() -> crate::ToolCommand {
            let docker = crate::ToolId::new("docker").unwrap();
            crate::ToolCommand::new(docker, vec!["info".into()]).unwrap()
        }

        fn needing_docker() -> CategoryDescriptor {
            let docker = crate::Capability::Tool(crate::ToolId::new("docker").unwrap());
            category("demo.images").with_needs(vec![crate::Need::required(docker)])
        }

        #[test]
        fn a_tool_need_without_its_check_is_refused() {
            let descriptor =
                ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![needing_docker()], vec![], LOCALES)
                    .unwrap();

            assert_eq!(descriptor.check_tools(), Err(DescriptorError::MissingToolCheck { tool: "docker".into() }));
        }

        #[test]
        fn a_tool_check_without_a_need_is_refused() {
            let descriptor = ModuleDescriptor::new(
                demo(),
                crate::ModuleGroup::Debug,
                vec![category("demo.caches")],
                vec![],
                LOCALES,
            )
            .unwrap();

            let error = descriptor.with_tool_check(docker_check());

            assert_eq!(error, Err(DescriptorError::UnusedToolCheck { tool: "docker".into() }));
        }

        #[test]
        fn a_second_check_of_the_same_tool_is_refused() {
            let descriptor =
                ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![needing_docker()], vec![], LOCALES)
                    .unwrap();

            let error = descriptor.with_tool_check(docker_check()).unwrap().with_tool_check(docker_check());

            assert_eq!(error, Err(DescriptorError::DuplicateToolCheck { tool: "docker".into() }));
        }

        fn key(key: &str) -> crate::TextKey {
            crate::TextKey::new(key).unwrap()
        }

        #[test]
        fn a_problem_outside_the_module_is_refused() {
            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES).unwrap();

            let error = descriptor.with_problems(vec![key("demo.error.broken"), key("core.error.internal")]);

            assert_eq!(
                error,
                Err(DescriptorError::ForeignProblem { key: "core.error.internal".into(), module: "demo".into() })
            );
        }

        #[test]
        fn a_problem_listed_twice_is_refused() {
            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES).unwrap();

            let error = descriptor.with_problems(vec![key("demo.left-out.open"), key("demo.left-out.open")]);

            assert_eq!(error, Err(DescriptorError::DuplicateProblem { key: "demo.left-out.open".into() }));
        }

        #[test]
        fn a_kept_reason_that_is_not_declared_or_is_listed_twice_is_refused() {
            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES)
                .unwrap()
                .with_problems(vec![key("demo.left-out.open")])
                .unwrap();

            let undeclared = descriptor.clone().with_kept(vec![key("demo.left-out.current")]);
            let twice = descriptor.with_kept(vec![key("demo.left-out.open"), key("demo.left-out.open")]);

            assert_eq!(undeclared, Err(DescriptorError::KeptReason { key: "demo.left-out.current".into() }));
            assert_eq!(twice, Err(DescriptorError::KeptReason { key: "demo.left-out.open".into() }));
        }

        #[test]
        fn a_reason_not_kept_for_good_is_kept_for_now() {
            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES)
                .unwrap()
                .with_problems(vec![key("demo.left-out.open"), key("demo.left-out.current")])
                .unwrap()
                .with_kept(vec![key("demo.left-out.current")])
                .unwrap();

            assert_eq!(descriptor.left_out_kind(&key("demo.left-out.open")), crate::LeftOutKind::ForNow);
            assert_eq!(descriptor.left_out_kind(&key("demo.left-out.unknown")), crate::LeftOutKind::ForNow);
            assert_eq!(descriptor.left_out_kind(&key("demo.left-out.current")), crate::LeftOutKind::Kept);
        }

        #[test]
        fn a_choice_name_that_is_not_kebab_case_is_refused() {
            for name in ["Keep", "keep_settings", ""] {
                let error = with_two_categories().with_choices(vec![choice(name, &["demo.caches"])]);

                assert_eq!(error, Err(DescriptorError::ChoiceName { name: name.into() }));
            }
        }

        #[test]
        fn a_choice_listed_twice_is_refused() {
            let both = vec![choice("keep", &["demo.caches"]), choice("keep", &["demo.logs"])];

            let error = with_two_categories().with_choices(both);

            assert_eq!(error, Err(DescriptorError::ChoiceName { name: "keep".into() }));
        }

        #[test]
        fn a_choice_without_a_category_is_refused() {
            let error = with_two_categories().with_choices(vec![choice("keep", &[])]);

            assert_eq!(error, Err(DescriptorError::EmptyChoice { name: "keep".into() }));
        }

        #[test]
        fn a_choice_of_a_category_the_module_does_not_have_is_refused() {
            let other = crate::ModuleChoice::new(
                "keep",
                vec![CategoryId::new(ModuleId::new("other").unwrap(), "other.caches").unwrap()],
            );

            let error = with_two_categories().with_choices(vec![other]);

            assert_eq!(
                error,
                Err(DescriptorError::ChoiceCategory { name: "keep".into(), category: "other.caches".into() })
            );
        }

        #[test]
        fn a_vec_sink_keeps_no_left_out() {
            let mut found: Vec<crate::NewFinding> = Vec::new();
            let category = CategoryId::new(demo(), "demo.caches").unwrap();

            crate::FindingSink::left_out(
                &mut found,
                crate::LeftOutItem {
                    category,
                    reason: key("demo.left-out.open"),
                    sizes: crate::Sizes::ZERO,
                    path: "/a".into(),
                    title: crate::Text::new(key("demo.finding.title")),
                },
            );

            assert_eq!(found, []);
        }

        #[test]
        fn a_place_seen_is_unknown_without_the_places_and_out_of_range() {
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            let system = crate::system::nothing_known();
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &system, &settings, &cancel);
            let places = [crate::PlaceSeen::Read];

            assert_eq!(cx.place_seen(0), crate::PlaceSeen::Unknown);
            assert_eq!(cx.with_places(&places).place_seen(1), crate::PlaceSeen::Unknown);
        }

        #[test]
        fn a_cancelled_module_reports_it() {
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            cancel.cancel();
            let system = crate::system::nothing_known();
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &system, &settings, &cancel);

            let error = EchoModule::new().finish(&cx, &mut Vec::new());

            assert_eq!(error, Err(ModuleError::Cancelled));
            assert_eq!(ModuleError::Cancelled.to_string(), "the module stopped: the operation was cancelled");
        }
    }

    mod positive {
        use super::{EchoModule, LOCALES, NoCommands, StoppedClock, choice, demo, matched, with_two_categories};
        use crate::{CancelToken, Module, ModuleContext, ModuleDescriptor, ModuleError, NewFinding, Text, TextKey};

        #[test]
        fn a_module_without_interests_is_valid() {
            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES).unwrap();
            let module = EchoModule { descriptor, interests: vec![] };
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            let system = crate::system::nothing_known();
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &system, &settings, &cancel);

            assert_eq!(module.finish(&cx, &mut Vec::new()), Ok(()));
            assert_eq!(module.interests(), []);
            assert_eq!(module.descriptor().id(), demo());
            assert_eq!(module.descriptor().categories(), []);
            assert_eq!(module.descriptor().locales().en, "{}");
            assert_eq!(module.descriptor().settings(), []);
            assert_eq!(cx.settings().module(), demo());
            assert!(!cx.is_walk_complete());
            assert!(cx.complete_walk().is_walk_complete());
        }

        #[test]
        fn a_place_seen_is_the_state_given_for_the_interest() {
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            let system = crate::system::nothing_known();
            let places = [crate::PlaceSeen::Read, crate::PlaceSeen::Unread];
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &system, &settings, &cancel).with_places(&places);

            assert_eq!(cx.place_seen(0), crate::PlaceSeen::Read);
            assert_eq!(cx.place_seen(1), crate::PlaceSeen::Unread);
            assert_eq!(cx.complete_walk().place_seen(1), crate::PlaceSeen::Unread);
            assert!(format!("{cx:?}").contains("places: 2"));
        }

        #[test]
        fn a_boxed_module_turns_matches_into_findings() {
            let module: Box<dyn Module> = Box::new(EchoModule::new());
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            let system = crate::system::nothing_known();
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &system, &settings, &cancel);
            let mut found: Vec<NewFinding> = Vec::new();

            module.on_match(&matched(), &cx, &mut found).unwrap();
            module.finish(&cx, &mut found).unwrap();

            assert_eq!(module.descriptor().id(), demo());
            assert_eq!(module.interests().len(), 1);
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].key().identity(), "/home/Library/Caches");
            assert_eq!(cx.clock().now(), std::time::SystemTime::UNIX_EPOCH);
            assert!(cx.commands().run("docker", &[], cx.cancel()).is_err());
            assert!(format!("{cx:?}").starts_with("ModuleContext"));
        }

        #[test]
        fn a_needed_tool_with_its_check_is_accepted() {
            let docker = crate::ToolId::new("docker").unwrap();
            let need = crate::Need::required(crate::Capability::Tool(docker));
            let images = super::category("demo.images").with_needs(vec![need]);
            let check = crate::ToolCommand::new(docker, vec!["info".into()]).unwrap();

            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![images], vec![], LOCALES)
                .unwrap()
                .with_tool_check(check.clone());

            let descriptor = descriptor.unwrap();
            assert_eq!(descriptor.check_tools(), Ok(()));
            assert_eq!(descriptor.tool_checks(), [check]);
        }

        #[test]
        fn a_descriptor_keeps_its_problems_in_order() {
            let keys = ["demo.left-out.open", "demo.error.broken"].map(|key| TextKey::new(key).unwrap());

            let descriptor = ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES)
                .unwrap()
                .with_problems(keys.to_vec())
                .unwrap();

            assert_eq!(descriptor.problems(), keys);
            assert_eq!(
                ModuleDescriptor::new(demo(), crate::ModuleGroup::Debug, vec![], vec![], LOCALES).unwrap().problems(),
                []
            );
        }

        #[test]
        fn a_descriptor_keeps_its_choices_in_order() {
            let choices = vec![choice("everything", &["demo.caches", "demo.logs"]), choice("keep", &["demo.caches"])];

            let descriptor = with_two_categories().with_choices(choices.clone()).unwrap();

            assert_eq!(descriptor.choices(), choices);
            assert_eq!(descriptor.choices()[1].label_key(demo()).as_str(), "demo.choice.keep.label");
            assert_eq!(descriptor.choices()[1].description_key(demo()).as_str(), "demo.choice.keep.description");
            assert_eq!(with_two_categories().choices(), []);
        }

        #[test]
        fn a_failure_names_its_reason() {
            let error = ModuleError::Failed { reason: Text::new(TextKey::new("demo.error.broken").unwrap()) };

            assert_eq!(error.to_string(), "the module failed: demo.error.broken");
        }
    }
}
