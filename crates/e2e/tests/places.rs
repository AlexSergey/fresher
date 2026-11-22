//! Places walked outside the scan roots, end to end: a fixture with a home (the scan root) and a shared temporary
//! folder side by side, the real scanner and scan cache, a module whose interest is anchored in the temporary folder,
//! and the real `DiskCleaner` (`Reach::Sandbox`, a fake Trash next to them).

#[cfg(test)]
#[path = "support/events.rs"]
mod events;

#[cfg(test)]
mod home {
    use std::fs;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;

    use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach};
    use fresher_core::Cleaner as _;
    use fresher_core::{
        Access, AccessStatus, ByteSize, Capability, CategoryDescriptor, CategoryId, Changes, Claim, CleanupContext,
        DangerLevel, Depth, Event, Finding, FindingId, FindingOrder, FindingSink, FindingsQuery, Interest,
        JournalPosition, KnownFolder, Locales, Match, Module, ModuleContext, ModuleDescriptor, ModuleError,
        ModuleGroup, ModuleId, NewFinding, Pattern, PlanItem, Protection, Rehearsal, Removability, Removal,
        SANDBOX_MARKER, Sizes, Subject, SystemPorts, Text, TextKey, VolumeFacts, VolumeId,
    };
    use fresher_engine::{CleanupPorts, Engine, Ports, Registry};
    use fresher_scanner::{Scanner, ScannerOptions, StdReader, known_folders_anchor};
    use fresher_store::{DataFolder, FileCleanupJournal, FileScanCache, FileSettingsStore};
    use fresher_testing::{
        FakeAccess, FakeChangeJournal, FakeClock, FakeCommandRunner, FakeInstalledApps, FakeKnownFolders,
        FakeOpenFiles, FakeRunningApps, FakeSystemUi, FakeTrash, FakeVolumes, FixtureTree,
    };

    use crate::events::until;

    const TEXTS: &str = r#"{
        "temp.items.name": "Temporary items",
        "temp.items.what": "Items a test left in the temporary folder.",
        "temp.items.made-by": "The test tree.",
        "temp.items.if-removed": "Nothing: it is a test.",
        "temp.items.comes-back": "With the next test.",
        "temp.finding.title": "A temporary item",
        "temp.finding.reason": "The test left it."
    }"#;

    /// When the scans are taken and where the scripted journal stands.
    const NOW: u64 = 1_735_689_600;
    const AT: JournalPosition = JournalPosition { journal: 7, cursor: 100 };

    /// A module that finds every top-level item of the shared temporary folder, whole.
    struct TempModule {
        descriptor: ModuleDescriptor,
        interests: Vec<Interest>,
    }

    impl TempModule {
        fn new(folder: KnownFolder) -> Self {
            let id = ModuleId::new("temp").unwrap();
            let category = CategoryDescriptor::new(CategoryId::new(id, "temp.items").unwrap(), DangerLevel::Safe);
            let interest = Interest::new(Pattern::Under(folder, "*".into()), Claim::Whole, Vec::new()).unwrap();
            let descriptor =
                ModuleDescriptor::new(id, ModuleGroup::Development, vec![category], vec![], Locales { en: TEXTS })
                    .unwrap();
            Self { descriptor, interests: vec![interest] }
        }
    }

    impl Module for TempModule {
        fn descriptor(&self) -> &ModuleDescriptor {
            &self.descriptor
        }

        fn interests(&self) -> &[Interest] {
            &self.interests
        }

        fn on_match(
            &self,
            found: &Match,
            _cx: &ModuleContext<'_>,
            out: &mut dyn FindingSink,
        ) -> Result<(), ModuleError> {
            // The sandbox's own marker is the test's, not an item.
            if found.path.file_name().is_some_and(|name| name == SANDBOX_MARKER) {
                return Ok(());
            }
            let text = |key: &str| Text::new(TextKey::new(key).unwrap());
            out.push(NewFinding::new(
                &self.descriptor.categories()[0],
                found.path.to_string_lossy(),
                Subject::Path(found.path.clone()),
                found.sizes,
                text("temp.finding.title"),
                text("temp.finding.reason"),
                Removal::MoveToTrash(found.path.clone()),
            ));
            Ok(())
        }

        fn finish(&self, _cx: &ModuleContext<'_>, _out: &mut dyn FindingSink) -> Result<(), ModuleError> {
            Ok(())
        }
    }

    /// The home `home/` and the shared temporary folder `tmp/` (a sandbox), side by side, with the engine over them.
    pub(crate) struct Places {
        _tree: FixtureTree,
        pub(crate) root: PathBuf,
        pub(crate) engine: Engine,
        /// The cleaner the engine cleans through.
        cleaner: Arc<DiskCleaner>,
        _data: tempfile::TempDir,
    }

    impl Places {
        /// The fixture with the module's interest anchored in `folder`, the shared temporary folder at `temp`
        /// (relative to the fixture).
        pub(crate) fn new(folder: KnownFolder, temp: &str) -> Self {
            let toml = format!(
                "[[file]]\npath = \"home/notes.txt\"\n[[file]]\npath = \"{temp}/old.log\"\nsize = 4096\n\
                 [[dir]]\npath = \"{temp}/session\"\n[[file]]\npath = \"{temp}/session/data.bin\"\nsize = 8192\n"
            );
            let tree = FixtureTree::from_toml_str(&toml).unwrap();
            tree.mark_sandbox(Path::new(temp)).unwrap();
            let root = fs::canonicalize(tree.root()).unwrap();
            let data = tempfile::tempdir().unwrap();
            let (engine, cleaner) = engine(&root, &root.join(temp), data.path(), folder);
            Self { _tree: tree, root, engine, cleaner, _data: data }
        }

        /// The cleaner's full rehearsal of the item `relative` of the fixture, under `roots` (the plan's and the
        /// settings' with the places, as the engine gives them).
        pub(crate) fn rehearse(&self, relative: &str, roots: &[PathBuf]) -> Rehearsal {
            let module = TempModule::new(KnownFolder::SharedTemp);
            let path = self.path(relative);
            let text = |key: &str| Text::new(TextKey::new(key).unwrap());
            let found = NewFinding::new(
                &module.descriptor.categories()[0],
                path.to_string_lossy().into_owned(),
                Subject::Path(path.clone()),
                Sizes::ZERO,
                text("temp.finding.title"),
                text("temp.finding.reason"),
                Removal::MoveToTrash(path),
            );
            let item = PlanItem::new(&Finding::new(FindingId::new(1), found, Removability::Removable));
            let (protection, access) = (Protection::default(), AccessStatus::default());
            let cx = CleanupContext { roots, protection: &protection, access: &access, guarded: &[] };
            self.cleaner.rehearse(roots, &[item], &cx, Depth::Full).remove(0)
        }

        /// Scans in `mode` and gives the events up to the end.
        pub(crate) fn scan(&self, mode: fresher_engine::ScanMode) -> Vec<Event> {
            let events = self.engine.subscribe();
            self.engine.start_scan(mode).unwrap();
            until(&events, |event| matches!(event, Event::ScanFinished { .. }))
        }

        /// The findings' paths relative to the fixture, sorted.
        pub(crate) fn findings(&self) -> Vec<String> {
            let page = self.engine.findings(&FindingsQuery::new(None, None, FindingOrder::Found, 0, 100)).unwrap();
            let mut found: Vec<String> = page
                .findings
                .iter()
                .filter_map(|finding| match finding.finding().subject() {
                    Subject::Path(path) => Some(path.strip_prefix(&self.root).unwrap().display().to_string()),
                    Subject::External { .. } => None,
                })
                .collect();
            found.sort();
            found
        }

        pub(crate) fn path(&self, relative: &str) -> PathBuf {
            self.root.join(relative)
        }
    }

    fn engine(root: &Path, temp: &Path, data: &Path, folder: KnownFolder) -> (Engine, Arc<DiskCleaner>) {
        let clock = Arc::new(FakeClock::at(NOW));
        let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) };
        let volumes = Arc::new(FakeVolumes::default().with("/", facts));
        let folders = Arc::new(
            FakeKnownFolders::new().with(KnownFolder::Home, root.join("home")).with(KnownFolder::SharedTemp, temp),
        );
        // Each rescan is told nothing changed: the place is read again all the same.
        let nothing = || Ok(Changes { folders: Vec::new(), subtrees: Vec::new(), end: AT });
        let journal = FakeChangeJournal::default().at(VolumeId::new(1), AT).answering(nothing()).answering(nothing());
        let options = ScannerOptions { threads: 2.try_into().unwrap(), ..ScannerOptions::default() };
        let scanner = Scanner::new(
            Arc::new(StdReader::new()),
            Arc::clone(&volumes) as _,
            Arc::new(journal),
            Arc::new(FileScanCache::new(DataFolder::open(data).unwrap())),
            known_folders_anchor(&*folders),
            options,
        )
        .with_clock(Arc::clone(&clock) as _);
        let running = Arc::new(FakeRunningApps::new(Vec::new()));
        let commands = Arc::new(FakeCommandRunner::default());
        let ports = Ports {
            walk: Arc::new(scanner),
            settings: Arc::new(FileSettingsStore::new(DataFolder::open(data).unwrap())),
            clock: Arc::clone(&clock) as _,
            commands: Arc::clone(&commands) as _,
            access: Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Granted)])),
            system_ui: Arc::new(FakeSystemUi::default()),
            system: SystemPorts {
                known_folders: Arc::clone(&folders) as _,
                apps: Arc::new(FakeInstalledApps::new(Vec::new())),
                running: Arc::clone(&running) as _,
                installs: Arc::new(fresher_testing::FakeInstallRecords::default()),
                files: Arc::new(fresher_testing::DiskFiles),
            },
        };
        let cleaner_ports = CleanerPorts {
            trash: Arc::new(FakeTrash::new(root.join("trash"))),
            open_files: Arc::new(FakeOpenFiles::default()),
            running,
            commands,
            clock,
        };
        let user = fs::metadata(root).unwrap().uid();
        let home = root.join("home");
        let cleaner = Arc::new(DiskCleaner::new(Reach::Sandbox, user, home, Protection::default(), cleaner_ports));
        let journal = Arc::new(FileCleanupJournal::open(&DataFolder::open(data).unwrap()));
        let cleanup = CleanupPorts {
            cleaner: Arc::clone(&cleaner) as _,
            journal,
            volumes,
            finder: Arc::new(fresher_testing::FakeFinderSettings::new(fresher_core::AutoEmpty::Unknown)),
            trashes: Vec::new(),
        };
        let mut registry = Registry::new().unwrap();
        registry.register(Box::new(TempModule::new(folder))).unwrap();
        (Engine::with_cleanup(registry, ports, cleanup).unwrap(), cleaner)
    }
}

#[cfg(test)]
mod negative {
    use std::path::PathBuf;

    use fresher_core::{Event, KnownFolder, Rehearsal, ScanKind, SettingUpdate, SettingValue, SkipReason};
    use fresher_engine::ScanMode;

    use crate::home::Places;

    fn kind(events: &[Event]) -> ScanKind {
        events
            .iter()
            .find_map(|event| match event {
                Event::ScanFinished { kind, .. } => Some(*kind),
                _ => None,
            })
            .unwrap()
    }

    #[test]
    fn no_module_naming_the_place_walks_nothing_outside_the_roots() {
        let places = Places::new(KnownFolder::UserTemp, "tmp");

        places.scan(ScanMode::Full);

        assert_eq!(places.findings(), Vec::<String>::new());
        let map = places.engine.storage_map().unwrap();
        assert_eq!(map.volumes[0].roots().len(), 1);
        assert_eq!(map.volumes[0].places(), []);
    }

    #[test]
    fn an_item_of_a_place_no_module_names_is_refused_as_outside_every_root() {
        let places = Places::new(KnownFolder::UserTemp, "tmp");
        places.scan(ScanMode::Full);

        let refused = places.rehearse("tmp/old.log", &[PathBuf::from("~")]);
        let walked = places.rehearse("tmp/old.log", &[PathBuf::from("~"), places.path("tmp")]);

        // No root places it: under a sandbox's reach that is "not in the sandbox", checked before the roots (a release
        // build's real-disk reach answers `OutsideRoots`, crates/cleaner/src/guard/mod.rs).
        assert_eq!(refused, Rehearsal::Skipped(SkipReason::NotInSandbox));
        assert!(matches!(walked, Rehearsal::Passed(_)), "{walked:?}");
    }

    #[test]
    fn a_place_inside_the_home_is_not_walked_twice() {
        let places = Places::new(KnownFolder::SharedTemp, "home/tmp");

        places.scan(ScanMode::Full);

        assert_eq!(places.findings(), ["home/tmp/old.log", "home/tmp/session"]);
        assert_eq!(places.engine.storage_map().unwrap().volumes[0].roots().len(), 1);
    }

    #[test]
    fn a_modules_switch_off_changes_nothing_the_walk_reads() {
        let places = Places::new(KnownFolder::SharedTemp, "tmp");
        places.scan(ScanMode::Full);
        let off = SettingUpdate { key: "temp.enabled".into(), value: SettingValue::Switch(false) };
        places.engine.update_setting(off).unwrap();

        let events = places.scan(ScanMode::Auto);

        assert!(matches!(kind(&events), ScanKind::Rescan { .. }));
        assert_eq!(places.findings(), Vec::<String>::new());
        assert_eq!(places.engine.storage_map().unwrap().volumes[0].roots().len(), 2);
    }
}

#[cfg(test)]
mod positive {
    use fresher_core::{CleanupSelection, Confirmation, Event, KnownFolder, PartKind, ScanKind, Text, TextKey};
    use fresher_engine::ScanMode;
    use serde_json::json;

    use crate::events::until;
    use crate::home::Places;

    #[test]
    fn the_place_beside_the_home_is_walked_shown_after_it_and_cleaned() {
        let places = Places::new(KnownFolder::SharedTemp, "tmp");

        places.scan(ScanMode::Full);

        assert_eq!(places.findings(), ["tmp/old.log", "tmp/session"]);
        let map = places.engine.storage_map().unwrap();
        let volume = &map.volumes[0];
        assert_eq!(volume.roots().len(), 2);
        assert_eq!(volume.places().len(), 1);
        assert_eq!(volume.places()[0].root, volume.roots()[1]);
        assert_eq!(volume.places()[0].name, Text::new(TextKey::new("core.place.shared-temp.name").unwrap()));
        let total: u64 = volume.parts().iter().map(|part| part.sizes.allocated.bytes()).sum();
        assert_eq!(total, volume.capacity().bytes());
        let explained = volume.parts().iter().find(|part| part.kind == PartKind::Explained).unwrap();
        assert_eq!(explained.sizes.logical.bytes(), 4096 + 8192);

        let selection: CleanupSelection = serde_json::from_value(
            json!({ "findings": [], "categories": [{ "category": "temp.items", "except": [] }] }),
        )
        .unwrap();
        let view = places.engine.prepare_cleanup(&selection).unwrap();
        assert_eq!(view.summary.count, 2);
        let echo = json!({ "plan": view.plan, "count": view.summary.count, "allocated": view.summary.sizes.allocated,
                           "highest": view.summary.highest, "method": "trash" });
        let confirmation: Confirmation = serde_json::from_value(echo).unwrap();
        let events = places.engine.subscribe();
        places.engine.confirm_cleanup(&confirmation).unwrap();
        until(&events, |event| matches!(event, Event::CleanupFinished { .. }));

        assert!(!places.path("tmp/old.log").exists());
        assert!(!places.path("tmp/session").exists());
        assert_eq!(std::fs::read_dir(places.path("trash")).unwrap().count(), 2);
    }

    #[test]
    fn a_rescan_with_the_same_places_uses_the_cache_and_reads_the_place_again() {
        let places = Places::new(KnownFolder::SharedTemp, "tmp");
        places.scan(ScanMode::Full);
        std::fs::write(places.path("tmp/new.log"), b"new").unwrap();

        let events = places.scan(ScanMode::Auto);

        let kind = events.iter().find_map(|event| match event {
            Event::ScanFinished { kind, .. } => Some(*kind),
            _ => None,
        });
        assert!(matches!(kind, Some(ScanKind::Rescan { .. })));
        assert_eq!(places.findings(), ["tmp/new.log", "tmp/old.log", "tmp/session"]);
    }
}
