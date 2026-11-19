use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fresher_core::{
    CancelToken, DiskWalk, FindingSink, InterestSet, LeftOutItem, Match, Module, ModuleContext, ModuleError, ModuleId,
    ModuleOutcome, NewFinding, PlaceSeen, ProjectFinder, SchemaError, SettingsSchema, StoredSettings, SystemFacts,
    SystemPorts, WalkError, WalkMode, WalkRequest, WalkSummary,
};
use serde_json::{Value, json};

use crate::{
    FakeAccess, FakeClock, FakeCommandRunner, FakeInstallRecords, FakeInstalledApps, FakeKnownFolders, FakeRunningApps,
    FakeSystemUi, FixtureTree, NaiveWalk,
};

/// The ports a module runs with in a test: a stopped clock, a table of command answers, the stored settings, a cancel
/// token, the access and the system's interface, and the system's facts: the installed and running apps, and the
/// known folders (by default laid out under the fixture tree's root, its home).
#[derive(Debug)]
pub struct Ports {
    pub clock: FakeClock,
    pub commands: FakeCommandRunner,
    /// The stored values, resolved against the module's schema: a missing value takes its default.
    pub settings: StoredSettings,
    pub cancel: CancelToken,
    /// No capability at all by default: every one is `NotApplicable`.
    pub access: FakeAccess,
    pub system_ui: FakeSystemUi,
    /// No app installed by default.
    pub apps: Arc<FakeInstalledApps>,
    /// Nothing running by default.
    pub running: Arc<FakeRunningApps>,
    /// No receipts and no stated macOS version by default.
    pub installs: Arc<FakeInstallRecords>,
    /// `None`: the home layout under the fixture tree's root.
    pub known_folders: Option<FakeKnownFolders>,
    /// The scan is restored from the cache (`ModuleContext::is_restored`); a walked one by default.
    pub restored: bool,
    /// The order the module receives the matches in: the walk's by default.
    pub order: MatchOrder,
    /// The folders the walk does not open, relative to the fixture tree's root: a place among them is not scanned.
    /// None by default.
    pub avoid: Vec<PathBuf>,
}

/// The order a module receives the matches of a run in: a module's answers must not depend on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MatchOrder {
    /// Each match as the walk reports it.
    #[default]
    Walked,
    /// Every match once the walk is done, the last one first; the held matches are delivered even when the walk was
    /// cancelled, before the run reports the cancellation.
    Reversed,
}

impl Ports {
    /// The clock at `seconds` after the Unix epoch, no command answers, every setting at its default, no capability,
    /// a system interface that records its calls, no app installed or running, the fixture home's folders, and a
    /// walked scan whose matches come in the walk's order.
    pub fn at(seconds: u64) -> Self {
        Self {
            clock: FakeClock::at(seconds),
            commands: FakeCommandRunner::default(),
            settings: StoredSettings::default(),
            cancel: CancelToken::new(),
            access: FakeAccess::default(),
            system_ui: FakeSystemUi::default(),
            apps: Arc::default(),
            running: Arc::default(),
            installs: Arc::default(),
            known_folders: None,
            restored: false,
            order: MatchOrder::Walked,
            avoid: Vec::new(),
        }
    }
}

/// What one module did with one fixture tree.
#[derive(Debug)]
pub struct ModuleRun {
    pub findings: Vec<NewFinding>,
    /// What the module recognized and did not offer, in the order it reported it.
    pub left_out: Vec<LeftOutItem>,
    pub outcome: ModuleOutcome,
    root: PathBuf,
    /// The path of every match the module received.
    pub(crate) matched: BTreeSet<PathBuf>,
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
/// [`NaiveWalk`], reading the facts the tree states instead of the disk's, finds the projects of the tree as the engine
/// does ([`ProjectFinder`]), sends each match to the module (indexed in the module's own interests), then calls
/// `finish`. A module's error ends its run with that outcome; a cancelled walk ends it as `Cancelled`.
pub fn run_module(module: &dyn Module, tree: &FixtureTree, ports: &Ports) -> Result<ModuleRun, HarnessError> {
    let id = module.descriptor().id();
    let settings = SettingsSchema::new([module.descriptor()])?.resolve(&ports.settings).module(id);
    let known_folders = ports.known_folders.clone().unwrap_or_else(|| FakeKnownFolders::home_layout(tree.root()));
    let system = SystemFacts::new(&SystemPorts {
        known_folders: Arc::new(known_folders.clone()),
        apps: Arc::clone(&ports.apps) as _,
        running: Arc::clone(&ports.running) as _,
        installs: Arc::clone(&ports.installs) as _,
    });
    let cx = ModuleContext::new(&ports.clock, &ports.commands, &system, &settings, &ports.cancel);
    let cx = if ports.restored { cx.restored() } else { cx };
    let mut interests = InterestSet::new();
    interests.add(id, module.interests());
    interests.add(ModuleId::system(), &ProjectFinder::interests());
    let mut finder = ProjectFinder::default();
    let mut sink = Recorder::default();
    let mut failure = None;
    let mut matched_paths = BTreeSet::new();
    let mut held = Vec::new();
    let mut deliver = |mut matched: Match, failure: &mut Option<ModuleError>, sink: &mut Recorder| {
        if failure.is_some() {
            return;
        }
        matched_paths.insert(matched.path.clone());
        if let Some((_, local)) = interests.owner(matched.interest) {
            matched.interest = local;
        }
        *failure = module.on_match(&matched, &cx, sink).err();
    };
    let mut on_match = |matched: Match| {
        if let Some((owner, local)) = interests.owner(matched.interest)
            && owner == ModuleId::system()
        {
            finder.add(local, &matched);
            return;
        }
        match ports.order {
            MatchOrder::Walked => deliver(matched, &mut failure, &mut sink),
            MatchOrder::Reversed => held.push(matched),
        }
    };
    let roots = [tree.root().to_path_buf()];
    let avoid: Vec<PathBuf> = ports.avoid.iter().map(|path| tree.root().join(path)).collect();
    let request =
        WalkRequest { roots: &roots, interests: &interests, avoid: &avoid, places: &[], mode: WalkMode::Full };
    let walk = NaiveWalk::new(tree.root()).with_known_folders(known_folders).with_stated(tree.stated().clone());
    let walked = walk.walk(&request, &ports.cancel, &mut on_match, &mut |_| {});
    for matched in held.into_iter().rev() {
        deliver(matched, &mut failure, &mut sink);
    }
    system.set_projects(finder.finish(walked.is_ok().then_some(&walk as &dyn DiskWalk)));
    let (walk_complete, places) = match walked {
        Ok(summary) => (summary.is_complete(), places_of(module, &interests, &summary)),
        Err(WalkError::Cancelled) => {
            failure = failure.or(Some(ModuleError::Cancelled));
            (false, Vec::new())
        }
        Err(error) => return Err(error.into()),
    };
    if failure.is_none() {
        let cx = cx.with_places(&places);
        let cx = if walk_complete { cx.complete_walk() } else { cx };
        failure = module.finish(&cx, &mut sink).err();
    }
    let outcome = match failure {
        None => ModuleOutcome::Completed,
        Some(ModuleError::Cancelled) => ModuleOutcome::Cancelled,
        Some(ModuleError::Failed { reason }) => ModuleOutcome::Failed { reason },
    };
    let Recorder { findings, left_out } = sink;
    Ok(ModuleRun { findings, left_out, outcome, root: tree.root().to_path_buf(), matched: matched_paths })
}

/// What the walk saw of the place of each of the module's interests, by its own index; `Unknown` when it said
/// nothing.
fn places_of(module: &dyn Module, interests: &InterestSet, summary: &WalkSummary) -> Vec<PlaceSeen> {
    let mut places = vec![PlaceSeen::Unknown; module.interests().len()];
    for (&interest, &seen) in &summary.places {
        if let Some(slot) = interests.owner(interest).and_then(|(_, local)| places.get_mut(local)) {
            *slot = seen;
        }
    }
    places
}

/// Keeps everything a module reports, in order.
#[derive(Default)]
struct Recorder {
    findings: Vec<NewFinding>,
    left_out: Vec<LeftOutItem>,
}

impl FindingSink for Recorder {
    fn push(&mut self, finding: NewFinding) {
        self.findings.push(finding);
    }

    fn left_out(&mut self, left_out: LeftOutItem) {
        self.left_out.push(left_out);
    }
}

impl ModuleRun {
    /// A stable JSON report of the run: the outcome, the findings sorted by identity and, when the module left any
    /// out, its left-outs in order; every path relative to the fixture root (the root itself is `.`), pretty-printed
    /// with a final newline.
    pub fn golden(&self) -> Result<String, serde_json::Error> {
        let mut findings: Vec<&NewFinding> = self.findings.iter().collect();
        findings.sort_by(|left, right| left.key().cmp(right.key()));
        let mut report = json!({ "outcome": self.outcome, "findings": findings });
        if !self.left_out.is_empty() {
            let left_out = self
                .left_out
                .iter()
                .map(|item| json!({ "category": item.category, "reason": item.reason, "sizes": item.sizes }));
            report["leftOut"] = Value::Array(left_out.collect());
        }
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
        ByteSize, CategoryDescriptor, CategoryId, Claim, DangerLevel, FindingSink, Interest, LeftOutItem, Locales,
        Match, Module, ModuleChoice, ModuleContext, ModuleDescriptor, ModuleError, ModuleId, NewFinding, Pattern,
        Removal, SettingDescriptor, Sizes, Subject, Text, TextArg, TextKey,
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

    /// `EN` with the texts `extra` (JSON members, comma-separated) added.
    pub(crate) fn en_with(extra: &str) -> &'static str {
        let body = EN.trim_end().trim_end_matches('}');
        String::leak(format!("{body}, {extra} }}"))
    }

    /// A module that claims every folder named `Caches` as a Low-danger finding; built with `failing()`, it fails in
    /// `finish`; with `leaving_out(reason)`, it leaves out one item of 7 bytes of its category in `finish` (of
    /// `left_out_category`, when set).
    pub(crate) struct CacheModule {
        descriptor: ModuleDescriptor,
        interests: Vec<Interest>,
        fail_in_finish: bool,
        left_out: Option<&'static str>,
        left_out_category: Option<CategoryId>,
    }

    impl CacheModule {
        pub(crate) fn with_dictionary(en: &'static str) -> Self {
            Self::with_settings(en, vec![])
        }

        pub(crate) fn with_settings(en: &'static str, settings: Vec<SettingDescriptor>) -> Self {
            let id = ModuleId::new("sample").unwrap();
            let category = CategoryDescriptor::new(CategoryId::new(id, "sample.caches").unwrap(), DangerLevel::Low);
            let descriptor = ModuleDescriptor::new(
                id,
                fresher_core::ModuleGroup::Development,
                vec![category],
                settings,
                Locales { en },
            )
            .unwrap();
            let interest = Interest::new(Pattern::FolderName("Caches".into()), Claim::Whole, vec![]).unwrap();
            Self {
                descriptor,
                interests: vec![interest],
                fail_in_finish: false,
                left_out: None,
                left_out_category: None,
            }
        }

        pub(crate) fn new() -> Self {
            Self::with_dictionary(EN)
        }

        pub(crate) fn failing() -> Self {
            Self::new().that_fails()
        }

        /// This module, failing in `finish`.
        pub(crate) fn that_fails(self) -> Self {
            Self { fail_in_finish: true, ..self }
        }

        /// This module, declaring `keys` as its problems.
        pub(crate) fn declaring(mut self, keys: &[&str]) -> Self {
            let keys = keys.iter().map(|key| TextKey::new(*key).unwrap()).collect();
            self.descriptor = self.descriptor.with_problems(keys).unwrap();
            self
        }

        /// This module, offering a choice `name` that selects its category.
        pub(crate) fn choosing(mut self, name: &str) -> Self {
            let id = CategoryId::new(self.descriptor.id(), "sample.caches").unwrap();
            self.descriptor = self.descriptor.with_choices(vec![ModuleChoice::new(name, vec![id])]).unwrap();
            self
        }

        pub(crate) fn leaving_out(self, reason: &'static str) -> Self {
            Self { left_out: Some(reason), ..self }
        }

        /// This module, leaving out its item in `category` instead of its own.
        pub(crate) fn in_category(self, category: CategoryId) -> Self {
            Self { left_out_category: Some(category), ..self }
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

        fn finish(&self, _cx: &ModuleContext<'_>, out: &mut dyn FindingSink) -> Result<(), ModuleError> {
            if let Some(reason) = self.left_out {
                let category = self.left_out_category.unwrap_or_else(|| self.descriptor.categories()[0].id());
                let sizes = Sizes::complete(ByteSize::new(7), ByteSize::new(7));
                out.left_out(LeftOutItem { category, reason: TextKey::new(reason).unwrap(), sizes });
            }
            if self.fail_in_finish {
                return Err(ModuleError::Failed { reason: Text::new(TextKey::new("sample.error.broken").unwrap()) });
            }
            Ok(())
        }
    }

    /// A module without categories whose one interest is every item of the caches; `finish` records what the walk saw
    /// of that place.
    pub(crate) struct PlaceProbe {
        descriptor: ModuleDescriptor,
        interests: Vec<Interest>,
        pub(crate) seen: std::sync::Mutex<Vec<fresher_core::PlaceSeen>>,
    }

    impl PlaceProbe {
        pub(crate) fn new() -> Self {
            let id = ModuleId::new("sample").unwrap();
            let group = fresher_core::ModuleGroup::Development;
            let descriptor = ModuleDescriptor::new(id, group, vec![], vec![], Locales { en: EN }).unwrap();
            let caches = Pattern::Under(fresher_core::KnownFolder::Home, "Library/Caches/*".into());
            let interest = Interest::new(caches, Claim::Item, vec![]).unwrap();
            Self { descriptor, interests: vec![interest], seen: std::sync::Mutex::default() }
        }
    }

    impl Module for PlaceProbe {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn interests(&self) -> &[Interest] {
            &self.interests
        }

        fn on_match(&self, _: &Match, _: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
            Ok(())
        }

        fn finish(&self, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
            self.seen.lock().unwrap().push(cx.place_seen(0));
            Ok(())
        }
    }

    /// A module without categories and interests whose `finish` keeps the projects the system knew then.
    pub(crate) struct ProjectProbe {
        descriptor: ModuleDescriptor,
        pub(crate) seen: std::sync::Mutex<Option<Vec<fresher_core::Project>>>,
    }

    impl ProjectProbe {
        pub(crate) fn new() -> Self {
            let id = ModuleId::new("sample").unwrap();
            let group = fresher_core::ModuleGroup::Development;
            let descriptor = ModuleDescriptor::new(id, group, vec![], vec![], Locales { en: EN }).unwrap();
            Self { descriptor, seen: std::sync::Mutex::default() }
        }
    }

    impl Module for ProjectProbe {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn interests(&self) -> &[Interest] {
            &[]
        }

        fn on_match(&self, _: &Match, _: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
            Ok(())
        }

        fn finish(&self, cx: &ModuleContext<'_>, _: &mut dyn FindingSink) -> Result<(), ModuleError> {
            *self.seen.lock().unwrap() = cx.system().projects().map(<[_]>::to_vec);
            Ok(())
        }
    }

    /// A Node project with dated folders: its `node_modules` has a package of 10 bytes.
    pub(crate) const NODE_PROJECT: &str = "[[file]]\npath = \"p/package.json\"\n\
        [[dir]]\npath = \"p\"\nmodified = 100\n\
        [[file]]\npath = \"p/node_modules/pkg/a.js\"\nsize = 10\n\
        [[dir]]\npath = \"p/node_modules/pkg\"\nmodified = 600\n\
        [[dir]]\npath = \"p/node_modules\"\nmodified = 500\n";

    /// A home whose caches hold one app's folder.
    pub(crate) const CACHES: &str = "[[file]]\npath = \"Library/Caches/com.example.App/data\"\n";

    /// Two `Caches` folders, the second one 10 bytes.
    pub(crate) const TREE: &str = "[[dir]]\npath = \"a/Caches\"\n[[file]]\npath = \"b/Caches/data\"\nsize = 10\n";
}

#[cfg(test)]
mod tests {
    mod negative {
        use fresher_core::{ModuleOutcome, WalkError};

        use crate::run_module::sample::{CACHES, CacheModule, NODE_PROJECT, PlaceProbe, TREE};
        use crate::{FixtureTree, HarnessError, Ports, run_module};

        #[test]
        fn the_module_never_receives_the_project_finder_s_matches() {
            let tree = FixtureTree::from_toml_str(&format!("{NODE_PROJECT}[[dir]]\npath = \"p/Caches\"\n")).unwrap();

            let run = run_module(&CacheModule::new(), &tree, &Ports::at(0)).unwrap();

            let expected = [tree.root().join("p/Caches")];
            assert_eq!(run.matched.iter().collect::<Vec<_>>(), expected.iter().collect::<Vec<_>>());
            assert_eq!(run.findings.len(), 1);
        }

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
        fn a_golden_without_left_outs_is_unchanged() {
            let tree = FixtureTree::from_toml_str(TREE).unwrap();

            let golden = run_module(&CacheModule::new(), &tree, &Ports::at(0)).unwrap().golden().unwrap();

            let report: serde_json::Value = serde_json::from_str(&golden).unwrap();
            let fields: Vec<&str> = report.as_object().unwrap().keys().map(String::as_str).collect();
            assert_eq!(fields, ["findings", "outcome"]);
        }

        #[test]
        fn a_place_avoided_reaches_the_module_not_scanned() {
            let tree = FixtureTree::from_toml_str(CACHES).unwrap();
            let probe = PlaceProbe::new();
            let ports = Ports { avoid: vec!["Library/Caches".into()], ..Ports::at(0) };

            run_module(&probe, &tree, &ports).unwrap();

            assert_eq!(*probe.seen.lock().unwrap(), [fresher_core::PlaceSeen::NotScanned]);
        }

        #[test]
        fn a_walk_error_is_the_harness_error() {
            let error = HarnessError::from(WalkError::Cancelled);

            assert_eq!(error.to_string(), "the walk of the fixture tree failed: the walk was cancelled");
        }
    }

    mod positive {
        use fresher_core::{ByteSize, ModuleOutcome};

        use crate::run_module::sample::{CACHES, CacheModule, NODE_PROJECT, PlaceProbe, ProjectProbe, TREE};
        use crate::{FixtureTree, MatchOrder, Ports, run_module};

        #[test]
        fn a_module_sees_the_projects_with_their_facts_in_finish() {
            use fresher_core::{ByteSize, LastChange, ProjectKind, Timestamp};
            let tree = FixtureTree::from_toml_str(NODE_PROJECT).unwrap();
            let probe = ProjectProbe::new();

            run_module(&probe, &tree, &Ports::at(0)).unwrap();

            let seen = probe.seen.lock().unwrap();
            let projects = seen.as_deref().unwrap();
            assert_eq!(projects.len(), 1);
            assert_eq!(projects[0].root, tree.root().join("p"));
            assert_eq!(projects[0].kinds, [ProjectKind::Node]);
            assert_eq!(projects[0].last_activity, LastChange::Known(Timestamp::from_secs(100)));
            assert_eq!(projects[0].artifacts.len(), 1);
            assert_eq!(projects[0].artifacts[0].path, tree.root().join("p/node_modules"));
            assert_eq!(projects[0].artifacts[0].sizes.logical, ByteSize::new(10));
            assert_eq!(projects[0].artifacts[0].last_change, LastChange::Known(Timestamp::from_secs(600)));
        }

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
        fn a_place_read_reaches_the_module_in_finish() {
            let tree = FixtureTree::from_toml_str(CACHES).unwrap();
            let probe = PlaceProbe::new();

            run_module(&probe, &tree, &Ports::at(0)).unwrap();

            assert_eq!(*probe.seen.lock().unwrap(), [fresher_core::PlaceSeen::Read]);
        }

        #[test]
        fn the_matches_reversed_reach_the_module_last_first() {
            let tree = FixtureTree::from_toml_str(TREE).unwrap();
            let identities = |order: MatchOrder| {
                let run = run_module(&CacheModule::new(), &tree, &Ports { order, ..Ports::at(0) }).unwrap();
                run.findings.iter().map(|finding| finding.key().identity().to_owned()).collect::<Vec<_>>()
            };

            let mut reversed = identities(MatchOrder::Reversed);

            assert_eq!(reversed.len(), 2);
            reversed.reverse();
            assert_eq!(reversed, identities(MatchOrder::Walked));
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
        fn the_harness_keeps_the_left_outs_and_the_golden_lists_them() {
            let tree = FixtureTree::from_toml_str(TREE).unwrap();
            let module = CacheModule::new().leaving_out("sample.left-out.open");

            let run = run_module(&module, &tree, &Ports::at(0)).unwrap();

            assert_eq!(run.left_out.len(), 1);
            assert_eq!(run.left_out[0].reason.as_str(), "sample.left-out.open");
            let report: serde_json::Value = serde_json::from_str(&run.golden().unwrap()).unwrap();
            assert_eq!(
                report["leftOut"],
                serde_json::json!([{
                    "category": "sample.caches",
                    "reason": "sample.left-out.open",
                    "sizes": { "logical": 7, "allocated": 7, "reclaimable": 7, "completeness": "complete" },
                }])
            );
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
