//! The events the engine sends up to every subscriber (the UI, later the history).

use serde::Serialize;
use ts_rs::TS;

use crate::{
    AccessStatus, ByteSize, Capability, CleanupId, CleanupReport, Coverage, Finding, FindingId, ItemReport,
    LeftOutCount, Method, ModuleId, ScanId, SettingKey, SettingValue, Text, TextKey, Timestamp,
};

/// Something that happened; a discriminated union on `type` in TypeScript.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Event {
    ScanStarted {
        scan: ScanId,
        modules: Vec<ModuleId>,
    },
    ScanProgress {
        scan: ScanId,
        #[ts(type = "number")]
        entries: u64,
        bytes: ByteSize,
    },
    FindingsAdded {
        scan: ScanId,
        findings: Vec<Finding>,
    },
    ModuleFinished {
        scan: ScanId,
        module: ModuleId,
        outcome: ModuleOutcome,
        /// What the module recognized and did not offer, counted per category and reason, sorted by category, then by
        /// reason key. Not the `leftOut` of a cleanup plan's view: a different type with a different payload.
        #[serde(rename = "leftOut")]
        left_out: Vec<LeftOutCount>,
    },
    /// What the scan saw and what it did not; sent right before `ScanFinished`.
    ScanCoverage {
        scan: ScanId,
        coverage: Coverage,
    },
    ScanFinished {
        scan: ScanId,
        outcome: ScanOutcome,
        kind: ScanKind,
    },
    SettingChanged {
        key: SettingKey,
        value: SettingValue,
    },
    AccessChanged {
        status: AccessStatus,
    },
    /// A confirmed cleanup began: its `total` items are acted on one by one.
    CleanupStarted {
        cleanup: CleanupId,
        method: Method,
        total: u32,
    },
    /// The outcomes of the next items, in plan order; `done` items have ended so far.
    CleanupItems {
        cleanup: CleanupId,
        done: u32,
        items: Vec<ItemReport>,
    },
    CleanupFinished {
        cleanup: CleanupId,
        report: CleanupReport,
    },
    /// Findings of `scan` that a cleanup removed have left the results.
    FindingsRemoved {
        scan: ScanId,
        findings: Vec<FindingId>,
    },
    /// The window is closing while a cleanup runs: the UI asks the user first.
    QuitRequested,
}

/// How a scan read the disk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ScanKind {
    /// Every folder was read.
    #[default]
    Full,
    /// Only the folders changed since the last scan were read.
    Rescan {
        #[serde(rename = "changedFolders")]
        changed_folders: u32,
    },
    /// Nothing was read: the last scan, taken at `taken`, was restored from the cache.
    Restored { taken: Timestamp },
}

/// How one module's part of a scan ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ModuleOutcome {
    Completed,
    /// The module failed; the other modules finish.
    Failed {
        reason: Text,
    },
    Cancelled,
    /// Not run: every category of the module needs a capability that is not allowed. The UI says so with
    /// `core.module.unavailable` and the missing capabilities' texts.
    Unavailable {
        missing: Vec<Capability>,
    },
}

impl ModuleOutcome {
    /// The module panicked: the engine caught it, and the module counts as failed with the core's own reason.
    pub fn panicked() -> Self {
        Self::Failed { reason: Text::new(TextKey::from_valid("core.module.panicked".into())) }
    }

    /// The key of the sentence the UI shows for `Unavailable`, before the missing capabilities.
    pub fn unavailable_key() -> TextKey {
        TextKey::from_valid("core.module.unavailable".into())
    }
}

/// How a scan ended.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ScanOutcome {
    Completed,
    /// Cancelled, or some folders or modules could not be read: the findings so far are kept.
    Partial,
    Failed {
        reason: Text,
    },
}

#[cfg(test)]
mod tests {
    mod positive {
        use serde_json::json;
        use ts_rs::{Config, TS};

        use crate::{
            ByteSize, Event, ModuleId, ModuleOutcome, ScanId, ScanKind, ScanOutcome, Text, TextKey, Timestamp,
        };

        #[test]
        fn events_serialize_with_a_type_tag() {
            let progress = Event::ScanProgress { scan: ScanId::new(1), entries: 10, bytes: ByteSize::new(2048) };
            let failed = Event::ModuleFinished {
                scan: ScanId::new(1),
                module: ModuleId::new("demo").unwrap(),
                outcome: ModuleOutcome::Failed { reason: Text::new(TextKey::new("demo.error.broken").unwrap()) },
                left_out: vec![],
            };

            assert_eq!(
                serde_json::to_value(progress).unwrap(),
                json!({ "type": "scan-progress", "scan": 1, "entries": 10, "bytes": 2048 })
            );
            assert_eq!(
                serde_json::to_value(failed).unwrap(),
                json!({
                    "type": "module-finished",
                    "scan": 1,
                    "module": "demo",
                    "outcome": { "kind": "failed", "reason": { "key": "demo.error.broken", "args": {} } },
                    "leftOut": [],
                })
            );
            assert_eq!(
                serde_json::to_value(Event::ScanFinished {
                    scan: ScanId::new(1),
                    outcome: ScanOutcome::Partial,
                    kind: ScanKind::Full
                })
                .unwrap(),
                json!({ "type": "scan-finished", "scan": 1, "outcome": { "kind": "partial" }, "kind": { "kind": "full" } })
            );
        }

        #[test]
        fn module_finished_serializes_its_left_outs() {
            let category = crate::CategoryId::new(ModuleId::new("demo").unwrap(), "demo.caches").unwrap();
            let mut count = crate::LeftOutCount::new(category, Text::new(TextKey::new("demo.left-out.open").unwrap()));
            count.add(crate::Sizes::complete(ByteSize::new(1), ByteSize::new(4096)));
            let finished = Event::ModuleFinished {
                scan: ScanId::new(3),
                module: ModuleId::new("demo").unwrap(),
                outcome: ModuleOutcome::Completed,
                left_out: vec![count],
            };

            assert_eq!(
                serde_json::to_value(finished).unwrap(),
                json!({
                    "type": "module-finished",
                    "scan": 3,
                    "module": "demo",
                    "outcome": { "kind": "completed" },
                    "leftOut": [{
                        "category": "demo.caches",
                        "reason": { "key": "demo.left-out.open", "args": {} },
                        "items": 1,
                        "sizes": { "logical": 1, "allocated": 4096, "reclaimable": 4096, "completeness": "complete" },
                    }],
                })
            );
            assert!(Event::decl(&Config::new()).contains("leftOut: Array<LeftOutCount>"));
        }

        #[test]
        fn an_unavailable_module_names_what_is_missing() {
            let outcome = ModuleOutcome::Unavailable { missing: vec![crate::Capability::AllFiles] };

            assert_eq!(
                serde_json::to_value(outcome).unwrap(),
                json!({ "kind": "unavailable", "missing": [{ "kind": "all-files" }] })
            );
        }

        #[test]
        fn scan_finished_carries_the_scan_kind() {
            let rescan = Event::ScanFinished {
                scan: ScanId::new(2),
                outcome: ScanOutcome::Completed,
                kind: ScanKind::Rescan { changed_folders: 12 },
            };
            let restored = ScanKind::Restored { taken: Timestamp::from_secs(100) };

            assert_eq!(
                serde_json::to_value(rescan).unwrap()["kind"],
                json!({ "kind": "rescan", "changedFolders": 12 })
            );
            assert_eq!(serde_json::to_value(restored).unwrap(), json!({ "kind": "restored", "taken": 100 }));
        }

        #[test]
        fn cleanup_events_name_their_cleanup() {
            let started =
                Event::CleanupStarted { cleanup: crate::CleanupId::new(4), method: crate::Method::Trash, total: 128 };
            let removed = Event::FindingsRemoved { scan: ScanId::new(2), findings: vec![crate::FindingId::new(7)] };

            assert_eq!(
                serde_json::to_value(started).unwrap(),
                json!({ "type": "cleanup-started", "cleanup": 4, "method": "trash", "total": 128 })
            );
            assert_eq!(
                serde_json::to_value(removed).unwrap(),
                json!({ "type": "findings-removed", "scan": 2, "findings": [7] })
            );
            assert_eq!(serde_json::to_value(Event::QuitRequested).unwrap(), json!({ "type": "quit-requested" }));
        }

        #[test]
        fn exports_the_event_union_to_typescript() {
            let declaration = Event::decl(&Config::new());

            assert!(declaration.starts_with("type Event = "), "{declaration}");
            for tag in [
                "scan-started",
                "scan-progress",
                "findings-added",
                "module-finished",
                "scan-finished",
                "setting-changed",
                "access-changed",
                "cleanup-started",
                "cleanup-items",
                "cleanup-finished",
                "findings-removed",
                "quit-requested",
            ] {
                let variant = format!("{{ \"type\": \"{tag}\"");
                assert!(declaration.contains(&variant), "{tag}: {declaration}");
            }
            assert!(declaration.contains("entries: number"), "{declaration}");
        }
    }
}
