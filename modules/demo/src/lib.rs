//! The demo module: made-up findings that prove the module contract end to end, before any real scan exists.
//!
//! It has no interests, reads nothing from the disk and runs no command: `finish` describes example items under a
//! made-up `/demo/` root that does not exist, and example images and volumes of a made-up tool, and leaves out one
//! made-up cache so the line of a left-out item can be seen.

#![forbid(unsafe_code)]

mod settings;

use fresher_core::{
    ByteSize, CategoryDescriptor, CategoryId, Completeness, DangerLevel, DescriptorError, FindingSink, IdError,
    Interest, LeftOutItem, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleGroup, ModuleId,
    NewFinding, Removal, RemovalError, SettingError, Sizes, Subject, Text, TextArg, TextKey, TextKeyError, ToolCommand,
    ToolId,
};

/// The made-up root every example path is under.
pub const DEMO_ROOT: &str = "/demo";

/// Why the demo module could not be built (a bug in its own constants).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum DemoError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Setting(#[from] SettingError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
}

/// The categories, by index into the descriptor's list.
const CACHES: usize = 0;
const DOWNLOADS: usize = 1;
const IMAGES: usize = 2;
const VOLUMES: usize = 3;

/// How an example item would be removed.
#[derive(Clone, Copy, Debug)]
enum Kind {
    /// A file or folder under the made-up root, moved to the Trash.
    Trash(&'static str),
    /// A cache folder under the made-up root, emptied but kept.
    EmptyFolder(&'static str),
    /// An item of the made-up tool, removed by a command of that tool.
    Tool,
}

/// One example finding; past the last one, the list repeats with a number added to every name.
#[derive(Clone, Copy, Debug)]
struct Example {
    category: usize,
    name: &'static str,
    kind: Kind,
    bytes: u64,
    completeness: Completeness,
    raise: Option<DangerLevel>,
}

const EXAMPLES: [Example; 8] = [
    example(CACHES, "com.example.editor", Kind::EmptyFolder("Library/Caches"), 734_003_200),
    example(DOWNLOADS, "example-installer.dmg", Kind::Trash("Downloads"), 268_435_456),
    example(IMAGES, "example-image", Kind::Tool, 1_288_490_188),
    example(VOLUMES, "example-volume", Kind::Tool, 536_870_912),
    Example {
        completeness: Completeness::Partial,
        ..example(CACHES, "com.example.browser", Kind::EmptyFolder("Library/Caches"), 2_147_483_648)
    },
    example(DOWNLOADS, "example-archive.zip", Kind::Trash("Downloads"), 52_428_800),
    example(IMAGES, "example-builder", Kind::Tool, 805_306_368),
    Example {
        raise: Some(DangerLevel::Medium),
        ..example(CACHES, "com.example.music", Kind::Trash("Library/Caches"), 3_221_225_472)
    },
];

/// The size of the one made-up cache the demo leaves out in every scan.
const LEFT_OUT_BYTES: u64 = 157_286_400;

const fn example(category: usize, name: &'static str, kind: Kind, bytes: u64) -> Example {
    Example { category, name, kind, bytes, completeness: Completeness::Complete, raise: None }
}

/// The demo module.
#[derive(Debug)]
pub struct DemoModule {
    descriptor: ModuleDescriptor,
    tool: ToolId,
    title: TextKey,
    reason: Text,
    failed: Text,
    left_out: TextKey,
}

impl DemoModule {
    pub fn new() -> Result<Self, DemoError> {
        let id = ModuleId::new("demo")?;
        let category = |name, danger| Ok::<_, IdError>(CategoryDescriptor::new(CategoryId::new(id, name)?, danger));
        let categories = vec![
            category("demo.caches", DangerLevel::Safe)?,
            category("demo.downloads", DangerLevel::Low)?,
            category("demo.images", DangerLevel::Medium)?,
            category("demo.volumes", DangerLevel::High)?,
        ];
        let settings = vec![settings::count_setting(id)?];
        let locales = Locales { en: include_str!("../locales/en.json") };
        let (failed, left_out) = (TextKey::new("demo.error.failed")?, TextKey::new("demo.left-out.open")?);
        let descriptor = ModuleDescriptor::new(id, ModuleGroup::Debug, categories, settings, locales)?
            .with_problems(vec![failed.clone(), left_out.clone()])?;
        Ok(Self {
            descriptor,
            tool: ToolId::new("demo-tool")?,
            title: TextKey::new("demo.finding.title")?,
            reason: Text::new(TextKey::new("demo.finding.reason")?),
            failed: Text::new(failed),
            left_out,
        })
    }

    /// The `index`-th example finding.
    fn finding(&self, index: usize) -> Result<NewFinding, RemovalError> {
        let example = EXAMPLES[index % EXAMPLES.len()];
        let copy = index / EXAMPLES.len();
        let name = if copy == 0 { example.name.to_owned() } else { format!("{}-{}", example.name, copy + 1) };
        let (subject, removal) = match example.kind {
            Kind::Trash(folder) => {
                let path = [DEMO_ROOT, folder, &name].iter().collect::<std::path::PathBuf>();
                (Subject::Path(path.clone()), Removal::MoveToTrash(path))
            }
            Kind::EmptyFolder(folder) => {
                let path = [DEMO_ROOT, folder, &name].iter().collect::<std::path::PathBuf>();
                (Subject::Path(path.clone()), Removal::EmptyFolder(path))
            }
            Kind::Tool => {
                let command = ToolCommand::new(self.tool, vec!["remove".into(), name.clone()])?;
                (Subject::External { tool: self.tool, id: name.clone() }, Removal::ToolCommand(command))
            }
        };
        let category = &self.descriptor.categories()[example.category];
        let bytes = ByteSize::new(example.bytes);
        let sizes = Sizes { completeness: example.completeness, ..Sizes::complete(bytes, bytes) };
        let title = Text::new(self.title.clone()).with("name", TextArg::Text(name.clone()));
        let mut finding = NewFinding::new(category, name, subject, sizes, title, self.reason.clone(), removal);
        if let Some(level) = example.raise {
            finding.raise_danger(level);
        }
        Ok(finding)
    }

    /// The one made-up cache the demo recognizes and does not offer, as if the app that made it were open.
    fn left_out(&self) -> LeftOutItem {
        let bytes = ByteSize::new(LEFT_OUT_BYTES);
        let category = self.descriptor.categories()[CACHES].id();
        LeftOutItem { category, reason: self.left_out.clone(), sizes: Sizes::complete(bytes, bytes) }
    }
}

impl Module for DemoModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &[]
    }

    fn on_match(
        &self,
        _matched: &Match,
        _cx: &ModuleContext<'_>,
        _out: &mut dyn FindingSink,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        for index in 0..settings::count(cx.settings()) {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let finding = self.finding(index).map_err(|_| ModuleError::Failed { reason: self.failed.clone() })?;
            out.push(finding);
        }
        out.left_out(self.left_out());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use fresher_core::{
        CancelToken, Module, ModuleContext, ModuleError, ModuleSettings, NewFinding, SettingsSchema, StoredSettings,
    };
    use fresher_testing::{FakeClock, FakeCommandRunner};

    use crate::DemoModule;

    /// The demo module's settings with `demo.count` stored as `count`, or the defaults.
    fn settings(module: &DemoModule, count: Option<i64>) -> ModuleSettings {
        let schema = SettingsSchema::new([module.descriptor()]).unwrap();
        let values = count.map(|count| ("demo.count".to_owned(), serde_json::json!(count))).into_iter().collect();
        schema.resolve(&StoredSettings { values }).module(module.descriptor().id())
    }

    /// Runs `finish` with `count` and gives what it found, and the commands it ran.
    fn run(count: Option<i64>, cancel: &CancelToken) -> (Result<Vec<NewFinding>, ModuleError>, Vec<String>) {
        let module = DemoModule::new().unwrap();
        let (clock, commands) = (FakeClock::at(0), FakeCommandRunner::default());
        let settings = settings(&module, count);
        let system = fresher_core::SystemFacts::new(&fresher_testing::fake_system(std::path::Path::new("/home")));
        let cx = ModuleContext::new(&clock, &commands, &system, &settings, cancel);
        let mut found = Vec::new();
        let result = module.finish(&cx, &mut found).map(|()| found);
        (result, commands.calls())
    }

    fn findings(count: Option<i64>) -> Vec<NewFinding> {
        run(count, &CancelToken::new()).0.unwrap()
    }

    mod negative {
        use std::path::{Component, Path};

        use fresher_core::{CancelToken, Module, ModuleError, Removal, Subject};

        use super::{findings, run};
        use crate::{DEMO_ROOT, DemoModule};

        #[test]
        fn reads_nothing_from_the_disk() {
            let module = DemoModule::new().unwrap();

            let (found, calls) = run(None, &CancelToken::new());

            assert_eq!(module.interests(), &[] as &[fresher_core::Interest]);
            assert_eq!(found.unwrap().len(), 8);
            assert_eq!(calls, Vec::<String>::new());
        }

        #[test]
        fn paths_are_under_the_made_up_root_only() {
            for finding in findings(Some(50)) {
                let mut paths: Vec<&Path> = Vec::new();
                if let Subject::Path(path) = finding.subject() {
                    paths.push(path);
                }
                if let Removal::MoveToTrash(path) | Removal::EmptyFolder(path) = finding.removal() {
                    paths.push(path);
                }
                for path in paths {
                    assert!(path.starts_with(DEMO_ROOT), "{} is outside {DEMO_ROOT}", path.display());
                    assert!(path.components().all(|part| matches!(part, Component::Normal(_) | Component::RootDir)));
                }
            }
        }

        #[test]
        fn never_lowers_a_category_danger_level() {
            let module = DemoModule::new().unwrap();
            let categories = module.descriptor().categories();

            for finding in findings(Some(50)) {
                let category = categories.iter().find(|category| category.id() == finding.category()).unwrap();
                assert!(finding.danger() >= category.danger(), "{:?} is below its category", finding.key());
            }
        }

        #[test]
        fn stops_when_the_scan_is_cancelled() {
            let cancel = CancelToken::new();
            cancel.cancel();

            let (found, _) = run(None, &cancel);

            assert_eq!(found, Err(ModuleError::Cancelled));
        }

        #[test]
        fn a_count_out_of_range_falls_back_to_the_default() {
            assert_eq!(findings(Some(0)).len(), 8);
            assert_eq!(findings(Some(51)).len(), 8);
        }
    }

    mod positive {
        use std::collections::BTreeSet;

        use fresher_core::{
            CancelToken, Completeness, DangerLevel, EntryFacts, EntryKind, Match, Module, ModuleContext, NewFinding,
            Removal, Sizes, Subject,
        };
        use fresher_testing::{FakeClock, FakeCommandRunner};

        use super::{findings, settings};
        use crate::DemoModule;

        #[test]
        fn produces_one_finding_per_danger_level() {
            let found = findings(None);

            let dangers: BTreeSet<_> = found.iter().map(NewFinding::danger).collect();
            assert_eq!(dangers, DangerLevel::ALL.into_iter().collect());
            assert!(found.iter().any(|finding| finding.sizes().completeness == Completeness::Partial));
            assert!(found.iter().any(|finding| matches!(finding.subject(), Subject::External { .. })
                && matches!(finding.removal(), Removal::ToolCommand(_))));
            assert!(found.iter().any(|finding| matches!(finding.removal(), Removal::EmptyFolder(_))));
            assert!(found.iter().any(|finding| matches!(finding.removal(), Removal::MoveToTrash(_))));
        }

        #[test]
        fn honours_the_count_setting() {
            let one = findings(Some(1));
            let fifty = findings(Some(50));

            assert_eq!(one.len(), 1);
            assert_eq!(fifty.len(), 50);
            let keys: BTreeSet<_> = fifty.iter().map(|finding| finding.key().clone()).collect();
            assert_eq!(keys.len(), 50);
        }

        #[test]
        fn on_match_finds_nothing() {
            let module = DemoModule::new().unwrap();
            let (clock, commands) = (FakeClock::at(0), FakeCommandRunner::default());
            let settings = settings(&module, None);
            let cancel = CancelToken::new();
            let system = fresher_core::SystemFacts::new(&fresher_testing::fake_system(std::path::Path::new("/home")));
            let cx = ModuleContext::new(&clock, &commands, &system, &settings, &cancel);
            let matched = Match {
                interest: 0,
                path: "/demo/x".into(),
                kind: EntryKind::File,
                sizes: Sizes::ZERO,
                modified: None,
                changed: None,
                accessed: None,
                inside: None,
                siblings: vec![],
                marker: None,
                facts: EntryFacts::REMOVABLE,
            };
            let mut found = Vec::new();

            module.on_match(&matched, &cx, &mut found).unwrap();

            assert_eq!(found, Vec::<NewFinding>::new());
        }
    }
}
