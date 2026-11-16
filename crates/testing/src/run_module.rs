use std::path::{Path, PathBuf};

use fresher_core::{
    CancelToken, DiskWalk, InterestSet, Match, Module, ModuleContext, ModuleError, ModuleOutcome, NewFinding,
    SchemaError, SettingsSchema, StoredSettings, WalkError, WalkMode, WalkRequest,
};
use serde_json::{Value, json};

use crate::{FakeClock, FakeCommandRunner, FixtureTree, NaiveWalk};

/// The ports a module runs with in a test: a stopped clock, a table of command answers, the stored settings and a
/// cancel token.
#[derive(Debug)]
pub struct Ports {
    pub clock: FakeClock,
    pub commands: FakeCommandRunner,
    /// The stored values, resolved against the module's schema: a missing value takes its default.
    pub settings: StoredSettings,
    pub cancel: CancelToken,
}

impl Ports {
    /// The clock at `seconds` after the Unix epoch, no command answers, every setting at its default.
    pub fn at(seconds: u64) -> Self {
        Self {
            clock: FakeClock::at(seconds),
            commands: FakeCommandRunner::default(),
            settings: StoredSettings::default(),
            cancel: CancelToken::new(),
        }
    }
}

/// What one module did with one fixture tree.
#[derive(Debug)]
pub struct ModuleRun {
    pub findings: Vec<NewFinding>,
    pub outcome: ModuleOutcome,
    root: PathBuf,
}

/// Why the harness could not run the module at all.
#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("the walk of the fixture tree failed: {0}")]
    Walk(#[from] WalkError),
    #[error("the module's settings schema is not valid: {0}")]
    Schema(#[from] SchemaError),
}

/// Runs `module` as the engine would, against the fixture tree as the home folder and only root: walks it with
/// [`NaiveWalk`], sends each match to the module (indexed in the module's own interests), then calls `finish`. A
/// module's error ends its run with that outcome; a cancelled walk ends it as `Cancelled`.
pub fn run_module(module: &dyn Module, tree: &FixtureTree, ports: &Ports) -> Result<ModuleRun, HarnessError> {
    let id = module.descriptor().id();
    let settings = SettingsSchema::new([module.descriptor()])?.resolve(&ports.settings).module(id);
    let cx = ModuleContext::new(&ports.clock, &ports.commands, &settings, &ports.cancel);
    let mut interests = InterestSet::new();
    interests.add(id, module.interests());
    let mut findings = Vec::new();
    let mut failure = None;
    let mut on_match = |mut matched: Match| {
        if failure.is_some() {
            return;
        }
        if let Some((_, local)) = interests.owner(matched.interest) {
            matched.interest = local;
        }
        failure = module.on_match(&matched, &cx, &mut findings).err();
    };
    let roots = [tree.root().to_path_buf()];
    let request = WalkRequest { roots: &roots, interests: &interests, avoid: &[], mode: WalkMode::Full };
    let walked = NaiveWalk::new(tree.root()).walk(&request, &ports.cancel, &mut on_match, &mut |_| {});
    match walked {
        Ok(_) => {}
        Err(WalkError::Cancelled) => failure = failure.or(Some(ModuleError::Cancelled)),
        Err(error) => return Err(error.into()),
    }
    if failure.is_none() {
        failure = module.finish(&cx, &mut findings).err();
    }
    let outcome = match failure {
        None => ModuleOutcome::Completed,
        Some(ModuleError::Cancelled) => ModuleOutcome::Cancelled,
        Some(ModuleError::Failed { reason }) => ModuleOutcome::Failed { reason },
    };
    Ok(ModuleRun { findings, outcome, root: tree.root().to_path_buf() })
}

impl ModuleRun {
    /// A stable JSON report of the run: the outcome and the findings sorted by identity, every path relative to the
    /// fixture root (the root itself is `.`), pretty-printed with a final newline.
    pub fn golden(&self) -> Result<String, serde_json::Error> {
        let mut findings: Vec<&NewFinding> = self.findings.iter().collect();
        findings.sort_by(|left, right| left.key().cmp(right.key()));
        let mut report = json!({ "outcome": self.outcome, "findings": findings });
        relative_paths(&mut report, &self.root);
        Ok(serde_json::to_string_pretty(&report)? + "\n")
    }
}

/// Rewrites every string in `value` that is `root` or a path under it as a path relative to `root`.
fn relative_paths(value: &mut Value, root: &Path) {
    match value {
        Value::String(text) => {
            if let Ok(relative) = Path::new(text.as_str()).strip_prefix(root) {
                let relative = relative.to_string_lossy();
                *text = if relative.is_empty() { ".".into() } else { relative.into_owned() };
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| relative_paths(item, root)),
        Value::Object(fields) => fields.values_mut().for_each(|field| relative_paths(field, root)),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

#[cfg(test)]
pub(crate) mod sample {
    use fresher_core::{
        CategoryDescriptor, CategoryId, Claim, DangerLevel, FindingSink, Interest, Locales, Match, Module,
        ModuleContext, ModuleDescriptor, ModuleError, ModuleId, NewFinding, Pattern, Removal, SettingDescriptor,
        Subject, Text, TextArg, TextKey,
    };

    /// The dictionary of [`CacheModule`]: every key it uses, nothing more.
    pub(crate) const EN: &str = r#"{
        "sample.caches.name": "Caches",
        "sample.caches.what": "Files an app keeps to start faster.",
        "sample.caches.made-by": "Many apps.",
        "sample.caches.if-removed": "The app rebuilds them.",
        "sample.caches.comes-back": "Yes, by itself.",
        "sample.enabled.label": "Sample",
        "sample.enabled.description": "Finds every folder named Caches.",
        "sample.item.title": "{name}",
        "sample.item.reason": "A folder named Caches."
    }"#;

    /// A module that claims every folder named `Caches` as a Low-danger finding; built with `failing()`, it fails in
    /// `finish`.
    pub(crate) struct CacheModule {
        descriptor: ModuleDescriptor,
        interests: Vec<Interest>,
        fail_in_finish: bool,
    }

    impl CacheModule {
        pub(crate) fn with_dictionary(en: &'static str) -> Self {
            Self::with_settings(en, vec![])
        }

        pub(crate) fn with_settings(en: &'static str, settings: Vec<SettingDescriptor>) -> Self {
            let id = ModuleId::new("sample").unwrap();
            let category = CategoryDescriptor::new(CategoryId::new(id, "sample.caches").unwrap(), DangerLevel::Low);
            let descriptor = ModuleDescriptor::new(id, vec![category], settings, Locales { en }).unwrap();
            let interest = Interest::new(Pattern::FolderName("Caches".into()), Claim::Whole, vec![]).unwrap();
            Self { descriptor, interests: vec![interest], fail_in_finish: false }
        }

        pub(crate) fn new() -> Self {
            Self::with_dictionary(EN)
        }

        pub(crate) fn failing() -> Self {
            Self { fail_in_finish: true, ..Self::new() }
        }
    }

    impl Module for CacheModule {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn interests(&self) -> &[Interest] {
            &self.interests
        }

        fn on_match(
            &self,
            matched: &Match,
            cx: &ModuleContext<'_>,
            out: &mut dyn FindingSink,
        ) -> Result<(), ModuleError> {
            if cx.cancel().is_cancelled() {
                return Err(ModuleError::Cancelled);
            }
            let text = |key: &str| Text::new(TextKey::new(key).unwrap());
            let name = matched.path.file_name().unwrap().to_string_lossy().into_owned();
            out.push(NewFinding::new(
                &self.descriptor.categories()[0],
                matched.path.to_string_lossy().into_owned(),
                Subject::Path(matched.path.clone()),
                matched.sizes,
                text("sample.item.title").with("name", TextArg::Text(name)),
                text("sample.item.reason"),
                Removal::MoveToTrash(matched.path.clone()),
            ));
            Ok(())
        }

        fn finish(&self, _cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
            if self.fail_in_finish {
                return Err(ModuleError::Failed { reason: Text::new(TextKey::new("sample.error.broken").unwrap()) });
            }
            Ok(())
        }
    }

    /// Two `Caches` folders, the second one 10 bytes.
    pub(crate) const TREE: &str = "[[dir]]\npath = \"a/Caches\"\n[[file]]\npath = \"b/Caches/data\"\nsize = 10\n";
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{ModuleOutcome, WalkError};

        use crate::run_module::sample::{CacheModule, TREE};
        use crate::{FixtureTree, HarnessError, Ports, run_module};

        #[test]
        fn a_failing_module_reports_its_failure() {
            let tree = FixtureTree::from_toml_str(TREE).unwrap();

            let run = run_module(&CacheModule::failing(), &tree, &Ports::at(0)).unwrap();

            assert!(matches!(run.outcome, ModuleOutcome::Failed { .. }));
            assert_eq!(run.findings.len(), 2);
        }

        #[test]
        fn a_cancelled_run_is_cancelled() {
            let tree = FixtureTree::from_toml_str(TREE).unwrap();
            let ports = Ports::at(0);
            ports.cancel.cancel();

            let run = run_module(&CacheModule::new(), &tree, &ports).unwrap();

            assert_eq!(run.outcome, ModuleOutcome::Cancelled);
            assert_eq!(run.findings.len(), 0);
        }

        #[test]
        fn a_walk_error_is_the_harness_error() {
            let error = HarnessError::from(WalkError::Cancelled);

            assert_eq!(error.to_string(), "the walk of the fixture tree failed: the walk was cancelled");
        }
    }

    mod positive {
        use fresher_core::{ByteSize, ModuleOutcome};

        use crate::run_module::sample::{CacheModule, TREE};
        use crate::{FixtureTree, Ports, run_module};

        #[test]
        fn run_module_returns_the_findings_of_a_test_module() {
            let tree = FixtureTree::from_toml_str(TREE).unwrap();

            let run = run_module(&CacheModule::new(), &tree, &Ports::at(0)).unwrap();

            assert_eq!(run.outcome, ModuleOutcome::Completed);
            let identities: Vec<&str> = run.findings.iter().map(|finding| finding.key().identity()).collect();
            let expected = [tree.root().join("a/Caches"), tree.root().join("b/Caches")];
            assert_eq!(identities, expected.iter().map(|path| path.to_str().unwrap()).collect::<Vec<_>>());
            assert_eq!(run.findings[1].sizes().logical, ByteSize::new(10));
        }

        #[test]
        fn the_golden_report_uses_paths_relative_to_the_root() {
            let tree = FixtureTree::from_toml_str("[[dir]]\npath = \"x/Caches\"\n").unwrap();

            let golden = run_module(&CacheModule::new(), &tree, &Ports::at(0)).unwrap().golden().unwrap();

            let root = tree.root().to_string_lossy().into_owned();
            assert!(!golden.contains(&root), "{golden}");
            let report: serde_json::Value = serde_json::from_str(&golden).unwrap();
            assert_eq!(report["outcome"]["kind"], "completed");
            assert_eq!(report["findings"][0]["key"]["identity"], "x/Caches");
            assert_eq!(report["findings"][0]["subject"]["value"], "x/Caches");
            assert_eq!(report["findings"][0]["removal"]["value"], "x/Caches");
            assert!(golden.ends_with("}\n"));
        }

        #[test]
        fn the_root_itself_is_a_dot() {
            let mut value =
                serde_json::json!({ "root": "/tmp/fixture", "other": "/tmp/fixtures/a", "n": [1, null, true] });

            super::super::relative_paths(&mut value, std::path::Path::new("/tmp/fixture"));

            assert_eq!(value, serde_json::json!({ "root": ".", "other": "/tmp/fixtures/a", "n": [1, null, true] }));
        }
    }
}
