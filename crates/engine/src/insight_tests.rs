//! Tests of the insights in a scan: sent with the module's `ModuleFinished`, in order, never counted.

use fresher_core::{ByteSize, DangerLevel, Event, Figure, Insight, InsightPart, ShownCommand, ToolId};

use crate::testing::text;

/// An insight called `identity` with one figure and one part of `size` bytes and a shown command.
fn insight(identity: &str, size: u64) -> Insight {
    let command = ShownCommand::new(ToolId::new("demo").unwrap(), vec!["prune".into()]).unwrap();
    let part = InsightPart {
        label: text("alpha.insight.part"),
        size: ByteSize::new(size),
        detail: text("alpha.insight.detail"),
        effect: text("alpha.insight.effect"),
        danger: DangerLevel::Safe,
        command: Some(command),
    };
    let figure = Figure { label: text("alpha.insight.figure"), size: ByteSize::new(size) };
    Insight::new(identity, text("alpha.insight.title"), text("alpha.insight.summary"), vec![figure], vec![part], vec![])
        .unwrap()
}

/// The insights of the `ModuleFinished` of module `module` in `events`.
fn insights(events: &[Event], module: &str) -> Vec<Insight> {
    let finished = events.iter().find_map(|event| match event {
        Event::ModuleFinished { module: id, insights, .. } if id.as_str() == module => Some(insights.clone()),
        _ => None,
    });
    finished.unwrap()
}

/// The identity and the first figure's size of each insight.
fn lines(insights: &[Insight]) -> Vec<(&str, u64)> {
    insights.iter().map(|insight| (insight.identity(), insight.figures()[0].size.bytes())).collect()
}

mod negative {
    use fresher_core::{DangerLevel, FindingSink, NewFinding};

    use super::{insight, insights, lines};
    use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

    #[test]
    fn a_repeated_identity_is_dropped() {
        let module = TestModule::builder("alpha").on_finish(|_, out| {
            out.describe(insight("orbstack", 1));
            out.describe(insight("orbstack", 2));
            out.describe(insight("docker-desktop", 3));
            Ok(())
        });
        let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

        let events = scan(&engine);

        assert_eq!(lines(&insights(&events, "alpha")), [("orbstack", 1), ("docker-desktop", 3)]);
    }

    #[test]
    fn insights_add_to_no_total() {
        let module = TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .folder("Caches")
            .on_finish(|_, out| {
                out.describe(insight("orbstack", 1_000_000_000));
                Ok(())
            });
        let walk = ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(5));
        let engine = Fixture::new(walk).module(module).engine();

        let events = scan(&engine);

        assert_eq!(lines(&insights(&events, "alpha")), [("orbstack", 1_000_000_000)]);
        let overview = engine.overview().unwrap();
        assert_eq!(overview.found.logical.bytes(), 5);
        assert_eq!(overview.found.allocated.bytes(), 5);
        let totals: Vec<_> =
            overview.totals.iter().map(|total| (total.category.as_str(), total.sizes.logical.bytes())).collect();
        assert_eq!(totals, [("alpha.caches", 5)]);
    }

    #[test]
    fn a_sink_that_keeps_findings_drops_insights() {
        let mut findings: Vec<NewFinding> = Vec::new();

        findings.describe(insight("orbstack", 1));

        assert_eq!(findings, []);
    }
}

mod positive {
    use fresher_core::{Event, ModuleError, ModuleOutcome};

    use super::{insight, insights, lines};
    use crate::testing::{Fixture, ScriptedWalk, TestModule, panic_now, scan, text};

    #[test]
    fn a_module_s_insights_reach_its_module_finished_in_order() {
        let module = TestModule::builder("alpha").on_finish(|_, out| {
            out.describe(insight("orbstack", 2));
            out.describe(insight("docker-desktop", 1));
            Ok(())
        });
        let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

        let events = scan(&engine);

        let described = insights(&events, "alpha");
        assert_eq!(lines(&described), [("orbstack", 2), ("docker-desktop", 1)]);
        assert_eq!(described[0], insight("orbstack", 2));
    }

    #[test]
    fn insights_given_before_a_failure_are_sent() {
        let failing = TestModule::builder("alpha").problems(&["alpha.error.broken"]).on_finish(|_, out| {
            out.describe(insight("orbstack", 1));
            Err(ModuleError::Failed { reason: text("alpha.error.broken") })
        });
        let panicking = TestModule::builder("beta").on_finish(|_, out| {
            out.describe(insight("docker-desktop", 2));
            panic_now("a module panics after an insight")
        });
        let cancelled = TestModule::builder("gamma").on_finish(|_, out| {
            out.describe(insight("third-runtime", 4));
            Err(ModuleError::Cancelled)
        });
        let engine = Fixture::new(ScriptedWalk::new()).module(failing).module(panicking).module(cancelled).engine();

        let events = scan(&engine);

        assert_eq!(lines(&insights(&events, "alpha")), [("orbstack", 1)]);
        assert_eq!(lines(&insights(&events, "beta")), [("docker-desktop", 2)]);
        assert_eq!(lines(&insights(&events, "gamma")), [("third-runtime", 4)]);
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
    fn a_module_without_insights_sends_an_empty_list() {
        let module = TestModule::builder("alpha");
        let engine = Fixture::new(ScriptedWalk::new()).module(module).engine();

        let events = scan(&engine);

        assert_eq!(insights(&events, "alpha"), []);
    }
}
