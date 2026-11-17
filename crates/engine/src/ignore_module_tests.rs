//! Tests of ignoring a module and a group of modules: their findings leave every answer at once and come back on
//! restore, without a rescan.

use std::sync::Arc;

use fresher_core::{DangerLevel, FindingOrder, FindingsQuery, IgnoreTarget, ModuleGroup};

use crate::Engine;
use crate::testing::{Fixture, MemorySettings, ScriptedWalk, TestModule, bytes, scan};

fn module(id: &str) -> IgnoreTarget {
    IgnoreTarget::Module { module: id.into() }
}

const fn group(group: ModuleGroup) -> IgnoreTarget {
    IgnoreTarget::Group { group }
}

/// Modules `alpha` (`alpha.caches`, every `Caches` folder) of group `alpha_group` and `beta` (`beta.logs`, every
/// `Logs` folder) of group Development over `settings`, after one scan that finds `/home/a/Caches` (finding 1),
/// `/home/Logs` (2) and `/home/b/Caches` (3); and its settings.
fn scanned(alpha_group: ModuleGroup, settings: MemorySettings) -> (Engine, Arc<MemorySettings>) {
    let walk = ScriptedWalk::new()
        .folder("Caches", "/home/a/Caches", bytes(10))
        .folder("Logs", "/home/Logs", bytes(20))
        .folder("Caches", "/home/b/Caches", bytes(30));
    let alpha =
        TestModule::builder("alpha").group(alpha_group).category("alpha.caches", DangerLevel::Safe).folder("Caches");
    let beta = TestModule::builder("beta").category("beta.logs", DangerLevel::Low).folder("Logs");
    let fixture = Fixture::new(walk).module(alpha).module(beta).settings(settings);
    let settings = Arc::clone(&fixture.settings);
    let engine = fixture.engine();
    scan(&engine);
    (engine, settings)
}

/// The ids of every finding the engine answers, in the order found.
fn shown(engine: &Engine) -> Vec<u32> {
    let query = FindingsQuery::new(None, None, FindingOrder::Found, 0, 100);
    engine.findings(&query).unwrap().findings.iter().map(|finding| finding.id().get()).collect()
}

mod negative {
    use fresher_core::{CommandError, FindingId, FormatVersion, IgnoredEntry, ModuleGroup, StoreError};

    use super::{group, module, scanned, shown};
    use crate::cleanup::testing::{echo, select};
    use crate::testing::MemorySettings;

    #[test]
    fn an_unknown_module_or_group_is_refused_and_nothing_is_saved() {
        let (engine, settings) = scanned(ModuleGroup::Development, MemorySettings::default());

        assert_eq!(engine.ignore(&module("gamma")), Err(CommandError::UnknownModule { id: "gamma".into() }));
        assert_eq!(engine.ignore(&module("Alpha")), Err(CommandError::UnknownModule { id: "Alpha".into() }));
        assert_eq!(
            engine.ignore(&group(ModuleGroup::Debug)),
            Err(CommandError::UnknownGroup { id: "debug".into() }),
            "no registered module is of the group"
        );
        assert_eq!(settings.saves(), 0);
        assert_eq!(shown(&engine), [1, 2, 3]);
    }

    #[test]
    fn an_ignored_modules_findings_are_in_no_answer_and_cannot_be_selected() {
        let (engine, _) = scanned(ModuleGroup::Development, MemorySettings::default());

        engine.ignore(&module("alpha")).unwrap();

        assert_eq!(shown(&engine), [2]);
        let totals: Vec<_> = engine.overview().unwrap().totals.iter().map(|total| total.category.as_str()).collect();
        assert_eq!(totals, ["beta.logs"]);
        assert_eq!(engine.finding(FindingId::new(1)), Err(CommandError::UnknownFinding { id: FindingId::new(1) }));
        assert_eq!(
            engine.prepare_cleanup(&select(&[3], &[])),
            Err(CommandError::UnknownFinding { id: FindingId::new(3) })
        );
        assert_eq!(engine.prepare_cleanup(&select(&[], &[("alpha.caches", &[])])), Err(CommandError::NothingToClean));
    }

    #[test]
    fn a_module_restored_inside_an_ignored_group_stays_hidden() {
        let (engine, _) = scanned(ModuleGroup::Debug, MemorySettings::default());
        engine.ignore(&module("alpha")).unwrap();
        engine.ignore(&group(ModuleGroup::Debug)).unwrap();

        engine.restore_ignored(&IgnoredEntry::Module { module: "alpha".into() }).unwrap();

        assert_eq!(shown(&engine), [2]);
    }

    #[test]
    fn ignoring_twice_or_restoring_what_is_not_ignored_publishes_nothing() {
        let (engine, settings) = scanned(ModuleGroup::Development, MemorySettings::default());
        let events = engine.subscribe();

        engine.ignore(&module("alpha")).unwrap();
        engine.ignore(&module("alpha")).unwrap();
        engine.ignore(&group(ModuleGroup::Development)).unwrap();
        engine.ignore(&group(ModuleGroup::Development)).unwrap();
        engine.restore_ignored(&IgnoredEntry::Module { module: "beta".into() }).unwrap();
        engine.restore_ignored(&IgnoredEntry::Group { group: ModuleGroup::Debug }).unwrap();

        assert_eq!(settings.saves(), 2);
        assert_eq!(events.try_iter().count(), 2);
    }

    #[test]
    fn a_read_only_file_refuses_both_kinds() {
        let newer = StoreError::NewerVersion { found: FormatVersion::new(2), supported: FormatVersion::new(1) };
        let (engine, settings) = scanned(ModuleGroup::Development, MemorySettings::unloadable(newer));

        assert_eq!(engine.ignore(&module("alpha")), Err(CommandError::SettingsReadOnly));
        assert_eq!(engine.ignore(&group(ModuleGroup::Development)), Err(CommandError::SettingsReadOnly));
        assert_eq!(settings.saves(), 0);
        assert_eq!(shown(&engine), [1, 2, 3]);
    }

    #[test]
    fn a_change_of_either_list_kills_the_pending_plan() {
        let (engine, _) = scanned(ModuleGroup::Debug, MemorySettings::default());
        let first = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
        engine.ignore(&module("beta")).unwrap();
        assert_eq!(engine.confirm_cleanup(&echo(&first, "trash")), Err(CommandError::PlanGone));

        let second = engine.prepare_cleanup(&select(&[1], &[])).unwrap();
        engine.ignore(&group(ModuleGroup::Development)).unwrap();
        assert_eq!(engine.confirm_cleanup(&echo(&second, "trash")), Err(CommandError::PlanGone));
    }
}

mod positive {
    use fresher_core::{
        Event, HiddenBy, HiddenModule, IgnoredCount, IgnoredEntry, ModuleGroup, ModuleId, SettingKey, SettingValue,
    };
    use serde_json::json;

    use super::{group, module, scanned, shown};
    use crate::testing::MemorySettings;

    #[test]
    fn an_ignored_group_hides_every_module_of_it_and_only_those() {
        let (engine, settings) = scanned(ModuleGroup::Debug, MemorySettings::default());
        let events = engine.subscribe();

        engine.ignore(&group(ModuleGroup::Debug)).unwrap();

        assert_eq!(shown(&engine), [2]);
        assert_eq!(settings.stored().values.get("core.ignore.groups"), Some(&json!(["debug"])));
        assert_eq!(
            events.try_iter().collect::<Vec<_>>(),
            [Event::SettingChanged {
                key: SettingKey::new("core.ignore.groups").unwrap(),
                value: SettingValue::Ids(vec!["debug".into()]),
            }]
        );
    }

    #[test]
    fn restoring_the_group_brings_back_what_is_not_ignored_on_its_own_without_a_rescan() {
        let (engine, _) = scanned(ModuleGroup::Development, MemorySettings::default());
        engine.ignore(&module("beta")).unwrap();
        engine.ignore(&group(ModuleGroup::Development)).unwrap();
        assert_eq!(shown(&engine), Vec::<u32>::new());

        engine.restore_ignored(&IgnoredEntry::Group { group: ModuleGroup::Development }).unwrap();
        assert_eq!(shown(&engine), [1, 3]);

        engine.restore_ignored(&IgnoredEntry::Module { module: "beta".into() }).unwrap();
        assert_eq!(shown(&engine), [1, 2, 3]);
    }

    #[test]
    fn the_overview_counts_the_four_lists() {
        let stored = [("core.ignore.categories", json!(["beta.logs"])), ("core.ignore.paths", json!(["/home/a"]))];
        let (engine, settings) = scanned(ModuleGroup::Debug, MemorySettings::with(&stored));

        engine.ignore(&module("alpha")).unwrap();
        engine.ignore(&group(ModuleGroup::Debug)).unwrap();

        let ignored = engine.overview().unwrap().ignored;
        assert_eq!(ignored, IgnoredCount { categories: 1, paths: 1, modules: 1, groups: 1 });
        assert_eq!(settings.stored().values.get("core.ignore.modules"), Some(&json!(["alpha"])));
    }

    #[test]
    fn a_stored_list_hides_its_modules_from_the_start() {
        let stored = [("core.ignore.modules", json!(["alpha", "gone"]))];
        let (engine, _) = scanned(ModuleGroup::Development, MemorySettings::with(&stored));

        assert_eq!(shown(&engine), [2]);
        assert_eq!(engine.overview().unwrap().ignored.modules, 2, "a module no longer registered is still listed");
    }

    #[test]
    fn the_overview_says_which_modules_are_hidden_and_why() {
        let (engine, _) = scanned(ModuleGroup::Debug, MemorySettings::default());
        let hidden = |id: &'static str, by| HiddenModule { module: ModuleId::new(id).unwrap(), by };
        assert_eq!(engine.overview().unwrap().hidden_modules, []);

        engine.ignore(&module("beta")).unwrap();
        assert_eq!(engine.overview().unwrap().hidden_modules, [hidden("beta", HiddenBy::Module)]);

        engine.ignore(&module("alpha")).unwrap();
        engine.ignore(&group(ModuleGroup::Debug)).unwrap();
        assert_eq!(
            engine.overview().unwrap().hidden_modules,
            [hidden("alpha", HiddenBy::Group), hidden("beta", HiddenBy::Module)]
        );
    }
}
