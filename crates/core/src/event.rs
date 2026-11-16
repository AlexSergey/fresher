//! The events the engine sends up to every subscriber (the UI, later the history).

use serde::Serialize;
use ts_rs::TS;

use crate::{AccessStatus, ByteSize, Finding, ModuleId, ScanId, SettingKey, SettingValue, Text, TextKey, Timestamp};

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
}

impl ModuleOutcome {
    /// The module panicked: the engine caught it, and the module counts as failed with the core's own reason.
    pub fn panicked() -> Self {
        Self::Failed { reason: Text::new(TextKey::from_valid("core.module.panicked".into())) }
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
            ] {
                let variant = format!("{{ \"type\": \"{tag}\"");
                assert!(declaration.contains(&variant), "{tag}: {declaration}");
            }
            assert!(declaration.contains("entries: number"), "{declaration}");
        }
    }
}
