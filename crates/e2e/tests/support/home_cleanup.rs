//! A flow home's cleanups and their report: the confirmation as the dialog gives it, how each item ended, the
//! journal's records and a report written relative to the sandbox. Pulled in with `#[path]`, beside `home.rs`,
//! `events.rs` and `report.rs`, only by the homes that use it.

use std::collections::BTreeMap;
use std::fs;

use fresher_core::{CleanupId, CleanupPlanView, Confirmation, DangerLevel, Event, ItemOutcome};
use serde_json::{Value, json};

use super::events::until;
use super::home::Flow;
use super::report::{sort_by_path, strip, trash_unnumbered};

impl Flow {
    /// Confirms `view` to the Trash as the dialog shows it and waits for the end.
    pub(crate) fn clean(&self, view: &CleanupPlanView) -> (CleanupId, Vec<Event>) {
        let events = self.engine.subscribe();
        let summary = view.summary;
        let echo = json!({ "plan": view.plan, "count": summary.count, "allocated": summary.sizes.allocated,
                           "highest": summary.highest, "method": "trash",
                           "highRiskUnderstood": summary.highest == DangerLevel::High });
        let confirmation: Confirmation = serde_json::from_value(echo).unwrap();
        let cleanup = self.engine.confirm_cleanup(&confirmation).unwrap();
        (cleanup, until(&events, |event| matches!(event, Event::CleanupFinished { .. })))
    }

    /// How each item of `view` ended in the cleanup's `events`, by its path relative to the sandbox, sorted.
    pub(crate) fn outcomes(&self, view: &CleanupPlanView, events: &[Event]) -> Vec<(String, ItemOutcome)> {
        let paths: BTreeMap<_, _> = view
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .filter_map(|item| Some((item.finding, self.relative_path(item.path.as_ref()?))))
            .collect();
        let mut ended: Vec<(String, ItemOutcome)> = events
            .iter()
            .filter_map(|event| match event {
                Event::CleanupItems { items, .. } => Some(items),
                _ => None,
            })
            .flatten()
            .map(|item| (paths[&item.finding].clone(), item.outcome.clone()))
            .collect();
        ended.sort_by(|left, right| left.0.cmp(&right.0));
        ended
    }

    /// The journal's records as JSON, without the items' identities (device and file numbers are the machine's), the
    /// item records sorted by the path they moved, in the places the item records take.
    pub(crate) fn journal(&self) -> Vec<Value> {
        let Ok(text) = fs::read_to_string(self.sandbox.data_folder().join("cleanups.jsonl")) else { return Vec::new() };
        let mut records: Vec<Value> = text
            .lines()
            .map(|line| {
                let mut record: Value = serde_json::from_str(line).unwrap();
                strip(&mut record, "identity");
                record
            })
            .collect();
        let at: Vec<usize> = (0..records.len()).filter(|&at| records[at]["type"] == "item").collect();
        let mut items: Vec<Value> = at.iter().map(|&at| records[at].clone()).collect();
        items.sort_by_key(|item| item["trashed"][0]["original"].as_str().unwrap_or_default().to_owned());
        for (&at, item) in at.iter().zip(items) {
            records[at] = item;
        }
        records
    }

    /// `report` with the sandbox's path written `~`, the fake Trash's numbering dropped, the findings' ids and the
    /// items' indexes removed (the walk's order is not the report's), and the lists of paths sorted by path.
    pub(crate) fn relative(&self, report: &Value) -> Value {
        let text = serde_json::to_string(report).unwrap().replace(&self.fixture.display().to_string(), "~");
        let mut report: Value = serde_json::from_str(&trash_unnumbered(&text, ".Trash/")).unwrap();
        for key in ["finding", "findings", "index"] {
            strip(&mut report, key);
        }
        sort_by_path(&mut report);
        report
    }
}
