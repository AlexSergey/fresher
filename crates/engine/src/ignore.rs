//! The commands on the settings beyond one value: ignoring and restoring, and putting every setting back.

use fresher_core::{CategoryDescriptor, CommandError, IgnoreTarget, IgnoredEntry, Subject};

use crate::Engine;
use crate::error::lock;

impl Engine {
    /// Adds a category, the path of a finding, a module or a group to the ignore lists; their findings leave every
    /// answer at once. Refused for a category, a module or a group nobody registered (`UnknownCategory`,
    /// `UnknownModule`, `UnknownGroup`), a finding of another scan or an ignored one (`UnknownFinding`), and a finding
    /// without a path of its own (`CannotIgnore`). Ignoring what is ignored already changes nothing.
    pub fn ignore(&self, target: &IgnoreTarget) -> Result<(), CommandError> {
        let entry = match target {
            IgnoreTarget::Category { category } => {
                let known = self.shared.registry.categories().map(CategoryDescriptor::id);
                if !known.into_iter().any(|id| id.as_str() == category) {
                    return Err(CommandError::UnknownCategory { id: category.clone() });
                }
                IgnoredEntry::Category { category: category.clone() }
            }
            IgnoreTarget::Finding { finding } => {
                match lock(&self.shared.results)?.finding(*finding)?.finding().subject() {
                    Subject::Path(path) => IgnoredEntry::Path { path: path.clone() },
                    Subject::External { .. } => return Err(CommandError::CannotIgnore),
                }
            }
            IgnoreTarget::Module { module } => match self.shared.registry.module_id(module) {
                Some(id) => IgnoredEntry::Module { module: id.as_str().into() },
                None => return Err(CommandError::UnknownModule { id: module.clone() }),
            },
            IgnoreTarget::Group { group } => {
                if self.shared.registry.modules_of(*group).next().is_none() {
                    return Err(CommandError::UnknownGroup { id: group.as_str().into() });
                }
                IgnoredEntry::Group { group: *group }
            }
        };
        self.set_ignored(&entry, true)
    }

    /// Takes an entry out of the ignore lists; its findings of the last scan come back at once, unless another entry
    /// still hides them (a module of an ignored group stays hidden). Restoring what is not ignored changes nothing.
    pub fn restore_ignored(&self, entry: &IgnoredEntry) -> Result<(), CommandError> {
        self.set_ignored(entry, false)
    }

    /// Puts every setting but the ignore lists back to its default, in one save; one `SettingChanged` per value that
    /// changed. Refused while the settings file is read-only.
    pub fn reset_settings(&self) -> Result<(), CommandError> {
        let mut settings = lock(&self.shared.settings)?;
        let changes = settings.reset(&*self.shared.ports.settings)?;
        self.take_changes(&settings, &changes)
    }

    fn set_ignored(&self, entry: &IgnoredEntry, ignored: bool) -> Result<(), CommandError> {
        let mut settings = lock(&self.shared.settings)?;
        match settings.set_ignored(entry, ignored, &*self.shared.ports.settings)? {
            Some(change) => self.take_changes(&settings, &[change]),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_core::{FindingId, FindingOrder, FindingsQuery, IgnoreTarget, Overview};

    use crate::Engine;
    use crate::cleanup::testing::fixture;
    use crate::testing::{MemorySettings, scan};

    fn category(id: &str) -> IgnoreTarget {
        IgnoreTarget::Category { category: id.into() }
    }

    fn finding(id: u32) -> IgnoreTarget {
        IgnoreTarget::Finding { finding: FindingId::new(id) }
    }

    /// The fixture of the cleanup's tests over `settings`, after one scan, and its settings.
    fn scanned(settings: MemorySettings) -> (Engine, Arc<MemorySettings>) {
        let fixture = fixture().settings(settings);
        let settings = Arc::clone(&fixture.settings);
        let engine = fixture.engine();
        scan(&engine);
        (engine, settings)
    }

    /// The ids of every finding the engine answers, largest first.
    fn shown(engine: &Engine) -> Vec<u32> {
        let query = FindingsQuery::new(None, None, FindingOrder::Found, 0, 100);
        engine.findings(&query).unwrap().findings.iter().map(|finding| finding.id().get()).collect()
    }

    fn overview(engine: &Engine) -> Overview {
        engine.overview().unwrap()
    }

    mod negative {
        use fresher_core::{
            CommandError, FindingId, FormatVersion, IgnoredEntry, Protected, SettingUpdate, SettingValue, StoreError,
        };
        use serde_json::json;

        use super::{category, finding, overview, scanned, shown};
        use crate::cleanup::Context;
        use crate::cleanup::testing::select;
        use crate::testing::MemorySettings;

        #[test]
        fn an_unknown_category_or_finding_is_refused_and_nothing_is_saved() {
            let (engine, settings) = scanned(MemorySettings::default());

            assert_eq!(
                engine.ignore(&category("gamma.caches")),
                Err(CommandError::UnknownCategory { id: "gamma.caches".into() })
            );
            assert_eq!(engine.ignore(&finding(99)), Err(CommandError::UnknownFinding { id: FindingId::new(99) }));
            assert_eq!(settings.saves(), 0);
        }

        #[test]
        fn an_ignored_finding_is_in_no_answer() {
            let (engine, _) = scanned(MemorySettings::default());

            engine.ignore(&finding(2)).unwrap();
            engine.ignore(&category("alpha.caches")).unwrap();

            assert_eq!(shown(&engine), Vec::<u32>::new());
            assert_eq!(overview(&engine).totals, []);
            assert_eq!(engine.finding(FindingId::new(2)), Err(CommandError::UnknownFinding { id: FindingId::new(2) }));
            assert_eq!(engine.ignore(&finding(2)), Err(CommandError::UnknownFinding { id: FindingId::new(2) }));
        }

        #[test]
        fn an_ignored_finding_cannot_be_selected() {
            let (engine, _) = scanned(MemorySettings::default());

            engine.ignore(&finding(1)).unwrap();

            let refused = engine.prepare_cleanup(&select(&[1], &[]));
            assert_eq!(refused, Err(CommandError::UnknownFinding { id: FindingId::new(1) }));
            let ignored_category = engine
                .ignore(&category("beta.logs"))
                .map(|()| engine.prepare_cleanup(&select(&[], &[("beta.logs", &[])])));
            assert_eq!(ignored_category, Ok(Err(CommandError::NothingToClean)));
        }

        #[test]
        fn the_cleaner_is_kept_from_an_ignored_path_and_what_holds_it() {
            let (engine, _) = scanned(MemorySettings::with(&[("core.ignore.paths", json!(["/home/b/Caches/x"]))]));

            let cx = Context::now(&engine.shared, crate::cleanup::Scope::Cleanup).unwrap();

            let protection = cx.borrow().protection;
            assert_eq!(protection.check("/home/b/Caches/x/Caches".as_ref()), Err(Protected::Ignored));
            assert_eq!(protection.check("/home/b/Caches".as_ref()), Err(Protected::Ignored));
            assert_eq!(protection.check("/home/b/Caches/y".as_ref()), Ok(()));
        }

        #[test]
        fn a_shared_prefix_is_not_below_an_ignored_path() {
            let (engine, _) = scanned(MemorySettings::with(&[("core.ignore.paths", json!(["/home/a/Cache"]))]));

            assert_eq!(shown(&engine), [1, 2, 3, 4]);
        }

        #[test]
        fn ignoring_twice_or_restoring_what_is_not_ignored_saves_nothing_more() {
            let (engine, settings) = scanned(MemorySettings::default());
            let events = engine.subscribe();

            engine.ignore(&finding(1)).unwrap();
            engine.ignore(&category("beta.logs")).unwrap();
            engine.ignore(&category("beta.logs")).unwrap();
            engine.restore_ignored(&IgnoredEntry::Path { path: "/elsewhere".into() }).unwrap();

            assert_eq!(settings.saves(), 2);
            assert_eq!(events.try_iter().count(), 2);
        }

        #[test]
        fn a_read_only_file_refuses_ignore_restore_and_reset() {
            let newer = StoreError::NewerVersion { found: FormatVersion::new(2), supported: FormatVersion::new(1) };
            let (engine, settings) = scanned(MemorySettings::unloadable(newer));

            assert_eq!(engine.ignore(&finding(1)), Err(CommandError::SettingsReadOnly));
            assert_eq!(
                engine.restore_ignored(&IgnoredEntry::Category { category: "beta.logs".into() }),
                Ok(()),
                "nothing to restore, nothing to save"
            );
            let changed = SettingUpdate { key: "core.motion".into(), value: SettingValue::Choice("reduced".into()) };
            assert_eq!(engine.update_setting(changed), Err(CommandError::SettingsReadOnly));
            assert_eq!(settings.saves(), 0);
        }

        #[test]
        fn a_change_of_a_setting_that_applies_now_does_not_mark_the_scan() {
            let (engine, _) = scanned(MemorySettings::default());

            let motion = SettingUpdate { key: "core.motion".into(), value: SettingValue::Choice("reduced".into()) };
            engine.update_setting(motion).unwrap();
            engine.ignore(&finding(1)).unwrap();

            assert!(!overview(&engine).last_scan.unwrap().settings_changed);
        }

        #[test]
        fn reset_keeps_the_ignore_lists_and_a_reset_of_defaults_publishes_nothing() {
            let stored = [
                ("core.ignore.categories", json!(["beta.logs"])),
                ("core.ignore.paths", json!(["/home/a"])),
                ("core.ignore.modules", json!(["alpha"])),
                ("core.ignore.groups", json!(["development"])),
            ];
            let (engine, settings) = scanned(MemorySettings::with(&stored));
            let events = engine.subscribe();

            engine.reset_settings().unwrap();

            assert_eq!(settings.saves(), 0);
            assert_eq!(events.try_iter().count(), 0);
            let four = fresher_core::IgnoredCount { categories: 1, paths: 1, modules: 1, groups: 1 };
            assert_eq!(overview(&engine).ignored, four);
        }

        #[test]
        fn reset_is_refused_on_a_read_only_file() {
            let (engine, _) = scanned(MemorySettings::unloadable(StoreError::Io { kind: std::io::ErrorKind::Other }));

            assert_eq!(engine.reset_settings(), Err(CommandError::SettingsReadOnly));
        }
    }

    mod positive {
        use fresher_core::{Event, IgnoredCount, IgnoredEntry, SettingKey, SettingUpdate, SettingValue};
        use serde_json::json;

        use super::{category, finding, overview, scanned, shown};
        use crate::testing::{MemorySettings, scan};

        #[test]
        fn ignoring_saves_the_lists_publishes_the_change_and_counts_them() {
            let (engine, settings) = scanned(MemorySettings::default());
            let events = engine.subscribe();

            engine.ignore(&category("beta.logs")).unwrap();
            engine.ignore(&finding(3)).unwrap();

            assert_eq!(shown(&engine), [1]);
            assert_eq!(settings.stored().values.get("core.ignore.categories"), Some(&json!(["beta.logs"])));
            assert_eq!(settings.stored().values.get("core.ignore.paths"), Some(&json!(["/home/b/Caches"])));
            let published: Vec<Event> = events.try_iter().collect();
            assert_eq!(
                published.last(),
                Some(&Event::SettingChanged {
                    key: SettingKey::new("core.ignore.paths").unwrap(),
                    value: SettingValue::Paths(vec!["/home/b/Caches".into()]),
                })
            );
            assert_eq!(overview(&engine).ignored, IgnoredCount { categories: 1, paths: 1, ..IgnoredCount::default() });
            assert_eq!(overview(&engine).totals.len(), 1);
        }

        #[test]
        fn restore_brings_the_findings_back_without_a_rescan() {
            let (engine, _) = scanned(MemorySettings::default());
            engine.ignore(&category("alpha.caches")).unwrap();
            engine.ignore(&finding(2)).unwrap();

            engine.restore_ignored(&IgnoredEntry::Category { category: "alpha.caches".into() }).unwrap();
            engine.restore_ignored(&IgnoredEntry::Path { path: "/home/Logs".into() }).unwrap();

            assert_eq!(shown(&engine), [1, 2, 3, 4]);
            assert_eq!(overview(&engine).ignored, IgnoredCount::default());
        }

        #[test]
        fn a_change_of_the_roots_marks_the_last_scan_until_the_next_scan() {
            let (engine, _) = scanned(MemorySettings::default());
            let roots = SettingValue::Folders(vec!["~/Projects".into()]);

            engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
            assert!(overview(&engine).last_scan.unwrap().settings_changed);

            scan(&engine);
            assert!(!overview(&engine).last_scan.unwrap().settings_changed);
        }

        #[test]
        fn reset_saves_once_and_announces_each_change() {
            let stored = [
                ("core.motion", json!("reduced")),
                ("alpha.enabled", json!(false)),
                ("core.ignore.paths", json!(["/home/a"])),
                ("gone.key", json!(1)),
            ];
            let (engine, settings) = scanned(MemorySettings::with(&stored));
            let events = engine.subscribe();

            engine.reset_settings().unwrap();

            assert_eq!(settings.saves(), 1);
            assert_eq!(settings.stored().values.keys().collect::<Vec<_>>(), ["core.ignore.paths"]);
            let changed: Vec<String> = events
                .try_iter()
                .filter_map(|event| match event {
                    Event::SettingChanged { key, .. } => Some(key.as_str().to_owned()),
                    _ => None,
                })
                .collect();
            assert_eq!(changed, ["core.motion", "alpha.enabled"]);
            assert!(overview(&engine).last_scan.unwrap().settings_changed);
            assert_eq!(engine.settings().unwrap().problems, []);
        }
    }
}
