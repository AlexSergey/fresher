//! What the cleaner never does, for any plan: random trees with links in, out and up, nested folders, a protected,
//! a hard-blocked, a carved-out and an ignored folder inside the scan root, the temporary places of a Mac laid out
//! inside it with their protection (every user's folders `never` but the user's own `T`, which is `whole`; its
//! `TemporaryItems` and the system's temporary folder `never`; the shared one `whole`), and a folder outside it;
//! random plans of real and made-up paths (outside the root, through links, dotted, the root itself, protected places
//! and their ancestors) with a random action and method. For each: a rehearsal at both depths changes nothing at all, and the
//! execution removes only plan items or what is inside them, strictly inside the root, never protected, ignored, a
//! hard-blocked folder or an ancestor of one; the outside folder and every protected subtree list exactly as before.
//! The same holds for a permanent cleanup while a folder of the root is swapped for a link to the outside folder and
//! back on another thread, and for the reach of one sandbox folder: nothing outside that folder ever goes. A dry run
//! (a development build on the real Mac, no marker anywhere) changes nothing at all and runs no tool. Failing seeds
//! are kept in `never.proptest-regressions`.

#[cfg(test)]
mod model {
    use std::collections::BTreeMap;
    use std::fs;
    use std::os::unix::fs::{MetadataExt as _, symlink};
    use std::path::{Component, Path, PathBuf};
    use std::sync::Arc;

    use fresher_cleaner::{CleanerPorts, DiskCleaner, Reach, SandboxFolder};
    use fresher_core::{
        AccessStatus, Action, ActionKind, ByteSize, CancelToken, CategoryDescriptor, CategoryId, Cleaner as _,
        CleanupContext, CleanupPlan, Confirmation, ConfirmedPlan, DangerLevel, Depth, Finding, FindingId, ItemDone,
        ItemOutcome, Method, ModuleId, NewFinding, Parts, PlanId, PlanItem, Protection, Rehearsal, Removability,
        Removal, SANDBOX_MARKER, SANDBOX_SIGNATURE, ScanId, Sizes, Subject, Text, TextKey, Timestamp, ToolCommand,
        ToolId,
    };
    use fresher_testing::{FakeClock, FakeCommandRunner, FakeOpenFiles, FakeTrash};
    use tempfile::TempDir;

    /// The scan root, relative to the tree.
    pub(crate) const SCAN: &str = "scan";
    /// A `never` place, its carve-out, a `whole` folder and an ignored folder, inside the scan root.
    pub(crate) const NEVER: &str = "scan/b/sys";
    pub(crate) const EXCEPT: &str = "scan/b/sys/local";
    pub(crate) const WHOLE: &str = "scan/home";
    pub(crate) const IGNORED: &str = "scan/a/ign";
    /// The folder outside every root.
    pub(crate) const OUTSIDE: &str = "outside";
    /// The temporary places of a Mac, as `crates/macos/src/protection.rs` protects them: every user's folders
    /// (`never`), the user's own `T` carved out of them (and `whole`), its `TemporaryItems`, another user's `T` and
    /// the user's own `C`; the system's temporary folder (`never`) and the shared one (`whole`).
    pub(crate) const USER_FOLDERS: &str = "scan/private/var/folders";
    pub(crate) const USER_TEMP: &str = "scan/private/var/folders/xx/me/T";
    pub(crate) const TEMPORARY_ITEMS: &str = "scan/private/var/folders/xx/me/T/TemporaryItems";
    pub(crate) const USER_CACHES: &str = "scan/private/var/folders/xx/me/C";
    pub(crate) const OTHER_TEMP: &str = "scan/private/var/folders/yy/other/T";
    pub(crate) const SYSTEM_TEMP: &str = "scan/private/var/tmp";
    pub(crate) const SHARED_TEMP: &str = "scan/private/tmp";
    /// The places of each list.
    const NEVER_PLACES: [&str; 4] = [NEVER, USER_FOLDERS, TEMPORARY_ITEMS, SYSTEM_TEMP];
    const EXCEPT_PLACES: [&str; 2] = [EXCEPT, USER_TEMP];
    const WHOLE_PLACES: [&str; 3] = [WHOLE, USER_TEMP, SHARED_TEMP];
    /// The folders every tree has.
    pub(crate) const BASES: [&str; 15] = [
        SCAN,
        "scan/a",
        IGNORED,
        "scan/b",
        NEVER,
        EXCEPT,
        WHOLE,
        OUTSIDE,
        USER_FOLDERS,
        USER_TEMP,
        TEMPORARY_ITEMS,
        USER_CACHES,
        OTHER_TEMP,
        SYSTEM_TEMP,
        SHARED_TEMP,
    ];
    const NAMES: [&str; 3] = ["x", "y", "z"];
    /// Where a link may point, relative to the tree (`None`: up and out by `..`).
    const TARGETS: [Option<&str>; 9] = [
        Some(OUTSIDE),
        Some("outside/o.txt"),
        Some("scan/a"),
        Some(NEVER),
        Some(""),
        Some(WHOLE),
        Some(USER_CACHES),
        Some(USER_TEMP),
        None,
    ];

    /// One generated entry: a base folder, the names below it, and what the last one is.
    #[derive(Clone, Debug)]
    pub(crate) enum Leaf {
        File,
        Folder,
        Link(usize),
    }

    /// One entry as listed: its file id, whether it is a folder, a link's target and a file's bytes.
    pub(crate) type Listed = (u64, bool, Option<PathBuf>, Vec<u8>);

    pub(crate) struct Tree {
        pub(crate) dir: TempDir,
    }

    impl Tree {
        pub(crate) fn build(entries: &[(usize, Vec<usize>, Leaf)]) -> Self {
            let tree = Self { dir: tempfile::tempdir().unwrap() };
            for base in BASES {
                fs::create_dir_all(tree.at(base)).unwrap();
            }
            fs::write(tree.at("outside/o.txt"), b"outside").unwrap();
            fs::write(tree.at(SCAN).join(SANDBOX_MARKER), format!("{SANDBOX_SIGNATURE}\n")).unwrap();
            for (base, names, leaf) in entries {
                let mut path = tree.at(BASES[*base % BASES.len()]);
                for name in names {
                    path.push(NAMES[*name % NAMES.len()]);
                }
                // Creating through a file or a link fails, and is left out: no entry is made through a link.
                let _ = fs::create_dir_all(&path).and_then(|()| match leaf {
                    Leaf::File => fs::write(path.join("f"), b"file"),
                    Leaf::Folder => fs::create_dir_all(path.join("d")),
                    Leaf::Link(target) => symlink(tree.target(*target), path.join("l")),
                });
            }
            tree
        }

        pub(crate) fn root(&self) -> &Path {
            self.dir.path()
        }

        pub(crate) fn at(&self, relative: &str) -> PathBuf {
            self.root().join(relative)
        }

        fn target(&self, index: usize) -> PathBuf {
            match TARGETS[index % TARGETS.len()] {
                Some(relative) => self.at(relative),
                None => PathBuf::from("../../.."),
            }
        }

        /// Every entry below the root, links not followed, by its path relative to the root.
        pub(crate) fn listing(&self) -> BTreeMap<PathBuf, Listed> {
            let mut entries = BTreeMap::new();
            collect(self.root(), self.root(), &mut entries);
            entries
        }

        /// The paths a plan may name: every entry, then made-up ones.
        pub(crate) fn candidates(&self) -> Vec<PathBuf> {
            let mut paths: Vec<PathBuf> = self.listing().keys().map(|relative| self.root().join(relative)).collect();
            let canonical = fs::canonicalize(self.root()).unwrap();
            let links: Vec<PathBuf> = paths.iter().filter(|path| path.is_symlink()).cloned().collect();
            for link in links {
                paths.push(link.join("o.txt"));
                paths.push(link.join("x"));
            }
            paths.extend([
                self.root().to_path_buf(),
                PathBuf::from("/etc/hosts"),
                PathBuf::from("/"),
                self.at("scan/.."),
                self.at("scan/a/../b"),
                self.at("scan/./a"),
                self.at("scan/nothing"),
                self.at("scan2"),
                canonical.join("scan/a"),
                canonical.join("scan/b/x"),
            ]);
            paths
        }
    }

    fn collect(root: &Path, folder: &Path, entries: &mut BTreeMap<PathBuf, Listed>) {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            let target = metadata.file_type().is_symlink().then(|| fs::read_link(&path).unwrap());
            let bytes = if metadata.is_file() { fs::read(&path).unwrap() } else { Vec::new() };
            entries.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                (metadata.ino(), metadata.is_dir(), target, bytes),
            );
            if metadata.is_dir() {
                collect(root, &path, entries);
            }
        }
    }

    /// A plan item of finding `id`.
    pub(crate) fn item(id: u32, action: Action) -> PlanItem {
        let module = ModuleId::new("demo").unwrap();
        let category = CategoryDescriptor::new(CategoryId::new(module, "demo.caches").unwrap(), DangerLevel::Low);
        let text = |key: &str| Text::new(TextKey::new(key).unwrap());
        let (subject, removal) = match action {
            Action::Remove(path) => (Subject::Path(path.clone()), Removal::MoveToTrash(path)),
            Action::Empty(path) => (Subject::Path(path.clone()), Removal::EmptyFolder(path)),
            Action::RemoveTogether(parts) => {
                (Subject::Path(parts.subject().to_path_buf()), Removal::MoveTogether(parts))
            }
            Action::Tool(command) => {
                (Subject::External { tool: command.tool(), id: "1".into() }, Removal::ToolCommand(command))
            }
        };
        let sizes = Sizes::complete(ByteSize::new(0), ByteSize::new(4096));
        let new = NewFinding::new(
            &category,
            format!("item-{id}"),
            subject,
            sizes,
            text("t.title"),
            text("t.reason"),
            removal,
        );
        PlanItem::new(&Finding::new(FindingId::new(id), new, Removability::Removable))
    }

    pub(crate) fn tool() -> Action {
        Action::Tool(ToolCommand::new(ToolId::new("docker").unwrap(), vec!["image".into(), "rm".into()]).unwrap())
    }

    /// The cleaner of the app in the sandbox, on the tree, with the fake ports, and what it is called with.
    pub(crate) struct Bench {
        pub(crate) tree: Tree,
        pub(crate) trash: Arc<FakeTrash>,
        pub(crate) runner: Arc<FakeCommandRunner>,
        pub(crate) cleaner: DiskCleaner,
        pub(crate) roots: Vec<PathBuf>,
        pub(crate) protection: Protection,
        pub(crate) access: AccessStatus,
    }

    impl Bench {
        pub(crate) fn new(tree: Tree) -> Self {
            Self::within(tree, Reach::Sandbox)
        }

        /// The bench whose cleaner reaches only the folder `folder` of the tree, marked as a sandbox.
        pub(crate) fn in_folder(tree: Tree, folder: &str) -> Self {
            let path = tree.at(folder);
            fs::write(path.join(SANDBOX_MARKER), format!("{SANDBOX_SIGNATURE}\n")).unwrap();
            let user = fs::metadata(tree.root()).unwrap().uid();
            let reach = Reach::SandboxFolder(SandboxFolder::new(&path, user, &Protection::default()).unwrap());
            Self::within(tree, reach)
        }

        /// The bench whose cleaner checks the tree as the real disk and acts on nothing, with no valid sandbox marker
        /// in the tree (the one the tree was built with is blanked: a wrong signature does not count) and the tool
        /// allowed, so that every kind of item can reach its act.
        #[cfg(debug_assertions)]
        pub(crate) fn dry_run(tree: Tree) -> Self {
            fs::write(tree.at(SCAN).join(SANDBOX_MARKER), b"").unwrap();
            let mut bench = Self::within(tree, Reach::DryRun(fresher_cleaner::DryRunToken::new()));
            let docker = fresher_core::Capability::Tool(ToolId::new("docker").unwrap());
            bench.access = AccessStatus {
                capabilities: vec![fresher_core::CapabilityState::new(docker, fresher_core::Access::Granted)],
            };
            bench
        }

        /// The bench whose Trash refuses the path `relative` of the tree, as macOS does for a locked item.
        pub(crate) fn refusing(tree: Tree, relative: &Path) -> Self {
            let physical = fs::canonicalize(tree.root()).unwrap().join(relative);
            let trash = FakeTrash::new(tree.at(".Trash")).refusing(physical);
            Self::with_trash(tree, Reach::Sandbox, trash)
        }

        fn within(tree: Tree, reach: Reach) -> Self {
            let trash = FakeTrash::new(tree.at(".Trash"));
            Self::with_trash(tree, reach, trash)
        }

        fn with_trash(tree: Tree, reach: Reach, trash: FakeTrash) -> Self {
            let trash = Arc::new(trash);
            let runner = Arc::new(FakeCommandRunner::default());
            let ports = CleanerPorts {
                trash: Arc::clone(&trash) as _,
                open_files: Arc::new(FakeOpenFiles::default()),
                running: Arc::new(fresher_testing::FakeRunningApps::default()),
                commands: Arc::clone(&runner) as _,
                clock: Arc::new(FakeClock::at(1_000)),
            };
            let user = fs::metadata(tree.root()).unwrap().uid();
            let cleaner = DiskCleaner::new(reach, user, tree.root().to_path_buf(), Protection::default(), ports);
            let at = |places: &[&str]| places.iter().map(|place| tree.at(place)).collect();
            let protection = Protection::new(at(&NEVER_PLACES), at(&WHOLE_PLACES), at(&EXCEPT_PLACES))
                .unwrap()
                .with_ignored(&[tree.at(IGNORED)])
                .unwrap();
            let roots = vec![tree.at(SCAN)];
            Self { tree, trash, runner, cleaner, roots, protection, access: AccessStatus::default() }
        }

        pub(crate) fn cx(&self) -> CleanupContext<'_> {
            CleanupContext { roots: &self.roots, protection: &self.protection, access: &self.access, guarded: &[] }
        }

        pub(crate) fn rehearse(&self, items: &[PlanItem], depth: Depth) -> Vec<Rehearsal> {
            self.cleaner.rehearse(&self.roots, items, &self.cx(), depth)
        }

        /// The items with the identities a quick rehearsal reads, confirmed with `method`, executed.
        pub(crate) fn execute(&self, items: Vec<PlanItem>, method: Method) -> Vec<ItemDone> {
            self.run(self.confirm(items, method))
        }

        /// The items with the identities a quick rehearsal reads, as a plan confirmed with `method`.
        pub(crate) fn confirm(&self, items: Vec<PlanItem>, method: Method) -> ConfirmedPlan {
            let rehearsed = self.rehearse(&items, Depth::Quick);
            let items: Vec<PlanItem> = items
                .into_iter()
                .zip(rehearsed)
                .map(|(item, rehearsal)| match rehearsal {
                    Rehearsal::Passed(identity) => item.with_identity(identity),
                    Rehearsal::PassedTogether(identities) => item.with_part_identities(identities),
                    Rehearsal::Skipped(_) => item,
                })
                .collect();
            let plan = CleanupPlan::new(
                PlanId::new(1),
                Timestamp::from_secs(1),
                ScanId::new(1),
                self.roots.clone(),
                items,
                vec![],
            )
            .unwrap();
            let summary = plan.summary();
            let echo: Confirmation = serde_json::from_value(serde_json::json!({
                "plan": 1, "count": summary.count, "allocated": summary.sizes.allocated,
                "highest": summary.highest, "method": method,
                "highRiskUnderstood": summary.highest == DangerLevel::High,
            }))
            .unwrap();
            plan.confirm(&echo, Timestamp::from_secs(1)).unwrap()
        }

        /// Executes the confirmed plan and gives every item's end.
        pub(crate) fn run(&self, plan: ConfirmedPlan) -> Vec<ItemDone> {
            let mut done = Vec::new();
            self.cleaner.execute(plan, &self.cx(), &CancelToken::new(), &mut |item| done.push(item));
            done
        }
    }

    fn inside(path: &Path, place: &str) -> bool {
        path.starts_with(place)
    }

    fn strictly_inside(path: &Path, place: &str) -> bool {
        path.starts_with(place) && path != Path::new(place)
    }

    fn above(path: &Path, place: &str) -> bool {
        Path::new(place).starts_with(path)
    }

    /// The model of what may go, by plain path components relative to the tree: strictly inside the scan root, not
    /// in or above a `never` place unless strictly inside one of its own carve-outs, not a carve-out, a `whole`
    /// folder or above them, not in or above the ignored folder.
    pub(crate) fn may_go(path: &Path) -> bool {
        strictly_inside(path, SCAN)
            && !is_kept(path)
            && !NEVER_PLACES.iter().chain(&EXCEPT_PLACES).chain(&WHOLE_PLACES).any(|place| above(path, place))
            && !above(path, IGNORED)
    }

    /// In a `never` place outside its own carve-outs, or in the ignored folder: must list exactly as before.
    pub(crate) fn is_kept(path: &Path) -> bool {
        let carved = |never: &str| {
            EXCEPT_PLACES.iter().any(|except| Path::new(except).starts_with(never) && strictly_inside(path, except))
        };
        NEVER_PLACES.iter().any(|never| inside(path, never) && !carved(never)) || inside(path, IGNORED)
    }

    /// Every path an item acts on relative to the tree, in order: `None` for one that is not a path of it.
    pub(crate) fn parts(tree: &Tree, item: &PlanItem) -> Vec<Option<PathBuf>> {
        item.action().parts().iter().map(|part| part.strip_prefix(tree.root()).ok().map(Path::to_path_buf)).collect()
    }

    /// The picked paths as one item of several paths, the first the subject: a path that is not plain, or that is,
    /// lies in or above a path kept before it, is left out; a remove of the first when fewer than two are left.
    pub(crate) fn together(paths: Vec<PathBuf>) -> Action {
        let first = paths[0].clone();
        let mut kept: Vec<PathBuf> = Vec::new();
        for path in paths {
            let plain = path.components().all(|part| matches!(part, Component::RootDir | Component::Normal(_)));
            if plain && !kept.iter().any(|earlier| earlier.starts_with(&path) || path.starts_with(earlier)) {
                kept.push(path);
            }
        }
        Parts::new(kept).map_or(Action::Remove(first), Action::RemoveTogether)
    }

    /// Whether `outcome` acted on the item.
    pub(crate) fn acted(outcome: &ItemOutcome) -> bool {
        matches!(
            outcome,
            ItemOutcome::Moved | ItemOutcome::Deleted | ItemOutcome::Emptied { .. } | ItemOutcome::ToolDone
        )
    }

    /// Gives every folder and file below `from` a twin of the same name below `to` (links are left out), so whatever
    /// an item below `from` is, the same path through a link to `to` names something.
    pub(crate) fn twin(from: &Path, to: &Path) {
        for entry in fs::read_dir(from).unwrap().map(Result::unwrap) {
            let kind = entry.file_type().unwrap();
            let twin = to.join(entry.file_name());
            if kind.is_dir() {
                fs::create_dir_all(&twin).unwrap();
                self::twin(&entry.path(), &twin);
            } else if kind.is_file() && !twin.exists() {
                fs::write(&twin, b"twin").unwrap();
            }
        }
    }

    pub(crate) fn is_path_action(item: &PlanItem) -> bool {
        item.action().kind() != ActionKind::Tool
    }
}

mod negative {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    use fresher_core::{Action, Depth, Failure, ItemDone, ItemOutcome, Method, PartlyWay, PlanItem};
    use fresher_testing::Swapper;
    use proptest::prelude::*;
    use proptest::sample::Index;
    use proptest::test_runner::TestCaseError;

    use crate::model::{
        BASES, Bench, Leaf, Listed, OUTSIDE, Tree, acted, is_kept, is_path_action, item, may_go, parts, together, tool,
        twin,
    };

    fn leaf() -> impl Strategy<Value = Leaf> {
        prop_oneof![Just(Leaf::File), Just(Leaf::Folder), (0_usize..9).prop_map(Leaf::Link)]
    }

    fn entries() -> impl Strategy<Value = Vec<(usize, Vec<usize>, Leaf)>> {
        prop::collection::vec((0..BASES.len(), prop::collection::vec(0_usize..3, 0..3), leaf()), 0..14)
    }

    /// One to four more picks: with the first, the parts of an item of several paths.
    fn more() -> impl Strategy<Value = Vec<Index>> {
        prop::collection::vec(any::<Index>(), 1..5)
    }

    /// Picks of the tree's candidate paths, each with an action: 0 remove, 1 empty, 2 a tool, 3 the path and the
    /// item's more picks together.
    fn picks() -> impl Strategy<Value = Vec<(Index, u8, Vec<Index>)>> {
        let action = prop_oneof![4 => Just(0_u8), 3 => Just(1_u8), 1 => Just(2_u8), 2 => Just(3_u8)];
        prop::collection::vec((any::<Index>(), action, more()), 1..12)
    }

    /// The paths `first` and `more` pick of the tree's candidates, as one item of several paths when they make one.
    fn together_of(candidates: &[PathBuf], first: Index, more: &[Index]) -> Action {
        together(std::iter::once(&first).chain(more).map(|pick| pick.get(candidates).clone()).collect())
    }

    fn plan(bench: &Bench, picks: &[(Index, u8, Vec<Index>)]) -> Vec<PlanItem> {
        let candidates = bench.tree.candidates();
        (1..)
            .zip(picks)
            .map(|(id, (pick, action, more))| {
                let path = pick.get(&candidates).clone();
                let action = match action {
                    0 => Action::Remove(path),
                    1 => Action::Empty(path),
                    2 => tool(),
                    _ => together_of(&candidates, *pick, more),
                };
                item(id, action)
            })
            .collect()
    }

    /// Whether the entry `path`, listed before, is gone or another entry now.
    fn went(path: &Path, before: &BTreeMap<PathBuf, Listed>, after: &BTreeMap<PathBuf, Listed>) -> bool {
        before.get(path).is_some_and(|listed| after.get(path).is_none_or(|now| now.0 != listed.0))
    }

    /// Whether the item ended so that what lies under its paths may have gone: acted on, or an item of several paths
    /// that failed after some of its parts went.
    fn may_have_removed(item: &PlanItem, done: &ItemDone) -> bool {
        acted(&done.outcome)
            || matches!(item.action(), Action::RemoveTogether(_)) && matches!(done.outcome, ItemOutcome::Failed { .. })
    }

    /// Whether `path` lies under a path of `item` (strictly inside the folder of an emptied one).
    fn under(bench: &Bench, item: &PlanItem, path: &Path) -> bool {
        parts(&bench.tree, item).into_iter().flatten().any(|planned| match item.action() {
            Action::Empty(_) => path.starts_with(&planned) && *path != planned,
            _ => path.starts_with(&planned),
        })
    }

    /// Whether `path` lies in a place that must list exactly as before.
    fn kept(path: &Path) -> bool {
        is_kept(path) || path.starts_with(OUTSIDE)
    }

    /// What holds after any execution: no tool ran, only plan items that may go were acted on, everything that went
    /// may go and lay under an item acted on, and the kept places and the base folders list exactly as before.
    fn holds(
        bench: &Bench,
        items: &[PlanItem],
        before: &BTreeMap<PathBuf, Listed>,
        done: &[ItemDone],
    ) -> Result<(), TestCaseError> {
        let after = bench.tree.listing();
        prop_assert_eq!(done.len(), items.len());
        prop_assert!(bench.runner.calls().is_empty(), "a tool ran in the sandbox");
        for (index, item) in items.iter().enumerate() {
            if acted(&done[index].outcome) {
                prop_assert!(is_path_action(item));
                for part in parts(&bench.tree, item) {
                    prop_assert!(part.as_deref().is_some_and(may_go), "acted on {:?}", item.action());
                }
            }
        }
        together_whole_or_not_at_all(bench, items, before, &after, done)?;
        let gone: Vec<&PathBuf> = before
            .iter()
            .filter(|(path, listed)| after.get(*path).is_none_or(|now| now.0 != listed.0))
            .map(|(path, _)| path)
            .collect();
        for path in gone {
            prop_assert!(may_go(path), "{} went", path.display());
            let covered =
                items.iter().zip(done).any(|(item, done)| may_have_removed(item, done) && under(bench, item, path));
            prop_assert!(covered, "{} went without a plan item over it", path.display());
        }
        for (path, listed) in before {
            if kept(path) {
                prop_assert_eq!(after.get(path), Some(listed), "{} changed", path.display());
            }
        }
        for folder in BASES {
            let path = PathBuf::from(folder);
            prop_assert_eq!(after.get(&path).map(|listed| listed.0), before.get(&path).map(|listed| listed.0));
        }
        Ok(())
    }

    /// An item of several paths acted on went whole: every part was listed before and is gone. One skipped or not
    /// reached moved no part: a part gone lies under another item that may have removed it.
    fn together_whole_or_not_at_all(
        bench: &Bench,
        items: &[PlanItem],
        before: &BTreeMap<PathBuf, Listed>,
        after: &BTreeMap<PathBuf, Listed>,
        done: &[ItemDone],
    ) -> Result<(), TestCaseError> {
        for (index, item) in items.iter().enumerate() {
            if !matches!(item.action(), Action::RemoveTogether(_)) {
                continue;
            }
            let parts = parts(&bench.tree, item);
            if acted(&done[index].outcome) {
                for part in &parts {
                    let gone = part.as_deref().is_some_and(|part| went(part, before, after));
                    prop_assert!(gone, "{:?} acted on, but {:?} stayed", item.action(), part);
                }
            } else if !matches!(done[index].outcome, ItemOutcome::Failed { .. }) {
                for part in parts.iter().flatten().filter(|part| went(part, before, after)) {
                    let other = (0..items.len()).filter(|other| *other != index).any(|other| {
                        may_have_removed(&items[other], &done[other]) && under(bench, &items[other], part)
                    });
                    prop_assert!(other, "{} went, its item {:?}", part.display(), done[index].outcome);
                }
                prop_assert!(done[index].trashed.is_empty());
            }
        }
        Ok(())
    }

    proptest! {
        /// One item of several paths that may go, now and then with one of any of the tree's candidate paths among
        /// them, the Trash refusing one of its parts or none: every part goes or none does, unless the Trash refuses
        /// a part at its act; then the parts before it went, the rest stayed, and the item says only part of it went
        /// (or, for the subject, that the Trash refused it), with what went to put back.
        #[test]
        fn a_together_item_goes_whole_or_not_at_all(
            entries in entries(),
            first in any::<Index>(),
            more in more(),
            stranger in proptest::option::of((any::<Index>(), any::<Index>())),
            permanent in any::<bool>(),
            refused in proptest::option::of(any::<Index>()),
        ) {
            let tree = Tree::build(&entries);
            let candidates = tree.candidates();
            let goers: Vec<PathBuf> =
                candidates.iter().filter(|path| path.strip_prefix(tree.root()).is_ok_and(may_go)).cloned().collect();
            if goers.len() < 2 {
                return Ok(());
            }
            let mut picked: Vec<PathBuf> = std::iter::once(&first).chain(&more).map(|pick| pick.get(&goers).clone()).collect();
            if let Some((at, pick)) = stranger {
                picked.insert(at.index(picked.len() + 1), pick.get(&candidates).clone());
            }
            let action = together(picked);
            let Action::RemoveTogether(paths) = &action else { return Ok(()) };
            let relative: Vec<PathBuf> =
                paths.all().iter().map(|part| part.strip_prefix(tree.root()).unwrap_or(part).to_path_buf()).collect();
            let refused = refused.map(|pick| pick.index(relative.len()));
            let bench = match refused {
                Some(index) => Bench::refusing(tree, &relative[index]),
                None => Bench::new(tree),
            };
            let before = bench.tree.listing();
            let method = if permanent { Method::Permanent } else { Method::Trash };

            let done = bench.execute(vec![item(1, action.clone())], method);

            let after = bench.tree.listing();
            let outcome = &done[0].outcome;
            let refusing = refused.filter(|_| !permanent);
            match (outcome, refusing) {
                (ItemOutcome::Moved | ItemOutcome::Deleted, None) => {
                    for part in &relative {
                        prop_assert!(went(part, &before, &after), "{} stayed", part.display());
                    }
                    let journalled = if permanent { 0 } else { relative.len() };
                    prop_assert_eq!(done[0].trashed.len(), journalled);
                }
                (ItemOutcome::Failed { reason }, Some(index)) => {
                    let of = u32::try_from(relative.len()).unwrap();
                    let done_parts = u32::try_from(index).unwrap();
                    let expected = if index == 0 {
                        Failure::TrashRefused.text()
                    } else {
                        Failure::Partly { done: done_parts, of, way: PartlyWay::Trash }.text()
                    };
                    prop_assert_eq!(reason, &expected);
                    for (at, part) in relative.iter().enumerate() {
                        prop_assert_eq!(went(part, &before, &after), at < index, "{}", part.display());
                    }
                    prop_assert_eq!(done[0].trashed.len(), index);
                    prop_assert_eq!(bench.trash.moved().len(), index);
                }
                (ItemOutcome::Skipped { .. } | ItemOutcome::NotReached, _) => {
                    prop_assert_eq!(&after, &before, "a refused item changed the tree");
                    prop_assert!(done[0].trashed.is_empty());
                }
                (outcome, refusing) => prop_assert!(false, "{:?} ended {:?} (refusing {:?})", action, outcome, refusing),
            }
            holds(&bench, &[item(1, action.clone())], &before, &done)?;
        }
    }

    proptest! {
        #[test]
        fn no_plan_removes_anything_it_may_not(entries in entries(), picks in picks(), permanent in any::<bool>()) {
            let bench = Bench::new(Tree::build(&entries));
            let items = plan(&bench, &picks);
            let before = bench.tree.listing();

            for depth in [Depth::Quick, Depth::Full] {
                bench.rehearse(&items, depth);
                prop_assert_eq!(&bench.tree.listing(), &before, "a rehearsal changed the tree");
            }
            prop_assert!(bench.trash.moved().is_empty());

            let method = if permanent { Method::Permanent } else { Method::Trash };
            let done = bench.execute(items.clone(), method);

            holds(&bench, &items, &before, &done)?;
        }
    }

    proptest! {
        /// The reach of one sandbox folder, any base folder of the tree: on top of everything above, nothing outside
        /// that folder is ever acted on, moved or deleted, and the folder itself stays.
        #[test]
        fn nothing_outside_the_sandbox_folder_goes(
            entries in entries(),
            picks in picks(),
            permanent in any::<bool>(),
            folder in 0..BASES.len(),
        ) {
            let folder = BASES[folder];
            let bench = Bench::in_folder(Tree::build(&entries), folder);
            let items = plan(&bench, &picks);
            let before = bench.tree.listing();

            for depth in [Depth::Quick, Depth::Full] {
                bench.rehearse(&items, depth);
                prop_assert_eq!(&bench.tree.listing(), &before, "a rehearsal changed the tree");
            }
            let method = if permanent { Method::Permanent } else { Method::Trash };
            let done = bench.execute(items.clone(), method);

            holds(&bench, &items, &before, &done)?;
            let strictly_inside = |path: &Path| path.starts_with(folder) && path != Path::new(folder);
            for (item, done) in items.iter().zip(&done) {
                if acted(&done.outcome) {
                    for part in parts(&bench.tree, item) {
                        prop_assert!(part.as_deref().is_some_and(strictly_inside), "acted on {:?}", item.action());
                    }
                }
            }
            let after = bench.tree.listing();
            for (path, listed) in &before {
                if !strictly_inside(path) {
                    prop_assert_eq!(after.get(path), Some(listed), "{} changed", path.display());
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    proptest! {
        /// A development build on the real Mac: whatever the tree, the plan and the method, every item passes or fails
        /// its checks and nothing is moved, emptied, deleted or run.
        #[test]
        fn a_dry_run_changes_nothing_in_any_tree(entries in entries(), picks in picks(), permanent in any::<bool>()) {
            let bench = Bench::dry_run(Tree::build(&entries));
            let items = plan(&bench, &picks);
            let before = bench.tree.listing();

            for depth in [Depth::Quick, Depth::Full] {
                bench.rehearse(&items, depth);
            }
            let method = if permanent { Method::Permanent } else { Method::Trash };
            let done = bench.execute(items.clone(), method);

            prop_assert_eq!(done.len(), items.len());
            prop_assert_eq!(&bench.tree.listing(), &before, "a dry run changed the tree");
            prop_assert!(bench.trash.moved().is_empty(), "a dry run moved something to the Trash");
            prop_assert!(bench.runner.calls().is_empty(), "a dry run ran a tool");
            for (item, done) in items.iter().zip(&done) {
                prop_assert!(!acted(&done.outcome), "a dry run acted on {:?}: {:?}", item.action(), done.outcome);
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// The permanent removal acts beneath held folders: `scan/a` swapped for a link to the outside folder, which
        /// holds a twin of everything in it, and back all through the execution never lets anything behind the link
        /// go.
        #[test]
        fn a_folder_swapped_for_a_link_during_a_permanent_cleanup_lets_nothing_behind_it_go(
            entries in entries(),
            picks in picks(),
        ) {
            let bench = Bench::new(Tree::build(&entries));
            twin(&bench.tree.at("scan/a"), &bench.tree.at(OUTSIDE));
            let items = plan(&bench, &picks);
            let before = bench.tree.listing();
            let plan = bench.confirm(items.clone(), Method::Permanent);

            let swapper = Swapper::start(&bench.tree.at("scan/a"), &bench.tree.at(OUTSIDE)).unwrap();
            let done = bench.run(plan);
            drop(swapper);

            holds(&bench, &items, &before, &done)?;
        }
    }
}

mod positive {
    use fresher_core::{Action, ItemOutcome, Method, SkipReason};

    use crate::model::{BASES, Bench, Leaf, SHARED_TEMP, TEMPORARY_ITEMS, Tree, USER_TEMP, item};

    #[test]
    fn the_bench_removes_what_may_go() {
        let bench = Bench::new(Tree::build(&[(1, vec![0], Leaf::File), (6, vec![1], Leaf::Folder)]));
        let items = vec![
            item(1, Action::Remove(bench.tree.at("scan/a/x/f"))),
            item(2, Action::Empty(bench.tree.at("scan/home/y"))),
            item(3, Action::Remove(bench.tree.at("scan/b/sys/local"))),
        ];

        let done = bench.execute(items, Method::Permanent);

        assert_eq!(done[0].outcome, ItemOutcome::Deleted);
        assert_eq!(done[1].outcome, ItemOutcome::Emptied { moved: 1, skipped: 0 });
        assert!(matches!(done[2].outcome, ItemOutcome::Skipped { .. }));
        assert!(!bench.tree.at("scan/a/x/f").exists());
        assert!(bench.tree.at("scan/home/y").is_dir());
    }

    /// The dry-run bench reaches the act: what the sandbox bench removes ends `DryRun`, the tool too.
    #[test]
    #[cfg(debug_assertions)]
    fn the_dry_run_bench_ends_what_may_go_dry_run() {
        let bench = Bench::dry_run(Tree::build(&[(1, vec![0], Leaf::File), (6, vec![1], Leaf::Folder)]));
        let items = vec![
            item(1, Action::Remove(bench.tree.at("scan/a/x/f"))),
            item(2, Action::Empty(bench.tree.at("scan/home/y"))),
            item(3, Action::Remove(bench.tree.at("scan/b/sys/local"))),
            item(4, crate::model::tool()),
        ];

        let done = bench.execute(items, Method::Permanent);

        let dry_run = ItemOutcome::Skipped { reason: SkipReason::DryRun };
        assert_eq!(done[0].outcome, dry_run);
        assert_eq!(done[1].outcome, dry_run);
        assert!(matches!(done[2].outcome, ItemOutcome::Skipped { .. }) && done[2].outcome != dry_run);
        assert_eq!(done[3].outcome, dry_run);
        assert!(bench.tree.at("scan/a/x/f").exists() && bench.tree.at("scan/home/y/d").is_dir());
    }

    #[test]
    fn the_bench_removes_items_inside_the_temporary_folders_and_nothing_of_the_systems() {
        let base = |place: &str| BASES.iter().position(|base| *base == place).unwrap();
        let entries = [
            (base(USER_TEMP), vec![0], Leaf::File),
            (base(SHARED_TEMP), vec![1], Leaf::File),
            (base(TEMPORARY_ITEMS), vec![2], Leaf::File),
        ];
        let bench = Bench::new(Tree::build(&entries));
        let items = vec![
            item(1, Action::Remove(bench.tree.at("scan/private/var/folders/xx/me/T/x"))),
            item(2, Action::Remove(bench.tree.at("scan/private/tmp/y"))),
            item(3, Action::Remove(bench.tree.at("scan/private/var/folders/xx/me/T/TemporaryItems/z"))),
            item(4, Action::Remove(bench.tree.at("scan/private/var/folders/xx/me/T"))),
            item(5, Action::Empty(bench.tree.at("scan/private/tmp"))),
        ];

        let done = bench.execute(items, Method::Permanent);

        assert_eq!(done[0].outcome, ItemOutcome::Deleted);
        assert_eq!(done[1].outcome, ItemOutcome::Deleted);
        for skipped in &done[2..4] {
            assert!(matches!(skipped.outcome, ItemOutcome::Skipped { .. }), "{:?}", skipped.outcome);
        }
        // The shared folder is whole: it is not emptied as one item either.
        assert_eq!(done[4].outcome, ItemOutcome::Skipped { reason: SkipReason::WholeFolder });
        assert!(bench.tree.at("scan/private/var/folders/xx/me/T/TemporaryItems/z/f").is_file());
        assert!(bench.tree.at("scan/private/tmp").is_dir());
    }
}
