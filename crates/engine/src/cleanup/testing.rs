//! What the cleanup's tests share: an engine with two modules over a scripted walk, selections and confirmations as
//! the UI sends them.

use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_core::{
    CleanupPlanView, CleanupReport, CleanupSelection, Confirmation, DangerLevel, Event, FindingId, PlanItemView,
};
use serde_json::json;

use crate::Engine;
use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, scan};

/// Modules `alpha` (`alpha.caches`, safe, every `Caches` folder) and `beta` (`beta.logs`, low, every `Logs` folder),
/// over a walk that finds, as findings 1 to 4: `/home/a/Caches` (10 bytes), `/home/Logs` (20), `/home/b/Caches` (30),
/// `/home/b/Caches/x/Caches` (5, inside finding 3).
pub(crate) fn fixture() -> Fixture {
    let walk = ScriptedWalk::new()
        .folder("Caches", "/home/a/Caches", bytes(10))
        .folder("Logs", "/home/Logs", bytes(20))
        .folder("Caches", "/home/b/Caches", bytes(30))
        .folder("Caches", "/home/b/Caches/x/Caches", bytes(5));
    let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches");
    let beta = TestModule::builder("beta").category("beta.logs", DangerLevel::Low).folder("Logs");
    Fixture::new(walk).module(alpha).module(beta)
}

/// The engine of `fixture`, after one scan.
pub(crate) fn scanned(fixture: Fixture) -> Engine {
    let engine = fixture.engine();
    scan(&engine);
    engine
}

/// Findings `findings` picked one by one, and each `(category, except)` picked whole.
pub(crate) fn select(findings: &[u32], categories: &[(&str, &[u32])]) -> CleanupSelection {
    let categories: Vec<_> =
        categories.iter().map(|(category, except)| json!({ "category": category, "except": except })).collect();
    serde_json::from_value(json!({ "findings": findings, "categories": categories })).unwrap()
}

/// The confirmation the UI sends for `view` by `method`, with `change` applied to its JSON.
pub(crate) fn echo_with(
    view: &CleanupPlanView,
    method: &str,
    change: impl FnOnce(&mut serde_json::Value),
) -> Confirmation {
    let summary = view.summary;
    let mut json = json!({
        "plan": view.plan,
        "count": summary.count,
        "allocated": summary.sizes.allocated,
        "highest": summary.highest,
        "method": method,
    });
    change(&mut json);
    serde_json::from_value(json).unwrap()
}

/// The confirmation the UI sends for `view` by `method`, as shown.
pub(crate) fn echo(view: &CleanupPlanView, method: &str) -> Confirmation {
    echo_with(view, method, |_| {})
}

/// The findings of the view's items, group by group.
pub(crate) fn items(view: &CleanupPlanView) -> Vec<u32> {
    view.groups.iter().flat_map(|group| group.items.iter().map(|item: &PlanItemView| item.finding.get())).collect()
}

/// The findings of the view's left-out list.
pub(crate) fn left_out(view: &CleanupPlanView) -> Vec<FindingId> {
    view.left_out.iter().map(fresher_core::LeftOut::finding).collect()
}

/// The events up to and including the next `CleanupFinished`; fails after 10 s instead of hanging.
pub(crate) fn wait_for_cleanup(events: &Receiver<Event>) -> Vec<Event> {
    let mut received = Vec::new();
    loop {
        let event = events.recv_timeout(Duration::from_secs(10)).unwrap();
        let finished = matches!(event, Event::CleanupFinished { .. });
        received.push(event);
        if finished {
            return received;
        }
    }
}

/// The report of the last `CleanupFinished` of `events`.
pub(crate) fn report(events: &[Event]) -> CleanupReport {
    match events.last() {
        Some(Event::CleanupFinished { report, .. }) => *report,
        other => crate::testing::panic_now(&format!("not a finished cleanup: {other:?}")),
    }
}
