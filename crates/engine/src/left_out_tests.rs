//! Tests of the left-outs in a scan: added up per category and reason, sent with the module's `ModuleFinished`.

use fresher_core::{CategoryId, Event, LeftOutCount, LeftOutItem, ModuleId, TextKey};

use crate::testing::bytes;

/// A left-out of `size` bytes of category `category` (of the module its first segment names) for `reason`.
fn item(category: &'static str, reason: &str, size: u64) -> LeftOutItem {
    let module = category.split_once('.').map_or(category, |(module, _)| module);
    let category = CategoryId::new(ModuleId::new(module).unwrap(), category).unwrap();
    LeftOutItem { category, reason: TextKey::new(reason).unwrap(), sizes: bytes(size) }
}

/// The left-outs of the `ModuleFinished` of module `module` in `events`.
fn left_outs(events: &[Event], module: &str) -> Vec<LeftOutCount> {
    let finished = events.iter().find_map(|event| match event {
        Event::ModuleFinished { module: id, left_out, .. } if id.as_str() == module => Some(left_out.clone()),
        _ => None,
    });
    finished.unwrap()
}

/// Each left-out as (category, reason key, items, logical bytes).
fn lines(counts: &[LeftOutCount]) -> Vec<(&str, &str, u32, u64)> {
    counts.iter().map(line).collect()
}

fn line(count: &LeftOutCount) -> (&str, &str, u32, u64) {
    (count.category.as_str(), count.reason.key().as_str(), count.items, count.sizes.logical.bytes())
}

mod negative {
    use fresher_core::{DangerLevel, Event, ModuleError, ModuleGroup, ModuleOutcome};

    use serde_json::json;

    use super::{item, left_outs, lines};
    use crate::testing::{
        Fixture, MemorySettings, ScriptedWalk, TestModule, bytes, panic_now, scan, text, wait_for_finish,
    };

    #[test]
    fn an_undeclared_left_out_is_counted_and_logged() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .on_finish(|_, out| {
                out.left_out(item("alpha.caches", "alpha.left-out.unknown", 3));
                out.left_out(item("alpha.caches", "alpha.left-out.unknown", 4));
                Ok(())
            });
        let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

        let events = scan(&engine);

        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.caches", "alpha.left-out.unknown", 2, 7)]);
    }

    #[test]
    fn left_outs_of_a_failed_module_are_kept() {
        let failing = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open", "alpha.error.broken"])
            .on_finish(|_, out| {
                out.left_out(item("alpha.caches", "alpha.left-out.open", 1));
                Err(ModuleError::Failed { reason: text("alpha.error.broken") })
            });
        let panicking = TestModule::builder("beta")
            .category("beta.caches", DangerLevel::Safe)
            .problems(&["beta.left-out.open"])
            .on_finish(|_, out| {
                out.left_out(item("beta.caches", "beta.left-out.open", 2));
                panic_now("a module panics after a left-out")
            });
        let cancelled = TestModule::builder("gamma")
            .category("gamma.caches", DangerLevel::Safe)
            .problems(&["gamma.left-out.open"])
            .on_finish(|_, out| {
                out.left_out(item("gamma.caches", "gamma.left-out.open", 4));
                Err(ModuleError::Cancelled)
            });
        let engine = Fixture::new(ScriptedWalk::new()).module(failing).module(panicking).module(cancelled).engine();

        let events = scan(&engine);

        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.caches", "alpha.left-out.open", 1, 1)]);
        assert_eq!(lines(&left_outs(&events, "beta")), [("beta.caches", "beta.left-out.open", 1, 2)]);
        assert_eq!(lines(&left_outs(&events, "gamma")), [("gamma.caches", "gamma.left-out.open", 1, 4)]);
        let outcomes: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::ModuleFinished { outcome, .. } => Some(outcome.clone()),
                _ => None,
            })
            .collect();
        assert!(matches!(
            outcomes.as_slice(),
            [ModuleOutcome::Failed { .. }, ModuleOutcome::Failed { .. }, ModuleOutcome::Cancelled]
        ));
    }

    #[test]
    fn a_left_out_before_an_error_or_a_panic_on_a_match_is_sent() {
        let failing = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .folder("Caches")
            .left_out_on_match(item("alpha.caches", "alpha.left-out.open", 1))
            .failing_on_match();
        let panicking = TestModule::builder("beta")
            .category("beta.logs", DangerLevel::Safe)
            .problems(&["beta.left-out.open"])
            .folder("Logs")
            .left_out_on_match(item("beta.logs", "beta.left-out.open", 2))
            .panicking_on_match();
        let walk =
            ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1)).folder("Logs", "/home/Logs", bytes(1));
        let engine = Fixture::new(walk).module(failing).module(panicking).engine();

        let events = scan(&engine);

        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.caches", "alpha.left-out.open", 1, 1)]);
        assert_eq!(lines(&left_outs(&events, "beta")), [("beta.logs", "beta.left-out.open", 1, 2)]);
    }

    #[test]
    fn a_cancelled_scan_sends_the_left_outs_of_its_matches() {
        let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(1));
        let (walk, _gate) = walk.hold();
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .folder("Caches")
            .left_out_on_match(item("alpha.caches", "alpha.left-out.open", 1));
        let engine = Fixture::new(walk).module(module).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();

        engine.cancel_scan().unwrap();

        let events = wait_for_finish(&events);
        let cancelled =
            events.iter().any(|event| matches!(event, Event::ModuleFinished { outcome: ModuleOutcome::Cancelled, .. }));
        assert!(cancelled);
        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.caches", "alpha.left-out.open", 1, 1)]);
    }

    #[test]
    fn a_left_out_of_a_category_not_its_own_is_dropped() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .on_finish(|_, out| {
                out.left_out(item("beta.caches", "alpha.left-out.open", 1));
                out.left_out(item("alpha.caches", "alpha.left-out.open", 2));
                Ok(())
            });
        let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

        let events = scan(&engine);

        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.caches", "alpha.left-out.open", 1, 2)]);
    }

    #[test]
    fn a_left_out_of_an_ignored_category_is_not_counted() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .category("alpha.logs", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .on_finish(|_, out| {
                out.left_out(item("alpha.caches", "alpha.left-out.open", 1));
                out.left_out(item("alpha.logs", "alpha.left-out.open", 2));
                Ok(())
            });
        let settings = MemorySettings::with(&[("core.ignore.categories", json!(["alpha.caches"]))]);
        let engine = Fixture::new(ScriptedWalk::new()).module(module).settings(settings).engine();

        let events = scan(&engine);

        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.logs", "alpha.left-out.open", 1, 2)]);
    }

    #[test]
    fn a_left_out_of_an_ignored_module_or_group_is_not_counted() {
        let module = |id: &'static str, category: &'static str, reason: &'static str| {
            TestModule::builder(id).category(category, DangerLevel::Safe).problems(&[reason]).on_finish(
                move |_, out| {
                    out.left_out(item(category, reason, 1));
                    Ok(())
                },
            )
        };
        let ignoring = |key: &str, value| {
            let settings = MemorySettings::with(&[(key, value)]);
            let alpha = module("alpha", "alpha.caches", "alpha.left-out.open");
            let modules = [alpha, module("beta", "beta.logs", "beta.left-out.open").group(ModuleGroup::Debug)];
            let fixture = modules.into_iter().fold(Fixture::new(ScriptedWalk::new()), Fixture::module);
            scan(&fixture.settings(settings).engine())
        };

        let by_module = ignoring("core.ignore.modules", json!(["alpha"]));
        let by_group = ignoring("core.ignore.groups", json!(["debug"]));

        assert_eq!(lines(&left_outs(&by_module, "alpha")), []);
        assert_eq!(lines(&left_outs(&by_module, "beta")), [("beta.logs", "beta.left-out.open", 1, 1)]);
        assert_eq!(lines(&left_outs(&by_group, "alpha")), [("alpha.caches", "alpha.left-out.open", 1, 1)]);
        assert_eq!(lines(&left_outs(&by_group, "beta")), []);
    }

    #[test]
    fn left_outs_hold_no_path() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .folder("Caches")
            .on_finish(|_, out| {
                out.left_out(item("alpha.caches", "alpha.left-out.open", 1));
                Ok(())
            });
        let walk = ScriptedWalk::new().folder("Caches", "/home/Library/Caches", bytes(1));
        let engine = Fixture::new(walk).module(module).engine();

        let events = scan(&engine);

        let finished = events.iter().find(|event| matches!(event, Event::ModuleFinished { .. })).unwrap();
        let json = serde_json::to_string(finished).unwrap();
        assert!(json.contains("alpha.left-out.open"), "{json}");
        assert!(!json.contains("/home"), "{json}");
        assert!(!json.contains("Library"), "{json}");
    }
}

mod positive {
    use fresher_core::DangerLevel;

    use super::{item, left_outs, lines};
    use crate::testing::{Fixture, ScriptedWalk, TestModule, scan};

    #[test]
    fn left_outs_of_one_category_and_reason_add_up() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .problems(&["alpha.left-out.open"])
            .on_finish(|_, out| {
                for size in [1, 2, 3] {
                    out.left_out(item("alpha.caches", "alpha.left-out.open", size));
                }
                Ok(())
            });
        let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

        let events = scan(&engine);

        assert_eq!(lines(&left_outs(&events, "alpha")), [("alpha.caches", "alpha.left-out.open", 3, 6)]);
    }

    #[test]
    fn module_finished_lists_left_outs_in_order() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .category("alpha.logs", DangerLevel::Safe)
            .problems(&["alpha.left-out.open", "alpha.left-out.busy"])
            .on_finish(|_, out| {
                out.left_out(item("alpha.logs", "alpha.left-out.open", 1));
                out.left_out(item("alpha.caches", "alpha.left-out.open", 2));
                out.left_out(item("alpha.caches", "alpha.left-out.busy", 4));
                Ok(())
            });
        let quiet = TestModule::builder("beta");
        let engine = Fixture::new(ScriptedWalk::new()).module(module).module(quiet).engine();

        let events = scan(&engine);

        assert_eq!(
            lines(&left_outs(&events, "alpha")),
            [
                ("alpha.caches", "alpha.left-out.busy", 1, 4),
                ("alpha.caches", "alpha.left-out.open", 1, 2),
                ("alpha.logs", "alpha.left-out.open", 1, 1),
            ]
        );
        assert_eq!(left_outs(&events, "beta"), []);
    }
}
