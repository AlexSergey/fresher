//! The contract every feature module implements and the engine drives.

use std::collections::BTreeSet;

use crate::settings::schema::ENABLED;
use crate::{
    CancelToken, CategoryDescriptor, Clock, CommandRunner, Interest, Match, ModuleId, ModuleSettings, NewFinding,
    SettingDescriptor, Text,
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

/// Who a module is: its id, its categories, its settings and its texts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDescriptor {
    id: ModuleId,
    categories: Vec<CategoryDescriptor>,
    settings: Vec<SettingDescriptor>,
    locales: Locales,
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
}

impl ModuleDescriptor {
    /// A descriptor whose categories and settings are unique and all in the module's namespace. The settings do not
    /// include `<module>.enabled`: the schema adds it.
    pub fn new(
        id: ModuleId,
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
        Ok(Self { id, categories, settings, locales })
    }

    pub const fn id(&self) -> ModuleId {
        self.id
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
    #[error("the module failed: {}", .reason.key())]
    Failed { reason: Text },
}

/// Where a module puts its findings.
pub trait FindingSink {
    fn push(&mut self, finding: NewFinding);
}

impl FindingSink for Vec<NewFinding> {
    fn push(&mut self, finding: NewFinding) {
        Vec::push(self, finding);
    }
}

/// The ports a module may use, built by the engine (or the test harness) for each call.
#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct ModuleContext<'a> {
    clock: &'a dyn Clock,
    commands: &'a dyn CommandRunner,
    settings: &'a ModuleSettings,
    cancel: &'a CancelToken,
}

impl<'a> ModuleContext<'a> {
    pub fn new(
        clock: &'a dyn Clock,
        commands: &'a dyn CommandRunner,
        settings: &'a ModuleSettings,
        cancel: &'a CancelToken,
    ) -> Self {
        Self { clock, commands, settings, cancel }
    }

    pub fn clock(&self) -> &'a dyn Clock {
        self.clock
    }

    pub fn commands(&self) -> &'a dyn CommandRunner {
        self.commands
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

    struct StoppedClock;

    impl Clock for StoppedClock {
        fn now(&self) -> SystemTime {
            SystemTime::UNIX_EPOCH
        }
    }

    struct NoCommands;

    impl CommandRunner for NoCommands {
        fn run(&self, program: &str, _args: &[&str]) -> Result<CommandOutput, CommandRunError> {
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
            let descriptor = ModuleDescriptor::new(demo(), vec![category("demo.caches")], vec![], LOCALES).unwrap();
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
            siblings: vec![],
            marker: None,
        }
    }

    mod negative {
        use super::{EchoModule, LOCALES, NoCommands, StoppedClock, category, demo};
        use crate::{
            CancelToken, CategoryDescriptor, CategoryId, DangerLevel, DescriptorError, Module, ModuleContext,
            ModuleDescriptor, ModuleError, ModuleId,
        };

        #[test]
        fn rejects_a_descriptor_with_duplicate_category_ids() {
            let categories = vec![category("demo.caches"), category("demo.logs"), category("demo.caches")];

            let error = ModuleDescriptor::new(demo(), categories, vec![], LOCALES);

            assert_eq!(error, Err(DescriptorError::DuplicateCategory { id: "demo.caches".into() }));
        }

        #[test]
        fn rejects_a_category_of_another_module() {
            let docker = ModuleId::new("docker").unwrap();
            let image = CategoryDescriptor::new(CategoryId::new(docker, "docker.images").unwrap(), DangerLevel::Low);

            let error = ModuleDescriptor::new(demo(), vec![category("demo.caches"), image], vec![], LOCALES);

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
            let error = ModuleDescriptor::new(demo(), vec![], vec![setting("core.scan.deep")], LOCALES);

            assert_eq!(
                error,
                Err(DescriptorError::ForeignSetting { key: "core.scan.deep".into(), module: "demo".into() })
            );
        }

        #[test]
        fn rejects_the_enabled_switch_and_a_duplicate_setting() {
            let reserved = ModuleDescriptor::new(demo(), vec![], vec![setting("demo.enabled")], LOCALES);
            let twice =
                ModuleDescriptor::new(demo(), vec![], vec![setting("demo.deep"), setting("demo.deep")], LOCALES);

            assert_eq!(reserved, Err(DescriptorError::ReservedSetting { key: "demo.enabled".into() }));
            assert_eq!(twice, Err(DescriptorError::DuplicateSetting { key: "demo.deep".into() }));
        }

        #[test]
        fn a_cancelled_module_reports_it() {
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            cancel.cancel();
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &settings, &cancel);

            let error = EchoModule::new().finish(&cx, &mut Vec::new());

            assert_eq!(error, Err(ModuleError::Cancelled));
            assert_eq!(ModuleError::Cancelled.to_string(), "the module stopped: the operation was cancelled");
        }
    }

    mod positive {
        use super::{EchoModule, LOCALES, NoCommands, StoppedClock, demo, matched};
        use crate::{CancelToken, Module, ModuleContext, ModuleDescriptor, ModuleError, NewFinding, Text, TextKey};

        #[test]
        fn a_module_without_interests_is_valid() {
            let descriptor = ModuleDescriptor::new(demo(), vec![], vec![], LOCALES).unwrap();
            let module = EchoModule { descriptor, interests: vec![] };
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &settings, &cancel);

            assert_eq!(module.finish(&cx, &mut Vec::new()), Ok(()));
            assert_eq!(module.interests(), []);
            assert_eq!(module.descriptor().id(), demo());
            assert_eq!(module.descriptor().categories(), []);
            assert_eq!(module.descriptor().locales().en, "{}");
            assert_eq!(module.descriptor().settings(), []);
            assert_eq!(cx.settings().module(), demo());
        }

        #[test]
        fn a_boxed_module_turns_matches_into_findings() {
            let module: Box<dyn Module> = Box::new(EchoModule::new());
            let cancel = CancelToken::new();
            let settings = crate::ModuleSettings::empty(demo());
            let cx = ModuleContext::new(&StoppedClock, &NoCommands, &settings, &cancel);
            let mut found: Vec<NewFinding> = Vec::new();

            module.on_match(&matched(), &cx, &mut found).unwrap();
            module.finish(&cx, &mut found).unwrap();

            assert_eq!(module.descriptor().id(), demo());
            assert_eq!(module.interests().len(), 1);
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].key().identity(), "/home/Library/Caches");
            assert_eq!(cx.clock().now(), std::time::SystemTime::UNIX_EPOCH);
            assert!(cx.commands().run("docker", &[]).is_err());
            assert!(format!("{cx:?}").starts_with("ModuleContext"));
        }

        #[test]
        fn a_failure_names_its_reason() {
            let error = ModuleError::Failed { reason: Text::new(TextKey::new("demo.error.broken").unwrap()) };

            assert_eq!(error.to_string(), "the module failed: demo.error.broken");
        }
    }
}
