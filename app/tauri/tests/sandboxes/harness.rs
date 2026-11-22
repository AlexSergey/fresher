//! One case file run through the app's own sandbox composition: generated into a temporary folder, scanned, every
//! negative and positive checked, every positive cleaned, what is left checked, and the report made. Every problem is
//! returned as a [`Failure`] naming its case; nothing here panics on a broken expectation.

use std::fmt;
use std::fs;
use std::io::Write as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime};

use fresher_app::{SandboxMode, SandboxPorts, SandboxTrash, access_of, sandbox_engine};
use fresher_core::{
    Access, ByteSize, Capability, CleanupDetail, CleanupPlanView, CleanupSelection, Confirmation, Event, Finding,
    FindingOrder, FindingsQuery, LeftOutCount, RecordedOutcome, SANDBOX_MARKER, Subject, TrashState, VolumeFacts,
    VolumeId, Volumes,
};
use fresher_engine::{Engine, ScanMode};
use fresher_macos::{current_user, protection, user_temp_dir};
use fresher_sandbox::{AfterCleanup, Case, CaseFile, Expectation, Sandbox, StatedSpace, WorldCommands, generate};
use fresher_testing::{FIXTURE_NOW, FakeAccess, FakeClock, FakeOpenFiles, FakeVolumes, TreeEntry, TreeReport};
use serde_json::{Value, json};
use tempfile::TempDir;

use crate::insight::{Described, check_insights};
use crate::left_out::{check_left_outs, sorted};
use crate::map::{check_map, sandbox_disk};

/// The longest a scan or a cleanup is waited for before the run fails instead of hanging.
const WAIT: Duration = Duration::from_secs(180);

/// The name of the sandbox inside the run's temporary folder.
const SANDBOX: &str = "sandbox";

/// What went wrong, and in which case (or, before any case is reached, in which case file).
#[derive(Debug)]
pub(crate) struct Failure {
    pub(crate) case: String,
    pub(crate) problem: String,
}

impl fmt::Display for Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "case `{}`: {}", self.case, self.problem)
    }
}

pub(crate) fn failure(case: &str, problem: impl fmt::Display) -> Failure {
    Failure { case: case.to_owned(), problem: problem.to_string() }
}

/// The repository's `fixtures/` folder.
pub(crate) fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// The names of the case files in `fixtures/sandboxes/`, without `.toml`, sorted; none when the folder is missing.
pub(crate) fn case_file_names() -> Vec<String> {
    let Ok(entries) = fs::read_dir(fixtures().join("sandboxes")) else { return Vec::new() };
    let mut names: Vec<String> = entries
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .map(|path| path.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// Reads `fixtures/sandboxes/<name>.toml`.
pub(crate) fn read_case_file(name: &str) -> String {
    let path = fixtures().join("sandboxes").join(format!("{name}.toml"));
    fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display())).unwrap()
}

/// The autotest of `fixtures/sandboxes/<name>.toml`: the run passes and its report equals
/// `fixtures/golden/sandbox-<name>.json` (`UPDATE_GOLDEN=1` rewrites the file instead).
pub(crate) fn check_case_file(name: &str) {
    assert_golden_volume();
    let report = run(name, &read_case_file(name)).map_err(|failure| format!("{name}.toml: {failure}")).unwrap();
    let golden = fixtures().join("golden").join(format!("sandbox-{name}.json"));
    let actual = serde_json::to_string_pretty(&report).unwrap() + "\n";
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::write(&golden, actual).unwrap();
        return;
    }
    let expected = fs::read_to_string(&golden).map_err(|error| format!("{}: {error}", golden.display())).unwrap();
    assert_eq!(actual, expected, "{name}.toml: the report differs from {}", golden.display());
}

/// The goldens hold allocated sizes as an APFS volume with 4 KiB blocks gives them (CI runs on macOS): a byte of data
/// takes one 4 KiB block and a sparse file takes none. Fails with a clear message on a temporary folder that differs.
fn assert_golden_volume() {
    let folder = tempfile::tempdir().unwrap();
    let (data, sparse) = (folder.path().join("data"), folder.path().join("sparse"));
    let mut one = fs::File::create(&data).unwrap();
    one.write_all(b"x").unwrap();
    one.sync_all().unwrap();
    fs::File::create(&sparse).unwrap().set_len(1024 * 1024).unwrap();
    let allocated = |path: &Path| fs::metadata(path).unwrap().blocks() * 512;
    let (one_byte, sparse) = (allocated(&data), allocated(&sparse));
    assert!(
        one_byte == 4096 && sparse == 0,
        "the sandbox goldens assume the temporary folder is on an APFS volume with 4 KiB blocks; {} gives one byte \
         {one_byte} bytes on disk and a sparse 1 MiB file {sparse}",
        folder.path().display()
    );
}

/// The ids of the modules the app registers, in its order.
pub(crate) fn registered_modules() -> Vec<String> {
    let world = World::new("registry").unwrap();
    let (engine, _) = world.engine(&[], None).unwrap();
    engine.modules().iter().map(|module| module.id.as_str().to_owned()).collect()
}

/// Runs the case file `text`, named `name` in failures: the report, or the first broken expectation. The Largest screen
/// (its picks joining the findings) and the storage map are checked after the scan and before the cleanup, the Trash
/// view both before and after it; a program the case file
/// does not state fails the run after the scan and after the cleanup; the insights its module described must be the
/// ones its cases expect.
pub(crate) fn run(name: &str, text: &str) -> Result<Value, Failure> {
    let file = CaseFile::parse(text).map_err(|error| failure(name, format!("the case file is refused: {error}")))?;
    let world = World::new(name)?;
    let (engine, commands) =
        world.engine(std::slice::from_ref(&file), file.module().map(fresher_core::ModuleId::as_str))?;
    // The core's own category is a positive's to expect too: a row the run picks on the Largest screen.
    let categories: Vec<_> = engine
        .modules()
        .into_iter()
        .flat_map(|module| module.categories)
        .map(|info| info.descriptor)
        .chain([fresher_core::own_file()])
        .collect();
    file.check(&categories, &engine.problems())
        .map_err(|error| failure(name, format!("the case file does not fit the modules: {error}")))?;
    let sandbox = world.sandbox();
    let around = world.around(name)?;
    let before = report_of(name, &sandbox)?;

    let (left_out, insights) = scan(name, &engine)?;
    let left_out = sorted(&left_out);
    check_no_unstated_calls(name, &commands)?;
    check_left_outs(name, &file, &left_out)?;
    check_insights(name, &file, &insights)?;
    let mut findings = findings(name, &engine)?;
    let (largest, picks) = crate::largest::check_largest(name, &file, &engine, &sandbox)?;
    findings.extend(picks);
    sort_findings(&mut findings);
    let mut positives = Vec::new();
    for case in file.cases() {
        match case.expect() {
            None => check_not_offered(case, &file, &sandbox, &findings)?,
            Some(expect) => positives.push(check_offered(case, expect, &sandbox, &findings)?),
        }
    }
    let map = check_map(name, &file, &engine, &sandbox, &findings)?;
    let trash_before = engine.trash();
    crate::trash::check_before(name, &trash_before)?;
    let (view, detail) = clean(name, &engine, &positives)?;
    check_no_unstated_calls(name, &commands)?;
    let (view, detail) = (view.as_ref(), detail.as_ref());
    let trash_after = engine.trash();
    crate::trash::check_after(name, &trash_after, detail)?;

    let after = report_of(name, &sandbox)?;
    for case in file.cases() {
        match case.expect() {
            None => check_untouched(case, &before, &after)?,
            Some(expect) => check_after(case, expect, &sandbox, &findings, view, detail)?,
        }
    }
    if world.around(name)? != around {
        return Err(failure(name, "the folder that holds the sandbox changed"));
    }
    let mut report = json!({
        "findings": findings.iter().map(|finding| finding_report(finding, &sandbox)).collect::<Vec<_>>(),
        "map": map,
        "cleanup": cleanup_report(&sandbox, &findings, view, detail),
        "trash": { "before": crate::trash::report(&trash_before), "after": crate::trash::report(&trash_after) },
        "treeAfter": tree_report(&after),
    });
    if let Some(left_out) = crate::left_out::report(&left_out) {
        report["leftOut"] = left_out;
    }
    if let Some(insights) = crate::insight::report(&file, &insights, &sandbox) {
        report["insights"] = insights;
    }
    if let Some(largest) = largest {
        report["largest"] = largest;
    }
    Ok(report)
}

/// A temporary folder holding the sandbox and a stand-in for the real Mac: its home, and the development build's data
/// folder (never made).
pub(crate) struct World {
    folder: TempDir,
    name: String,
}

impl World {
    pub(crate) fn new(name: &str) -> Result<Self, Failure> {
        let folder = tempfile::tempdir().map_err(|error| failure(name, error))?;
        let real_home = folder.path().join("real-home");
        fs::create_dir(&real_home).map_err(|error| failure(name, error))?;
        Ok(Self { folder, name: name.to_owned() })
    }

    fn real_home(&self) -> PathBuf {
        self.folder.path().join("real-home")
    }

    fn real_data(&self) -> PathBuf {
        self.folder.path().join("real-data")
    }

    /// The sandbox folder at its canonical path, as the generator writes it.
    pub(crate) fn sandbox(&self) -> PathBuf {
        fs::canonicalize(self.folder.path()).unwrap_or_else(|_| self.folder.path().to_path_buf()).join(SANDBOX)
    }

    /// `files` generated into the sandbox, and the sandbox mode's engine on it with fakes for every port it keeps
    /// real, the sandbox on a fake volume sized by its files, and the world's commands it runs; only `module`
    /// registered when one is named.
    pub(crate) fn engine(
        &self,
        files: &[CaseFile],
        module: Option<&str>,
    ) -> Result<(Engine, Arc<WorldCommands>), Failure> {
        let name = &self.name;
        let fixture_now = SystemTime::UNIX_EPOCH + Duration::from_secs(FIXTURE_NOW);
        // A demo world is generated at its own moment.
        let now = files.iter().find_map(CaseFile::demo).map_or(fixture_now, fresher_sandbox::Demo::now);
        let user_temp = user_temp_dir().ok();
        let real = protection(&self.real_home(), &self.real_data(), None, user_temp.as_deref())
            .map_err(|error| failure(name, error))?;
        let generated = generate(files, &self.folder.path().join(SANDBOX), now, &real, current_user())
            .map_err(|error| failure(name, format!("the sandbox cannot be generated: {error}")))?;
        let mode = SandboxMode::new(generated.sandbox.path(), SandboxTrash::Sandbox);
        let mode = match module {
            None => mode,
            Some(id) => mode.only(fresher_core::ModuleId::new(leak(id)).map_err(|error| failure(name, error))?),
        };
        let tree = TreeReport::of(generated.sandbox.path()).map_err(|error| failure(name, error))?;
        let tree_size = sandbox_entries(&tree).iter().map(|entry| entry.logical_size).sum();
        let world = fresher_sandbox::World::read(&generated.sandbox).map_err(|error| failure(name, error))?;
        let commands = Arc::new(world.commands(generated.sandbox.path()));
        let ports = fakes(tree_size, &world, &generated.sandbox, Arc::clone(&commands))
            .ok_or_else(|| failure(name, "the sandbox's world.toml holds no `now`"))?;
        let engine = sandbox_engine(&mode, &self.real_home(), &self.real_data(), ports)
            .map_err(|error| failure(name, format!("sandbox mode does not start: {error}")))?;
        if let Some(id) = module
            && !engine.modules().iter().any(|info| info.id.as_str() == id)
        {
            return Err(failure(name, format!("the app registers no module `{id}`")));
        }
        // The case file's `[settings]` were stored before the app started: a key the registered modules do not know or
        // a value of the wrong kind comes back as a problem of the stored settings.
        let view =
            engine.settings().map_err(|error| failure(name, format!("the settings cannot be read: {error:?}")))?;
        if let Some(problem) = view.problems.first() {
            let (key, problem) = (problem.key(), format!("{problem:?}"));
            return Err(failure(name, format!("the setting `{key}` of the case file is refused: {problem}")));
        }
        // The switch decides whether a guarded folder or other apps' data is read: the run turns on the switch of
        // every stated one, as the user would before scanning it.
        for capability in world.stated_capabilities() {
            let key = format!("core.access.{}", capability.id());
            let update = json!({ "key": key, "value": { "kind": "switch", "value": true } });
            let update = serde_json::from_value(update).map_err(|error| failure(name, error))?;
            engine.update_setting(update).map_err(|error| failure(name, format!("{key}: {error:?}")))?;
        }
        Ok((engine, commands))
    }

    /// Everything in the temporary folder but the sandbox.
    fn around(&self, name: &str) -> Result<Vec<TreeEntry>, Failure> {
        let report = TreeReport::of(self.folder.path()).map_err(|error| failure(name, error))?;
        Ok(report.entries.into_iter().filter(|entry| !Path::new(&entry.path).starts_with(SANDBOX)).collect())
    }
}

/// A module id lives as long as the app: the test leaks the few it reads.
fn leak(id: &str) -> &'static str {
    Box::leak(id.to_owned().into_boxed_str())
}

/// Fakes for every port sandbox mode keeps real: no clock, volume, open file, tool or permission of the machine, and
/// the running apps, the moment, the access, the installs and the stated facts of the sandbox's `world` (at `sandbox`),
/// as sandbox mode takes them: the clock stands at the moment the sandbox was generated. The volume holds `tree_size`
/// bytes of the sandbox's files; the programs are the world's `commands`. `None` when the world holds no moment.
fn fakes(
    tree_size: u64,
    world: &fresher_sandbox::World,
    sandbox: &Sandbox,
    commands: Arc<WorldCommands>,
) -> Option<SandboxPorts> {
    let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) };
    let now = world.now()?.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    let volumes = FakeVolumes::default().with("/", facts).with_space("/", sandbox_disk(tree_size));
    // A demo world's volume is the one the app answers, as in sandbox mode.
    let volumes: Arc<dyn Volumes> = match world.volume() {
        Some(volume) => Arc::new(StatedSpace::new(Arc::new(volumes), volume)),
        None => Arc::new(volumes),
    };
    Some(SandboxPorts {
        clock: Arc::new(FakeClock::at(now.as_secs())),
        volumes,
        open_files: Arc::new(FakeOpenFiles::default()),
        running: Arc::new(world.running_apps()),
        commands,
        access: access_of(
            world,
            &sandbox.home(),
            Arc::new(FakeAccess::with(&[(Capability::AllFiles, Access::Unknown)])),
        ),
        installs: Arc::new(world.install_records()),
        stated: world.stated(sandbox.path()),
    })
}

/// The reads of the trial module's made-up tool (`<tool> version`, `<tool> list`), a bare name that exists on no Mac:
/// sandbox mode answers them not installed, as the trial's case files expect, so they are the one unstated call a
/// case file may make. Its removal is never among them.
const TRIAL_TOOL_READS: [&str; 2] = ["version", "list"];

/// Fails the case file `file` when a program it does not state was run: every call `commands` could not answer, but
/// the trial tool's reads and the read of Finder's Trash setting (sandbox mode lets that one through to the Mac).
pub(crate) fn check_no_unstated_calls(file: &str, commands: &WorldCommands) -> Result<(), Failure> {
    let trial_read =
        |call: &str| TRIAL_TOOL_READS.iter().any(|read| call == format!("{} {read}", fresher_module_trial::TRIAL_TOOL));
    let finder_read = |call: &str| call == "/usr/bin/defaults read com.apple.finder FXRemoveOldTrashItems";
    let unanswered: Vec<String> =
        commands.unanswered().into_iter().filter(|call| !trial_read(call) && !finder_read(call)).collect();
    if unanswered.is_empty() {
        return Ok(());
    }
    Err(failure(file, format!("a program the case file does not state was run: {}", unanswered.join("; "))))
}

fn report_of(name: &str, sandbox: &Path) -> Result<TreeReport, Failure> {
    TreeReport::of(sandbox).map_err(|error| failure(name, error))
}

/// A full scan, waited for: what its modules left out and what they described, as each reported it.
pub(crate) fn scan(name: &str, engine: &Engine) -> Result<(Vec<LeftOutCount>, Described), Failure> {
    let events = engine.subscribe();
    engine.start_scan(ScanMode::Full).map_err(|error| failure(name, format!("the scan does not start: {error:?}")))?;
    let (mut left_out, mut described) = (Vec::new(), Vec::new());
    until(name, &events, |event| {
        if let Event::ModuleFinished { module, left_out: counts, insights, .. } = event {
            left_out.extend(counts.iter().cloned());
            described.extend(insights.iter().map(|insight| (*module, insight.clone())));
        }
        matches!(event, Event::ScanFinished { .. })
    })?;
    Ok((left_out, described))
}

/// Waits for the first event that `is_last`; on a timeout, the failure names the last event seen.
fn until(name: &str, events: &Receiver<Event>, mut is_last: impl FnMut(&Event) -> bool) -> Result<(), Failure> {
    let mut last: Option<Event> = None;
    loop {
        let event = events.recv_timeout(WAIT).map_err(|error| {
            failure(name, format!("no end in sight after {} s ({error}); the last event: {last:?}", WAIT.as_secs()))
        })?;
        if is_last(&event) {
            return Ok(());
        }
        last = Some(event);
    }
}

/// Every finding of the scan, of every registered module, by subject and category, so the findings' ids never order
/// the report.
pub(crate) fn findings(name: &str, engine: &Engine) -> Result<Vec<Finding>, Failure> {
    let query = FindingsQuery::new(None, None, FindingOrder::Found, 0, 1000);
    let mut page = engine.findings(&query).map_err(|error| failure(name, format!("{error:?}")))?;
    if page.total as usize > page.findings.len() {
        return Err(failure(name, format!("{} findings: more than one page", page.total)));
    }
    sort_findings(&mut page.findings);
    Ok(page.findings)
}

/// By subject and category, so the findings' ids never order the report.
fn sort_findings(findings: &mut [Finding]) {
    findings.sort_by_key(|finding| (format!("{:?}", finding.finding().subject()), finding.finding().category()));
}

pub(crate) fn path_of(finding: &Finding) -> Option<&Path> {
    match finding.finding().subject() {
        Subject::Path(path) => Some(path),
        Subject::External { .. } => None,
    }
}

/// A negative case: no finding at or below any of its paths, nor at a folder above them that only this case writes
/// into (a look-alike file's folder is what a rule would offer).
fn check_not_offered(case: &Case, file: &CaseFile, sandbox: &Path, findings: &[Finding]) -> Result<(), Failure> {
    let others: Vec<&Path> = file
        .cases()
        .iter()
        .filter(|other| other.name() != case.name())
        .flat_map(|other| other.tree().paths().chain(other.tree().markers()))
        .collect();
    let only_its_own = |folder: &Path| !others.iter().any(|other| other.starts_with(folder));
    for path in case.tree().paths() {
        let below = sandbox.join(path);
        let folders: Vec<PathBuf> = path
            .ancestors()
            .skip(1)
            .filter(|folder| !folder.as_os_str().is_empty() && only_its_own(folder))
            .map(|folder| sandbox.join(folder))
            .collect();
        let offered = |found: &&Path| found.starts_with(&below) || folders.iter().any(|folder| found == folder);
        if let Some(found) = findings.iter().filter_map(path_of).find(offered) {
            return Err(failure(case.name(), format!("offered: {}", relative(found, sandbox))));
        }
    }
    Ok(())
}

/// A positive case: its finding, at its item, in its category, with its danger.
fn check_offered<'a>(
    case: &Case,
    expect: &Expectation,
    sandbox: &Path,
    findings: &'a [Finding],
) -> Result<&'a Finding, Failure> {
    let item = sandbox.join(expect.item());
    let found = findings
        .iter()
        .find(|finding| path_of(finding) == Some(item.as_path()))
        .ok_or_else(|| failure(case.name(), format!("not offered: {}", expect.item().display())))?;
    let (category, danger) = (found.finding().category(), found.finding().danger());
    if category != expect.category() {
        let problem = format!("offered in `{}`, not in `{}`", category.as_str(), expect.category().as_str());
        return Err(failure(case.name(), problem));
    }
    if danger != expect.danger() {
        return Err(failure(case.name(), format!("offered as {danger:?}, not as {:?}", expect.danger())));
    }
    Ok(found)
}

/// Prepares the cleanup of `positives`, confirms it to the Trash with the summary it got and waits for its end; no
/// cleanup runs when the plan holds no item (everything left out).
fn clean(
    name: &str,
    engine: &Engine,
    positives: &[&Finding],
) -> Result<(Option<CleanupPlanView>, Option<CleanupDetail>), Failure> {
    if positives.is_empty() {
        return Ok((None, None));
    }
    let ids: Vec<_> = positives.iter().map(|finding| finding.id()).collect();
    let selection: CleanupSelection =
        serde_json::from_value(json!({ "findings": ids, "categories": [] })).map_err(|error| failure(name, error))?;
    let view = engine
        .prepare_cleanup(&selection)
        .map_err(|error| failure(name, format!("the cleanup cannot be prepared: {error:?}")))?;
    if view.summary.count == 0 {
        return Ok((Some(view), None));
    }
    let summary = view.summary;
    let echo: Confirmation = serde_json::from_value(json!({ "plan": view.plan, "count": summary.count,
        "allocated": summary.sizes.allocated, "highest": summary.highest, "method": "trash" }))
    .map_err(|error| failure(name, error))?;
    let events = engine.subscribe();
    let id = engine
        .confirm_cleanup(&echo)
        .map_err(|error| failure(name, format!("the cleanup is not confirmed: {error:?}")))?;
    until(name, &events, |event| matches!(event, Event::CleanupFinished { .. }))?;
    let detail = engine.cleanup(id).map_err(|error| failure(name, format!("{error:?}")))?;
    Ok((Some(view), Some(detail)))
}

/// A negative case: every entry at or below its paths as it was before the cleanup.
fn check_untouched(case: &Case, before: &TreeReport, after: &TreeReport) -> Result<(), Failure> {
    let markers = case.tree().markers().map(|folder| folder.join(SANDBOX_MARKER));
    let paths: Vec<PathBuf> = case.tree().paths().map(Path::to_path_buf).chain(markers).collect();
    let own = |report: &TreeReport| -> Vec<TreeEntry> {
        let below = |entry: &&TreeEntry| paths.iter().any(|path| Path::new(&entry.path).starts_with(path));
        report.entries.iter().filter(below).cloned().collect()
    };
    let (was, is) = (own(before), own(after));
    if was != is {
        let missing = |from: &[TreeEntry], to: &[TreeEntry]| -> Vec<String> {
            from.iter().filter(|entry| !to.contains(entry)).map(|entry| entry.path.clone()).collect()
        };
        let (gone, added) = (missing(&was, &is), missing(&is, &was));
        return Err(failure(case.name(), format!("changed by the cleanup: gone {gone:?}, added {added:?}")));
    }
    Ok(())
}

/// A positive case: as its `after-cleanup` says.
fn check_after(
    case: &Case,
    expect: &Expectation,
    sandbox: &Path,
    findings: &[Finding],
    view: Option<&CleanupPlanView>,
    detail: Option<&CleanupDetail>,
) -> Result<(), Failure> {
    let item = sandbox.join(expect.item());
    let left_out = view.into_iter().flat_map(|view| &view.left_out).find(|left| {
        findings.iter().any(|finding| finding.id() == left.finding() && path_of(finding) == Some(item.as_path()))
    });
    let recorded = detail
        .into_iter()
        .flat_map(|detail| &detail.items)
        .find(|recorded| recorded.path.as_deref() == Some(item.as_path()));
    let outcome = match (left_out, recorded) {
        (Some(left), _) => RecordedOutcome::Skipped { reason: left.reason().code() },
        (None, Some(recorded)) => recorded.outcome,
        (None, None) => return Err(failure(case.name(), "neither cleaned nor left out")),
    };
    let present = fs::symlink_metadata(&item).ok();
    let as_expected = match expect.after() {
        AfterCleanup::InTrash => {
            outcome == RecordedOutcome::Moved
                && present.is_none()
                && recorded.is_some_and(|recorded| recorded.trash == Some(TrashState::InTrash))
                && in_sandbox_trash(sandbox, &item)
        }
        AfterCleanup::Emptied => {
            matches!(outcome, RecordedOutcome::Emptied { .. }) && present.as_ref().is_some_and(fs::Metadata::is_dir)
        }
        AfterCleanup::Skipped(code) => outcome == RecordedOutcome::Skipped { reason: code } && present.is_some(),
    };
    if !as_expected {
        let still = if present.is_some() { "still there" } else { "gone" };
        let problem = format!("expected {:?}, the cleanup ended {outcome:?} and the item is {still}", expect.after());
        return Err(failure(case.name(), problem));
    }
    Ok(())
}

/// Whether the sandbox's Trash, `home/.Trash`, holds an entry moved there from `item` (named `<n>-<its name>`).
fn in_sandbox_trash(sandbox: &Path, item: &Path) -> bool {
    let (Some(name), Ok(entries)) = (item.file_name(), fs::read_dir(sandbox.join("home/.Trash"))) else {
        return false;
    };
    let suffix = format!("-{}", name.to_string_lossy());
    entries.filter_map(Result::ok).any(|entry| entry.file_name().to_string_lossy().ends_with(&suffix))
}

/// `path` relative to the sandbox, with `/` separators; a path outside it as it is.
pub(crate) fn relative(path: &Path, sandbox: &Path) -> String {
    path.strip_prefix(sandbox).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn finding_report(finding: &Finding, sandbox: &Path) -> Value {
    let inner = finding.finding();
    let subject = match inner.subject() {
        Subject::Path(path) => relative(path, sandbox),
        Subject::External { tool, id } => format!("{}:{id}", tool.as_str()),
    };
    json!({
        "subject": subject,
        "module": finding.module(),
        "category": inner.category(),
        "danger": inner.danger(),
        "logical": inner.sizes().logical,
        "app": inner.app(),
    })
}

fn cleanup_report(
    sandbox: &Path,
    findings: &[Finding],
    view: Option<&CleanupPlanView>,
    detail: Option<&CleanupDetail>,
) -> Value {
    let subject = |id| findings.iter().find(|finding| finding.id() == id).and_then(path_of);
    let mut left_out: Vec<Value> = view
        .into_iter()
        .flat_map(|view| &view.left_out)
        .map(|left| {
            json!({
                "path": subject(left.finding()).map(|path| relative(path, sandbox)),
                "reason": left.reason().code(),
            })
        })
        .collect();
    let mut items: Vec<Value> = detail
        .into_iter()
        .flat_map(|detail| &detail.items)
        .map(|item| {
            json!({
                "path": item.path.as_deref().map(|path| relative(path, sandbox)),
                "category": item.category,
                "danger": item.danger,
                "action": item.action,
                "outcome": item.outcome,
                "trash": item.trash,
            })
        })
        .collect();
    // The engine orders items of one size by their findings' ids, which differ from run to run: the report does not.
    let by_path = |entry: &Value| (entry["path"].to_string(), entry["category"].to_string());
    left_out.sort_by_key(by_path);
    items.sort_by_key(by_path);
    json!({ "leftOut": left_out, "items": items })
}

/// The sandbox after the cleanup, without what depends on where it was made or on the order the cleanup took: an
/// entry of the sandbox's Trash is named `<n>-<its name>` in the order of the moves, so its `<n>-` is left out.
fn tree_report(after: &TreeReport) -> Value {
    let mut entries: Vec<TreeEntry> = sandbox_entries(after)
        .into_iter()
        .map(|entry| TreeEntry { path: without_trash_order(&entry.path), ..entry })
        .collect();
    entries.sort_by(|left, right| {
        (&left.path, left.logical_size, format!("{:?}", left.kind)).cmp(&(
            &right.path,
            right.logical_size,
            format!("{:?}", right.kind),
        ))
    });
    json!({
        "entries": entries,
        "totalLogicalSize": entries.iter().map(|entry| entry.logical_size).sum::<u64>(),
    })
}

/// `path` with the `<n>-` of a Trash entry's name left out: `home/.Trash/0-cache/a` is `home/.Trash/cache/a`.
fn without_trash_order(path: &str) -> String {
    let Some(rest) = path.strip_prefix("home/.Trash/") else { return path.to_owned() };
    match rest.split_once('-') {
        Some((order, name)) if !order.is_empty() && order.bytes().all(|byte| byte.is_ascii_digit()) => {
            format!("home/.Trash/{name}")
        }
        _ => path.to_owned(),
    }
}

/// The folders of the sandbox's temporary places, the same in every sandbox: a report shows what is inside them only.
const TEMPORARY_LAYOUT: [&str; 10] = [
    "private",
    "private/tmp",
    "private/var",
    "private/var/folders",
    "private/var/folders/xx",
    "private/var/folders/xx/sandbox",
    "private/var/folders/xx/sandbox/0",
    "private/var/folders/xx/sandbox/C",
    "private/var/folders/xx/sandbox/T",
    "private/var/folders/xx/sandbox/X",
];

/// The sandbox's entries but what depends on where it was made or is in every sandbox: the app's data folder and the
/// folders of the temporary places are left out, and the checklist (which names the sandbox's path) is kept without
/// its size.
fn sandbox_entries(report: &TreeReport) -> Vec<TreeEntry> {
    report
        .entries
        .iter()
        .filter(|entry| !Path::new(&entry.path).starts_with(".fresher-data"))
        .filter(|entry| !TEMPORARY_LAYOUT.contains(&entry.path.as_str()))
        .map(|entry| {
            let sized = Path::new(&entry.path) != Path::new("EXPECTED.md");
            TreeEntry { logical_size: if sized { entry.logical_size } else { 0 }, ..entry.clone() }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use fresher_sandbox::{World, WorldCommands};

    /// The commands of a world stating nothing, at `/box`.
    fn commands() -> WorldCommands {
        World::default().commands(Path::new("/box"))
    }

    mod negative {
        use fresher_core::{CancelToken, CommandRunner as _};

        use super::commands;
        use crate::harness::check_no_unstated_calls;

        #[test]
        fn the_trial_tools_removal_is_no_read() {
            let commands = commands();
            let _ = commands.run("fresher-trial-tool", &["list"], &CancelToken::new());
            let _ = commands.run("fresher-trial-tool", &["remove", "trial-object"], &CancelToken::new());

            let failure = check_no_unstated_calls("trial", &commands).unwrap_err();

            assert!(failure.problem.ends_with("was run: fresher-trial-tool remove trial-object"), "{failure}");
        }

        #[test]
        fn a_defaults_write_is_no_read() {
            let commands = commands();
            let args = ["write", "com.apple.finder", "FXRemoveOldTrashItems", "1"];
            let _ = commands.run("/usr/bin/defaults", &args, &CancelToken::new());

            assert!(check_no_unstated_calls("docker", &commands).is_err());
        }

        #[test]
        fn an_unstated_call_fails_the_file_naming_it() {
            let commands = commands();
            let _ = commands.run("/bin/rm", &["-rf", "/box/home"], &CancelToken::new());
            let _ = commands.run("/bin/echo", &[], &CancelToken::new());

            let failure = check_no_unstated_calls("docker", &commands).unwrap_err();

            assert_eq!(failure.case, "docker");
            assert_eq!(
                failure.problem,
                "a program the case file does not state was run: /bin/rm -rf /box/home; /bin/echo"
            );
        }
    }

    mod positive {
        use fresher_core::{CancelToken, CommandRunner as _};

        use super::commands;
        use crate::harness::check_no_unstated_calls;

        #[test]
        fn no_call_passes() {
            assert!(check_no_unstated_calls("docker", &commands()).is_ok());
        }

        #[test]
        fn the_trial_tools_reads_pass() {
            let commands = commands();
            let _ = commands.run("fresher-trial-tool", &["version"], &CancelToken::new());
            let _ = commands.run("fresher-trial-tool", &["list"], &CancelToken::new());

            assert!(check_no_unstated_calls("trial", &commands).is_ok());
        }

        #[test]
        fn the_read_of_finders_trash_setting_passes() {
            let commands = commands();
            let args = ["read", "com.apple.finder", "FXRemoveOldTrashItems"];
            let _ = commands.run("/usr/bin/defaults", &args, &CancelToken::new());

            assert!(check_no_unstated_calls("docker", &commands).is_ok());
        }
    }
}
