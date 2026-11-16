//! The trial module: folders made only to try a cleanup, so a development build has something real to clean.
//!
//! Registered in development builds only. Its rules claim a folder named `fresher-trial-cache` (moved to the Trash)
//! or `fresher-trial-kept` (emptied, kept) only when the sandbox marker lies right next to it; the cleaner checks the
//! marker's signature again before it removes anything. In `finish` it lists the objects of a made-up tool,
//! `fresher-trial-tool`, which only the fake command runner of the tests answers.

#![forbid(unsafe_code)]

mod tool;

use fresher_core::{
    Capability, CategoryDescriptor, CategoryId, DangerLevel, DescriptorError, FindingSink, IdError, Interest, Locales,
    Match, Module, ModuleContext, ModuleDescriptor, ModuleError, ModuleId, Need, RemovalError, Text, TextKey,
    TextKeyError, ToolCommand, ToolId,
};
use fresher_rules::{RuleError, RuleSet};

/// The made-up tool whose objects the `trial.tool` category lists.
pub const TRIAL_TOOL: &str = "fresher-trial-tool";

/// Why the trial module could not be built (a bug in its own constants or rules).
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum TrialError {
    #[error(transparent)]
    Id(#[from] IdError),
    #[error(transparent)]
    TextKey(#[from] TextKeyError),
    #[error(transparent)]
    Descriptor(#[from] DescriptorError),
    #[error(transparent)]
    Removal(#[from] RemovalError),
    #[error(transparent)]
    Rules(#[from] RuleError),
}

/// The `trial.tool` category, by index into the descriptor's list.
const TOOL: usize = 2;

/// The trial module.
#[derive(Debug)]
pub struct TrialModule {
    descriptor: ModuleDescriptor,
    rules: RuleSet,
    interests: Vec<Interest>,
    tool: ToolId,
    title: TextKey,
    reason: Text,
    failed: Text,
}

impl TrialModule {
    pub fn new() -> Result<Self, TrialError> {
        let id = ModuleId::new("trial")?;
        let tool = ToolId::new(TRIAL_TOOL)?;
        let category = |name, danger| Ok::<_, IdError>(CategoryDescriptor::new(CategoryId::new(id, name)?, danger));
        let categories = vec![
            category("trial.caches", DangerLevel::Safe)?,
            category("trial.kept", DangerLevel::Low)?,
            category("trial.tool", DangerLevel::Medium)?.with_needs(vec![Need::required(Capability::Tool(tool))]),
        ];
        let locales = Locales { en: include_str!("../locales/en.json") };
        let descriptor = ModuleDescriptor::new(id, categories, vec![], locales)?
            .with_tool_check(ToolCommand::new(tool, tool::check_args())?)?;
        let rules = RuleSet::parse(&descriptor, include_str!("../rules.toml"))?;
        Ok(Self {
            interests: rules.interests(),
            rules,
            descriptor,
            tool,
            title: TextKey::new("trial.object.title")?,
            reason: Text::new(TextKey::new("trial.object.reason")?),
            failed: Text::new(TextKey::new("trial.error.failed")?),
        })
    }
}

impl Module for TrialModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn interests(&self) -> &[Interest] {
        &self.interests
    }

    fn on_match(&self, matched: &Match, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if let Some(finding) = self.rules.evaluate(matched, cx) {
            out.push(finding);
        }
        Ok(())
    }

    fn finish(&self, cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
        if cx.cancel().is_cancelled() {
            return Err(ModuleError::Cancelled);
        }
        let failed = || ModuleError::Failed { reason: self.failed.clone() };
        let Some(listing) = tool::list(self.tool, cx.commands(), cx.cancel()).map_err(|_| failed())? else {
            return Ok(());
        };
        let category = &self.descriptor.categories()[TOOL];
        for object in listing {
            out.push(object.finding(self.tool, category, &self.title, &self.reason).map_err(|_| failed())?);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{Interest, Module, Pattern};

        use crate::TrialModule;

        #[test]
        fn every_rule_names_a_folder_only_a_developer_makes() {
            let module = TrialModule::new().unwrap();

            let names: Vec<&Pattern> = module.interests().iter().map(Interest::pattern).collect();

            assert_eq!(
                names,
                [&Pattern::FolderName("fresher-trial-cache".into()), &Pattern::FolderName("fresher-trial-kept".into())]
            );
            assert!(module.interests().iter().all(|interest| interest.siblings() == [".fresher-sandbox"]));
        }
    }

    mod positive {
        use fresher_core::{Capability, CategoryDescriptor, DangerLevel, Module, NeedKind, SANDBOX_MARKER};

        use crate::{TRIAL_TOOL, TrialModule};

        #[test]
        fn the_categories_keep_their_danger_levels_and_the_tool_has_its_check() {
            let module = TrialModule::new().unwrap();
            let descriptor = module.descriptor();

            let levels: Vec<_> = descriptor.categories().iter().map(CategoryDescriptor::danger).collect();
            assert_eq!(levels, [DangerLevel::Safe, DangerLevel::Low, DangerLevel::Medium]);
            let needs = descriptor.categories()[2].needs();
            assert!(matches!(needs, [need] if need.kind == NeedKind::Required
                && matches!(need.capability, Capability::Tool(tool) if tool.as_str() == TRIAL_TOOL)));
            assert_eq!(descriptor.check_tools(), Ok(()));
            assert_eq!(SANDBOX_MARKER, ".fresher-sandbox");
        }
    }
}
