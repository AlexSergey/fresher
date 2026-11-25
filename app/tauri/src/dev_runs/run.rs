//! One run's record: begun at the start, every action and engine event folded into it and appended to its events,
//! the snapshot rewritten at the start, after every scan and after every cleanup. A write that fails is logged and the
//! run goes on.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use fresher_core::{
    Build, Capability, CleanupId, Clock, Event, Finding, LeftOutCount, ModuleId, ModuleInfo, SCAN_ROOTS, ScanId,
    SettingKey, SettingValue,
};
use fresher_engine::Engine;
use serde::Serialize;

use super::english::english;
use super::files::{self, RunFiles};
use super::record::{
    ACTED, ActionRecord, ClearItem, ClearRecord, FORMAT, FindingRecord, LeftOutRecord, ModuleRecord, Record,
    ScanRecord, Session, clear_outcome,
};
use super::{DevRunsRefusal, Launch, Mode, clock};

/// The group name of an item whose category no module claims.
const UNKNOWN: &str = "unknown";

/// A run being recorded.
pub(crate) struct Run {
    files: RunFiles,
    record: Record,
    clock: Arc<dyn Clock>,
    scan_started: Option<SystemTime>,
    /// The number of the last snapshot made; the first, written at the start, is 0.
    sequence: u64,
}

/// A snapshot made under the run's lock, to be written after it is released.
#[derive(Debug)]
pub(crate) struct Snapshot {
    sequence: u64,
    bytes: Vec<u8>,
}

/// Writes the snapshots of one run, never one older than the last it was given: two threads may race to write.
#[derive(Debug)]
pub(crate) struct SnapshotWriter {
    folder: PathBuf,
    path: PathBuf,
    last: u64,
}

impl SnapshotWriter {
    /// Replaces the snapshot with `snapshot` unless a later one was already given; a failure is logged.
    pub(crate) fn write(&mut self, snapshot: &Snapshot) {
        if snapshot.sequence <= self.last {
            return;
        }
        self.last = snapshot.sequence;
        if let Err(error) = files::replace(&self.folder, &self.path, &snapshot.bytes) {
            tracing::warn!(%error, "the development run record's snapshot cannot be written");
        }
    }
}

/// An event line of the events file.
#[derive(Serialize)]
struct EventLine<'a> {
    at: &'a str,
    event: &'a Event,
}

/// An action line of the events file.
#[derive(Serialize)]
struct ActionLine<'a> {
    at: &'a str,
    action: &'a str,
    args: &'a serde_json::Value,
}

impl fmt::Debug for Run {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Run").field("snapshot", &self.files.snapshot()).finish_non_exhaustive()
    }
}

impl Run {
    /// Creates the record's pair for the run of `build` on `engine` and writes the first snapshot; `None` for a release
    /// build, which records nothing.
    pub(crate) fn begin(
        launch: &Launch,
        build: &Build,
        home: &Path,
        macos: Option<String>,
        clock: Arc<dyn Clock>,
        engine: &Engine,
    ) -> Result<Option<Self>, DevRunsRefusal> {
        let Some(mode) = Mode::of(build) else {
            return Ok(None);
        };
        let started = clock.now();
        let files = RunFiles::create(&launch.folder, started, &launch.build, &mode)?;
        let values = values(engine).unwrap_or_default();
        let roots = match values.iter().find(|(key, _)| key.as_str() == SCAN_ROOTS) {
            Some((_, SettingValue::Folders(roots))) => roots.clone(),
            _ => Vec::new(),
        };
        let access = engine.access_status();
        let session = Session {
            started: clock::utc(started).iso(),
            build: launch.build.clone(),
            mode: mode.label(),
            macos,
            home: home.to_path_buf(),
            roots,
            full_disk_access: access.access(&Capability::AllFiles),
        };
        let record = Record {
            format: FORMAT,
            session,
            settings: plain(&values),
            access: access.capabilities,
            actions: Vec::new(),
            scans: Vec::new(),
            clear: Vec::new(),
        };
        let run = Self { files, record, clock, scan_started: None, sequence: 0 };
        run.files
            .rewrite(&run.record)
            .map_err(|source| DevRunsRefusal::Write { path: run.files.snapshot().to_path_buf(), source })?;
        Ok(Some(run))
    }

    /// Records a command the UI sent, before it runs.
    pub(crate) fn action(&mut self, command: &str, args: serde_json::Value) {
        let at = self.now();
        self.append(&ActionLine { at: &at, action: command, args: &args });
        self.record.actions.push(ActionRecord { at, command: command.to_owned(), args });
    }

    /// The writer of this run's snapshots, which [`Self::event`] makes.
    pub(crate) fn writer(&self) -> SnapshotWriter {
        let (folder, path) = (self.files.folder().to_path_buf(), self.files.snapshot().to_path_buf());
        SnapshotWriter { folder, path, last: self.sequence }
    }

    /// Records an engine event and folds it into the record; progress is never recorded. After a scan or a cleanup,
    /// answers the new snapshot, for [`SnapshotWriter::write`] once the run's lock is released.
    pub(crate) fn event(&mut self, event: &Event, engine: &Engine) -> Option<Snapshot> {
        if matches!(event, Event::ScanProgress { .. }) {
            return None;
        }
        let at = self.now();
        self.append(&EventLine { at: &at, event });
        match event {
            Event::ScanStarted { scan, modules } => self.scan_started(*scan, modules, at, engine),
            Event::FindingsAdded { scan, findings } => self.findings_added(*scan, findings, engine),
            Event::ModuleFinished { scan, module, outcome, left_out, .. } => {
                let left_out = left_out_records(left_out, engine);
                let record = self.scan(*scan).map(|scan| scan.modules.entry(*module).or_default());
                if let Some(record) = record {
                    record.outcome = Some(outcome.clone());
                    record.left_out = left_out;
                }
            }
            Event::ScanCoverage { scan, coverage } => {
                if let Some(scan) = self.scan(*scan) {
                    scan.coverage = Some(coverage.clone());
                }
            }
            Event::FindingsRemoved { scan, findings } => {
                if let Some(scan) = self.scan(*scan) {
                    scan.removed.extend_from_slice(findings);
                }
            }
            Event::ScanFinished { scan, outcome, kind } => {
                let duration_ms = self.scan_started.take().map(|started| millis_since(started, self.clock.now()));
                if let Some(record) = self.scan(*scan) {
                    record.kind = Some(*kind);
                    record.outcome = Some(outcome.clone());
                    record.finished = Some(at);
                    record.duration_ms = duration_ms;
                }
                return self.snapshot(engine);
            }
            Event::CleanupStarted { cleanup, method, .. } => {
                let scan = self.record.scans.last().map(|scan| scan.id);
                self.record.clear.push(ClearRecord {
                    id: *cleanup,
                    scan,
                    method: *method,
                    dry_run: false,
                    ended: None,
                    groups: BTreeMap::new(),
                });
            }
            Event::CleanupFinished { cleanup, .. } => {
                self.cleanup_finished(*cleanup, engine);
                return self.snapshot(engine);
            }
            // Settings and access are read again from the engine at every snapshot.
            Event::ScanProgress { .. }
            | Event::SettingChanged { .. }
            | Event::AccessChanged { .. }
            | Event::CleanupItems { .. }
            | Event::AppsMeasuring { .. }
            | Event::QuitRequested => {}
        }
        None
    }

    fn scan_started(&mut self, id: ScanId, modules: &[ModuleId], at: String, engine: &Engine) {
        let infos = engine.modules();
        let modules = modules
            .iter()
            .map(|module| {
                let group = infos.iter().find(|info| info.id == *module).map(|info| info.group);
                (*module, ModuleRecord { group, ..ModuleRecord::default() })
            })
            .collect();
        self.scan_started = Some(self.clock.now());
        self.record.scans.push(ScanRecord {
            id,
            kind: None,
            started: at,
            finished: None,
            duration_ms: None,
            outcome: None,
            coverage: None,
            modules,
            removed: Vec::new(),
        });
    }

    fn findings_added(&mut self, id: ScanId, findings: &[Finding], engine: &Engine) {
        let dictionary = engine.dictionary();
        let Some(scan) = self.scan(id) else {
            return;
        };
        for finding in findings {
            let inner = finding.finding();
            scan.modules.entry(finding.module()).or_default().findings.push(FindingRecord {
                finding: finding.clone(),
                title_text: english(dictionary, inner.title()),
                reason_text: english(dictionary, inner.reason()),
            });
        }
    }

    fn cleanup_finished(&mut self, id: CleanupId, engine: &Engine) {
        let detail = match engine.cleanup(id) {
            Ok(detail) => detail,
            Err(error) => {
                tracing::warn!(?error, "the development run record cannot read the cleanup");
                return;
            }
        };
        let infos = engine.modules();
        let scan = self.record.scans.last().map(|scan| scan.id);
        // A cleanup whose start was missed is opened here.
        if !self.record.clear.iter().any(|clear| clear.id == id) {
            self.record.clear.push(ClearRecord {
                id,
                scan,
                method: detail.summary.method,
                dry_run: false,
                ended: None,
                groups: BTreeMap::new(),
            });
        }
        let Some(clear) = self.record.clear.iter_mut().find(|clear| clear.id == id) else {
            return;
        };
        let mut groups: BTreeMap<String, Vec<ClearItem>> = BTreeMap::new();
        for item in detail.items {
            let (module, group) = owner(&infos, &item.category);
            let (outcome, reason) = clear_outcome(&item.outcome, item.action, clear.method);
            groups.entry(group).or_default().push(ClearItem {
                module,
                category: item.category,
                danger: item.danger,
                path: item.path,
                sizes: item.sizes,
                action: item.action,
                outcome,
                reason,
            });
        }
        let outcomes = || groups.values().flatten().map(|item| item.outcome);
        clear.dry_run = outcomes().any(|outcome| outcome.starts_with("would-"))
            && !outcomes().any(|outcome| ACTED.contains(&outcome));
        clear.ended = Some(detail.summary.ended);
        clear.groups = groups;
    }

    /// The scan of `id`, looked for from the last: a closed scan still gets the findings a cleanup removed.
    fn scan(&mut self, id: ScanId) -> Option<&mut ScanRecord> {
        self.record.scans.iter_mut().rev().find(|scan| scan.id == id)
    }

    fn now(&self) -> String {
        clock::utc(self.clock.now()).iso()
    }

    fn append(&mut self, line: &impl Serialize) {
        if let Err(error) = self.files.append(line) {
            tracing::warn!(%error, "the development run record's events cannot be written");
        }
    }

    /// Reads the settings and the access again and makes the next snapshot; `None` (logged) when it cannot be made.
    fn snapshot(&mut self, engine: &Engine) -> Option<Snapshot> {
        if let Some(values) = values(engine) {
            self.record.settings = plain(&values);
        }
        self.record.access = engine.access_status().capabilities;
        match files::pretty(&self.record) {
            Ok(bytes) => {
                self.sequence = self.sequence.saturating_add(1);
                Some(Snapshot { sequence: self.sequence, bytes })
            }
            Err(error) => {
                tracing::warn!(%error, "the development run record's snapshot cannot be made");
                None
            }
        }
    }
}

/// Every setting's value; `None` (logged) when the engine cannot answer.
fn values(engine: &Engine) -> Option<BTreeMap<SettingKey, SettingValue>> {
    match engine.settings() {
        Ok(view) => Some(view.values),
        Err(error) => {
            tracing::warn!(?error, "the development run record cannot read the settings");
            None
        }
    }
}

/// Every value in its plain stored form, as the settings file holds it.
fn plain(values: &BTreeMap<SettingKey, SettingValue>) -> BTreeMap<SettingKey, serde_json::Value> {
    values.iter().map(|(key, value)| (key.clone(), value.to_json())).collect()
}

fn left_out_records(left_out: &[LeftOutCount], engine: &Engine) -> Vec<LeftOutRecord> {
    let dictionary = engine.dictionary();
    left_out
        .iter()
        .map(|count| LeftOutRecord { count: count.clone(), reason_text: english(dictionary, &count.reason) })
        .collect()
}

/// The module and the kebab-case group name of the module whose categories hold `category`.
fn owner(infos: &[ModuleInfo], category: &str) -> (String, String) {
    let info = infos.iter().find(|info| info.categories.iter().any(|known| known.descriptor.id().as_str() == category));
    let Some(info) = info else {
        return (UNKNOWN.to_owned(), UNKNOWN.to_owned());
    };
    let group = serde_json::to_value(info.group).ok().and_then(|value| value.as_str().map(str::to_owned));
    (info.id.as_str().to_owned(), group.unwrap_or_else(|| UNKNOWN.to_owned()))
}

fn millis_since(started: SystemTime, now: SystemTime) -> u64 {
    now.duration_since(started).map_or(0, |elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use crate::dev_runs::run::{Snapshot, SnapshotWriter};

        #[test]
        fn an_earlier_snapshot_never_replaces_a_later_one() {
            let folder = tempfile::tempdir().unwrap();
            let path = folder.path().join("run.json");
            let mut writer = SnapshotWriter { folder: folder.path().to_path_buf(), path: path.clone(), last: 0 };

            writer.write(&Snapshot { sequence: 2, bytes: b"later".to_vec() });
            writer.write(&Snapshot { sequence: 1, bytes: b"earlier".to_vec() });

            assert_eq!(fs::read(&path).unwrap(), b"later");
        }

        #[test]
        fn a_snapshot_of_the_same_sequence_never_replaces_the_written_one() {
            let folder = tempfile::tempdir().unwrap();
            let path = folder.path().join("run.json");
            let mut writer = SnapshotWriter { folder: folder.path().to_path_buf(), path: path.clone(), last: 0 };

            writer.write(&Snapshot { sequence: 2, bytes: b"first".to_vec() });
            writer.write(&Snapshot { sequence: 2, bytes: b"second".to_vec() });

            assert_eq!(fs::read(&path).unwrap(), b"first");
        }
    }
}
