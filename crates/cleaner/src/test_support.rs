//! Helpers of the crate's own tests: a fixture tree with everything the guard checks against, plan items, a locked
//! file, a read-only folder, and a disk image mounted inside a fixture tree.

use std::ffi::OsStr;
use std::fs::{self, Permissions};
use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use fresher_core::{
    Access, AccessStatus, Action, ByteSize, CancelToken, Capability, CapabilityState, CategoryDescriptor, CategoryId,
    DangerLevel, Finding, FindingId, GuardedPlace, ModuleId, NewFinding, OpenSet, PlanItem, Protection, Removability,
    Removal, RunningApps, RunningSet, Sizes, Subject, Text, TextKey,
};
use fresher_core::{
    Cleaner as _, CleanupContext, Clock, CommandRunner, Ended, ItemDone, Landed, OpenFiles, Trash, TrashError,
};
use fresher_testing::{FakeClock, FakeCommandRunner, FakeOpenFiles, FakeRunningApps, FakeTrash, FixtureTree};
use tempfile::TempDir;

use crate::{CleanerPorts, DiskCleaner};

use crate::guard::walk::{HeldWalk, SizeWalk, WalkStop};
use crate::guard::{GuardContext, Purpose};
use crate::paths::Roots;
use crate::reach::Reach;

/// The owner of `path`: in a test, whoever built the fixture tree.
pub(crate) fn user_of(path: &Path) -> u32 {
    fs::metadata(path).unwrap().uid()
}

/// A plan item of finding `id` with `action`, allocating `allocated` bytes.
pub(crate) fn plan_item(id: u32, action: Action, allocated: u64) -> PlanItem {
    let module = ModuleId::new("demo").unwrap();
    let category = CategoryDescriptor::new(CategoryId::new(module, "demo.caches").unwrap(), DangerLevel::Low);
    let text = |key: &str| Text::new(TextKey::new(key).unwrap());
    let (subject, removal) = match action {
        Action::Remove(path) => (Subject::Path(path.clone()), Removal::MoveToTrash(path)),
        Action::Empty(path) => (Subject::Path(path.clone()), Removal::EmptyFolder(path)),
        Action::RemoveTogether(parts) => (Subject::Path(parts.subject().to_path_buf()), Removal::MoveTogether(parts)),
        Action::Tool(command) => {
            (Subject::External { tool: command.tool(), id: format!("{id}") }, Removal::ToolCommand(command))
        }
    };
    let sizes = Sizes::complete(ByteSize::new(allocated), ByteSize::new(allocated));
    let finding = NewFinding::new(
        &category,
        format!("item-{id}"),
        subject,
        sizes,
        text("demo.finding.title"),
        text("demo.finding.reason"),
        removal,
    );
    PlanItem::new(&Finding::new(FindingId::new(id), finding, Removability::Removable))
}

/// A fixture tree that is a sandbox and the only scan root, with what the guard checks against; each part can be
/// changed by the test.
pub(crate) struct Setup {
    pub(crate) tree: FixtureTree,
    pub(crate) reach: Reach,
    pub(crate) user: u32,
    pub(crate) system: Protection,
    pub(crate) protection: Protection,
    pub(crate) roots: Roots,
    pub(crate) access: AccessStatus,
    pub(crate) guarded: Vec<GuardedPlace>,
    pub(crate) open: OpenSet,
    pub(crate) running: RunningSet,
    pub(crate) walk: Box<dyn SizeWalk>,
    pub(crate) cancel: CancelToken,
}

impl Setup {
    /// The tree described by `toml`, with the sandbox marker at its root.
    pub(crate) fn new(toml: &str) -> Self {
        Self::of(FixtureTree::from_toml_str(toml).unwrap().with_sandbox().unwrap())
    }

    /// The tree described by `toml`, with no sandbox marker.
    pub(crate) fn new_without_sandbox(toml: &str) -> Self {
        Self::of(FixtureTree::from_toml_str(toml).unwrap())
    }

    fn of(tree: FixtureTree) -> Self {
        let root = vec![tree.root().to_path_buf()];
        Self {
            reach: Reach::Sandbox,
            user: user_of(tree.root()),
            system: Protection::default(),
            protection: Protection::default(),
            roots: Roots::new(&root, &root),
            access: AccessStatus::default(),
            guarded: Vec::new(),
            open: OpenSet::default(),
            running: RunningSet::default(),
            walk: Box::new(HeldWalk),
            cancel: CancelToken::new(),
            tree,
        }
    }

    /// `relative` under the root, as the plan names it.
    pub(crate) fn at(&self, relative: &str) -> PathBuf {
        self.tree.root().join(relative)
    }

    /// `relative` under the root's canonical path, as the system names it.
    pub(crate) fn physical(&self, relative: &str) -> PathBuf {
        fs::canonicalize(self.tree.root()).unwrap().join(relative)
    }

    pub(crate) fn status(states: &[(Capability, Access)]) -> AccessStatus {
        AccessStatus {
            capabilities: states.iter().map(|&(capability, access)| CapabilityState::new(capability, access)).collect(),
        }
    }

    pub(crate) fn cx(&self, purpose: Purpose) -> GuardContext<'_> {
        GuardContext {
            reach: &self.reach,
            user: self.user,
            system: &self.system,
            protection: &self.protection,
            roots: &self.roots,
            access: &self.access,
            guarded: &self.guarded,
            open: &self.open,
            running: &self.running,
            purpose,
            walk: &*self.walk,
            cancel: &self.cancel,
        }
    }
}

/// The item at `relative` of the setup, as a rehearsal verifies it (the removals' tests act on it).
pub(crate) fn verified(setup: &Setup, relative: &str) -> crate::guard::Verified {
    let item = plan_item(1, Action::Remove(setup.at(relative)), 0);
    match crate::guard::verify(&item, &setup.cx(Purpose::Rehearse), fresher_core::Depth::Quick) {
        Ok(verified) => verified,
        Err(error) => unreachable!("{relative} did not verify: {error:?}"),
    }
}

/// A plan of `items` over the setup's root, confirmed with `method` by an echo of its own summary.
pub(crate) fn confirmed(
    setup: &Setup,
    items: &[PlanItem],
    method: fresher_core::Method,
) -> fresher_core::ConfirmedPlan {
    confirmed_over(&[setup.tree.root().to_path_buf()], items, method)
}

/// A plan of `items` over `roots`, confirmed with `method` by an echo of its own summary.
pub(crate) fn confirmed_over(
    roots: &[PathBuf],
    items: &[PlanItem],
    method: fresher_core::Method,
) -> fresher_core::ConfirmedPlan {
    use fresher_core::{CleanupPlan, Confirmation, PlanId, ScanId, Timestamp};

    let roots = roots.to_vec();
    let plan = CleanupPlan::new(
        PlanId::new(1),
        Timestamp::from_secs(1_000),
        ScanId::new(1),
        roots,
        items.to_vec(),
        Vec::new(),
    )
    .unwrap();
    let summary = plan.summary();
    let echo: Confirmation = serde_json::from_value(serde_json::json!({
        "plan": 1,
        "count": summary.count,
        "allocated": summary.sizes.allocated,
        "highest": summary.highest,
        "method": method,
        "highRiskUnderstood": summary.highest == DangerLevel::High,
    }))
    .unwrap();
    plan.confirm(&echo, Timestamp::from_secs(1_000)).unwrap()
}

/// One entry of a listing: its path below the root, file id, size, a link's target and a file's bytes.
pub(crate) type Entry = (PathBuf, u64, u64, Option<PathBuf>, Vec<u8>);

/// What a test sees of a tree: every entry below `root` (links not followed) with its kind, file id, size, a link's
/// target and a file's bytes, sorted by path.
pub(crate) fn listing(root: &Path) -> Vec<Entry> {
    fn collect(root: &Path, folder: &Path, entries: &mut Vec<Entry>) {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let target = metadata.file_type().is_symlink().then(|| fs::read_link(&path).unwrap());
            let bytes = if metadata.is_file() { fs::read(&path).unwrap() } else { Vec::new() };
            let relative = path.strip_prefix(root).unwrap().to_path_buf();
            entries.push((relative, metadata.ino(), metadata.len(), target, bytes));
            if metadata.is_dir() {
                collect(root, &path, entries);
            }
        }
    }
    let mut entries = Vec::new();
    collect(root, root, &mut entries);
    entries.sort();
    entries
}

/// A Trash that does something to the disk (or the clock, or the open files) on its first move, then moves like the
/// fake Trash it wraps.
pub(crate) struct Meddling {
    pub(crate) inner: Arc<FakeTrash>,
    pub(crate) first: Box<dyn Fn() + Send + Sync>,
    pub(crate) done: Mutex<bool>,
}

impl Meddling {
    pub(crate) fn new(inner: Arc<FakeTrash>, first: impl Fn() + Send + Sync + 'static) -> Self {
        Self { inner, first: Box::new(first), done: Mutex::new(false) }
    }
}

impl Trash for Meddling {
    fn move_to_trash(&self, path: &Path, held: &fs::File) -> Result<Landed, TrashError> {
        let first = {
            let mut done = self.done.lock().unwrap();
            !std::mem::replace(&mut *done, true)
        };
        if first {
            (self.first)();
        }
        self.inner.move_to_trash(path, held)
    }

    fn put_back(&self, landed: &Path, original: &Path) -> Result<(), TrashError> {
        self.inner.put_back(landed, original)
    }
}

/// A `DiskCleaner` on a sandboxed fixture tree, with the fakes of its ports kept at hand.
pub(crate) struct Rig {
    pub(crate) setup: Setup,
    pub(crate) trash: Arc<FakeTrash>,
    pub(crate) open_files: Arc<FakeOpenFiles>,
    pub(crate) running: Arc<FakeRunningApps>,
    pub(crate) clock: Arc<FakeClock>,
    pub(crate) runner: Arc<FakeCommandRunner>,
    pub(crate) cleaner: DiskCleaner,
    roots: Vec<PathBuf>,
}

impl Rig {
    pub(crate) fn new(toml: &str) -> Self {
        Self::with(Setup::new(toml), Reach::Sandbox)
    }

    /// The tree of `toml` with no sandbox marker, and a cleaner that checks it and acts on nothing. The tool of
    /// the tests (`docker`) is allowed, so a tool item reaches the stop before its act.
    #[cfg(debug_assertions)]
    pub(crate) fn dry_run(toml: &str) -> Self {
        let mut rig = Self::with(Setup::new_without_sandbox(toml), Reach::DryRun(crate::DryRunToken::new()));
        let docker = Capability::Tool(fresher_core::ToolId::new("docker").unwrap());
        rig.setup.access = Setup::status(&[(docker, Access::Granted)]);
        rig
    }

    fn with(setup: Setup, reach: Reach) -> Self {
        let trash = Arc::new(FakeTrash::new(setup.at(".Trash")));
        let open_files = Arc::new(FakeOpenFiles::default());
        let running = Arc::new(FakeRunningApps::default());
        let clock = Arc::new(FakeClock::at(1_000));
        let runner = Arc::new(FakeCommandRunner::default());
        let roots = vec![setup.tree.root().to_path_buf()];
        let ports = CleanerPorts {
            trash: Arc::clone(&trash) as Arc<dyn Trash>,
            open_files: Arc::clone(&open_files) as Arc<dyn OpenFiles>,
            running: Arc::clone(&running) as Arc<dyn RunningApps>,
            commands: Arc::clone(&runner) as Arc<dyn CommandRunner>,
            clock: Arc::clone(&clock) as Arc<dyn Clock>,
        };
        let home = setup.tree.root().to_path_buf();
        let cleaner = DiskCleaner::new(reach, setup.user, home, Protection::default(), ports);
        Self { setup, trash, open_files, running, clock, runner, cleaner, roots }
    }

    /// The same rig whose Trash does `first` on its first move.
    pub(crate) fn meddling(mut self, first: impl Fn() + Send + Sync + 'static) -> Self {
        self.cleaner.ports.trash = Arc::new(Meddling::new(Arc::clone(&self.trash), first));
        self
    }

    /// The same rig with the system's protection `system` gives.
    pub(crate) fn with_system(mut self, system: impl FnOnce(&Self) -> Protection) -> Self {
        self.cleaner.protection = system(&self);
        self
    }

    /// The same rig with another set of open files.
    pub(crate) fn with_open_files(mut self, open_files: FakeOpenFiles) -> Self {
        self.open_files = Arc::new(open_files);
        self.cleaner.ports.open_files = Arc::clone(&self.open_files) as Arc<dyn OpenFiles>;
        self
    }

    /// The same rig with other running apps.
    pub(crate) fn with_running(mut self, running: FakeRunningApps) -> Self {
        self.running = Arc::new(running);
        self.cleaner.ports.running = Arc::clone(&self.running) as Arc<dyn RunningApps>;
        self
    }

    pub(crate) fn at(&self, relative: &str) -> PathBuf {
        self.setup.at(relative)
    }

    /// The paths `relative` of the tree removed together, as one item, the first the subject.
    pub(crate) fn together(&self, relative: &[&str]) -> Action {
        let parts = relative.iter().map(|path| self.at(path)).collect();
        Action::RemoveTogether(fresher_core::Parts::new(parts).unwrap())
    }

    pub(crate) fn cx(&self) -> CleanupContext<'_> {
        CleanupContext {
            roots: &self.roots,
            protection: &self.setup.protection,
            access: &self.setup.access,
            guarded: &self.setup.guarded,
        }
    }

    /// `actions` as plan items (finding ids from 1), with the identities a quick rehearsal reads.
    pub(crate) fn items(&self, actions: Vec<Action>) -> Vec<PlanItem> {
        let items: Vec<PlanItem> = (1..).zip(actions).map(|(id, action)| plan_item(id, action, 4096)).collect();
        let rehearsed = self.cleaner.rehearse(&self.roots, &items, &self.cx(), fresher_core::Depth::Quick);
        items
            .into_iter()
            .zip(rehearsed)
            .map(|(item, rehearsal)| match rehearsal {
                fresher_core::Rehearsal::Passed(identity) => item.with_identity(identity),
                fresher_core::Rehearsal::PassedTogether(identities) => item.with_part_identities(identities),
                fresher_core::Rehearsal::Skipped(_) => item,
            })
            .collect()
    }

    /// Executes a plan of `items` confirmed with `method`, and gives every item's end in the order they came.
    pub(crate) fn execute(
        &self,
        items: &[PlanItem],
        method: fresher_core::Method,
        cancel: &CancelToken,
    ) -> (Vec<ItemDone>, Ended) {
        let plan = confirmed(&self.setup, items, method);
        let mut done = Vec::new();
        let ended = self.cleaner.execute(plan, &self.cx(), cancel, &mut |item| done.push(item));
        (done, ended)
    }
}

/// The physical path of a path action of the rig's tree, as the cleaner hands it to the Trash.
pub(crate) fn physical_of(rig: &Rig, action: &Action) -> PathBuf {
    let path = action.path().unwrap();
    let relative = path.strip_prefix(rig.setup.tree.root()).unwrap();
    rig.setup.physical(&relative.to_string_lossy())
}

/// A walk that answers what the test says, and counts its calls.
#[derive(Debug)]
pub(crate) struct FixedWalk {
    answer: Result<u64, WalkStop>,
    calls: Mutex<u32>,
}

impl FixedWalk {
    pub(crate) fn answering(answer: Result<u64, WalkStop>) -> Self {
        Self { answer, calls: Mutex::new(0) }
    }

    pub(crate) fn calls(&self) -> u32 {
        *self.calls.lock().unwrap()
    }
}

impl SizeWalk for FixedWalk {
    fn measure(
        &self,
        _item: &fs::File,
        _metadata: &fs::Metadata,
        _device: u64,
        _cancel: &CancelToken,
    ) -> Result<u64, WalkStop> {
        *self.calls.lock().unwrap() += 1;
        self.answer.clone()
    }
}

impl SizeWalk for Arc<FixedWalk> {
    fn measure(
        &self,
        item: &fs::File,
        metadata: &fs::Metadata,
        device: u64,
        cancel: &CancelToken,
    ) -> Result<u64, WalkStop> {
        (**self).measure(item, metadata, device, cancel)
    }
}

/// Sets the mode of `path`, and gives the old one back when dropped, so the temporary tree can be removed.
pub(crate) struct Mode(PathBuf, u32);

impl Mode {
    pub(crate) fn set(path: &Path, mode: u32) -> Self {
        let old = fs::metadata(path).unwrap().mode() & 0o7777;
        fs::set_permissions(path, Permissions::from_mode(mode)).unwrap();
        Self(path.to_path_buf(), old)
    }
}

impl Drop for Mode {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.0, Permissions::from_mode(self.1));
    }
}

/// Locks `path` with the user's immutable flag (`chflags uchg`), and unlocks it when dropped.
pub(crate) struct Flags(PathBuf);

impl Flags {
    pub(crate) fn lock(path: &Path) -> Self {
        let locked = Command::new("/usr/bin/chflags").arg("uchg").arg(path).status().unwrap();
        assert!(locked.success(), "chflags could not lock {}", path.display());
        Self(path.to_path_buf())
    }
}

impl Drop for Flags {
    fn drop(&mut self) {
        let _ = Command::new("/usr/bin/chflags").arg("nouchg").arg(&self.0).status();
    }
}

const HDIUTIL: &str = "/usr/bin/hdiutil";

/// A small disk image of the test, mounted at a folder of its fixture tree with `-nobrowse`, detached when dropped.
#[derive(Debug)]
pub(crate) struct DiskImage {
    mount_point: PathBuf,
    _image: TempDir,
}

impl DiskImage {
    /// Mounts a new image at `folder`, which is created; `None` when `hdiutil` is not on this Mac (the test that needs
    /// it is skipped).
    pub(crate) fn mount_at(folder: &Path) -> Option<Self> {
        if !Path::new(HDIUTIL).exists() {
            return None;
        }
        let image_folder = tempfile::tempdir().unwrap();
        let image = image_folder.path().join("image.dmg");
        fs::create_dir_all(folder).unwrap();
        let created = hdiutil(&[
            "create".as_ref(),
            "-quiet".as_ref(),
            "-ov".as_ref(),
            "-size".as_ref(),
            "8m".as_ref(),
            "-fs".as_ref(),
            "APFS".as_ref(),
            "-volname".as_ref(),
            "FresherCleanerTest".as_ref(),
            image.as_os_str(),
        ]);
        assert!(created, "hdiutil could not create the image");
        let attached = hdiutil(&[
            "attach".as_ref(),
            "-quiet".as_ref(),
            "-nobrowse".as_ref(),
            "-noautoopen".as_ref(),
            "-mountpoint".as_ref(),
            folder.as_os_str(),
            image.as_os_str(),
        ]);
        assert!(attached, "hdiutil could not attach the image");
        Some(Self { mount_point: folder.to_path_buf(), _image: image_folder })
    }

    pub(crate) fn mount_point(&self) -> &Path {
        &self.mount_point
    }
}

impl Drop for DiskImage {
    fn drop(&mut self) {
        let point = self.mount_point.as_os_str();
        if !hdiutil(&["detach".as_ref(), "-quiet".as_ref(), point]) {
            hdiutil(&["detach".as_ref(), "-quiet".as_ref(), "-force".as_ref(), point]);
        }
    }
}

/// Runs `hdiutil`, trying again a few times: images made by tests running at once sometimes find it busy.
fn hdiutil(arguments: &[&OsStr]) -> bool {
    (0..5).any(|attempt| {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        Command::new(HDIUTIL).args(arguments).status().is_ok_and(|status| status.success())
    })
}
