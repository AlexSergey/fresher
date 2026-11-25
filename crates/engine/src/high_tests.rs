//! Tests of the High switch: withheld, no High finding leaves the engine and every way to remove one is refused.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fresher_core::{
    ByteSize, CancelToken, CategoryId, Cleaner, CleanupContext, ConfirmedPlan, DangerLevel, Depth, DiskSpace, Ended,
    Finding, FindingId, FindingOrder, FindingsQuery, HighLevel, ItemDone, ModuleId, NodeKind, PartKind, PlanItem,
    PutBackOutcome, Rehearsal, Removability, StateId, StorageMap, Timestamp, TrashState, TrashedItem, VolumeFacts,
    VolumeId,
};
use fresher_testing::{FakeCleaner, FakeVolumes, FixedDiskState};

use crate::Engine;
use crate::cleanup::{Planned, Purpose, rehearse_and_keep};
use crate::testing::{Fixture, ScriptedWalk, TestModule, TestModuleBuilder, bytes, category, finding, scan};

/// The ids of the world's findings, in the order the module gives them.
const CACHES: u32 = 1;
const DATA: u32 = 2;
const LOGS: u32 = 3;
const RAISED: u32 = 4;

/// Category `category` of module `alpha`.
fn id(category: &'static str) -> CategoryId {
    CategoryId::new(ModuleId::new("alpha").unwrap(), category).unwrap()
}

/// Module `alpha`: `alpha.caches` (Safe, `/home/Caches`, 100 bytes), `alpha.data` (High, `/home/Data`, 200),
/// `alpha.logs` (Low, `/home/Logs`, 30, and `/home/Raised`, 40, raised to High); its choice `all` selects the caches
/// and the data, `only-data` the data alone.
fn alpha() -> TestModuleBuilder {
    TestModule::builder("alpha")
        .category("alpha.caches", DangerLevel::Safe)
        .category("alpha.data", DangerLevel::High)
        .category("alpha.logs", DangerLevel::Low)
        .choice("all", &["alpha.caches", "alpha.data"])
        .choice("only-data", &["alpha.data"])
        .folder("Caches")
        .folder("Data")
        .folder("Logs")
        .folder("Raised")
        .quiet_on_match()
        .on_finish(|_, out| {
            let (caches, data) =
                (category("alpha.caches", DangerLevel::Safe), category("alpha.data", DangerLevel::High));
            let logs = category("alpha.logs", DangerLevel::Low);
            out.push(finding(&caches, "/home/Caches", bytes(100)));
            out.push(finding(&data, "/home/Data", bytes(200)));
            out.push(finding(&logs, "/home/Logs", bytes(30)));
            let mut raised = finding(&logs, "/home/Raised", bytes(40));
            raised.raise_danger(DangerLevel::High);
            out.push(raised);
            Ok(())
        })
}

/// The home `/home` (400 bytes) the scan commits: each finding's folder owned by its category, and `Movies` (30).
fn home() -> FixedDiskState {
    let folders = [
        ("/home", 400),
        ("/home/Caches", 100),
        ("/home/Data", 200),
        ("/home/Logs", 30),
        ("/home/Raised", 40),
        ("/home/Movies", 30),
    ];
    let nodes = folders.map(|(path, size)| (PathBuf::from(path), NodeKind::Folder, bytes(size)));
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(0), nodes)
        .with_owner(Path::new("/home/Caches"), id("alpha.caches"))
        .with_owner(Path::new("/home/Data"), id("alpha.data"))
        .with_owner(Path::new("/home/Logs"), id("alpha.logs"))
        .with_owner(Path::new("/home/Raised"), id("alpha.logs"))
}

/// The start-up disk: 1000 bytes, 400 used, 500 free.
fn startup() -> FakeVolumes {
    let facts = VolumeFacts { id: VolumeId::new(1), device: 1, case_sensitive: false, used: ByteSize::new(1) };
    let space = FakeVolumes::disk("Macintosh HD", ByteSize::new(1000), ByteSize::new(400), ByteSize::new(500));
    FakeVolumes::default().with("/", facts).with_space("/", DiskSpace { mount_point: PathBuf::from("/"), ..space })
}

/// The world of `alpha` over `home`, the engine built with `high`.
fn world(high: HighLevel) -> Fixture {
    let walk = ScriptedWalk::new()
        .folder("Caches", "/home/Caches", bytes(100))
        .folder("Data", "/home/Data", bytes(200))
        .folder("Logs", "/home/Logs", bytes(30))
        .folder("Raised", "/home/Raised", bytes(40))
        .committing(vec![Arc::new(home())]);
    Fixture::new(walk).module(alpha()).volumes(startup()).high(high)
}

fn scanned(fixture: Fixture) -> Engine {
    let engine = fixture.engine();
    scan(&engine);
    engine
}

/// The ids of every finding the engine answers on one page.
fn page(engine: &Engine, category: Option<&str>) -> Vec<u32> {
    let query = FindingsQuery::new(None, category.map(Into::into), FindingOrder::Found, 0, 100);
    engine.findings(&query).unwrap().findings.iter().map(|finding| finding.id().get()).collect()
}

/// Each category total as (category, count, allocated bytes, highest level).
fn totals(engine: &Engine) -> Vec<(String, u32, u64, DangerLevel)> {
    let overview = engine.overview().unwrap();
    let line = |total: &fresher_core::CategoryTotal| {
        (total.category.as_str().to_owned(), total.count, total.sizes.allocated.bytes(), total.highest_danger)
    };
    overview.totals.iter().map(line).collect()
}

/// The allocated bytes of the part `kind` of the only volume of `map`.
fn part(map: &StorageMap, kind: fn(&PartKind) -> bool) -> u64 {
    map.volumes[0].parts().iter().filter(|part| kind(&part.kind)).map(|part| part.sizes.allocated.bytes()).sum()
}

fn explained(map: &StorageMap) -> u64 {
    part(map, |kind| matches!(kind, PartKind::Explained))
}

fn your_folders(map: &StorageMap) -> u64 {
    part(map, |kind| matches!(kind, PartKind::YourFolders))
}

/// A High finding the engine never made, as an uninstall's tail receives one.
fn high_finding() -> Finding {
    let new = finding(&category("alpha.data", DangerLevel::High), "/home/Data", bytes(200));
    Finding::new(FindingId::new(99), new, Removability::Removable)
}

/// Drives the tail every plan is made by with `selected`, as the uninstall does.
fn tail(engine: &Engine, selected: &[&Finding]) -> Result<fresher_core::CleanupPlanView, fresher_core::CommandError> {
    let shared = &engine.shared;
    let deaths = shared.pending.lock().unwrap().kill();
    let scan = engine.overview().unwrap().last_scan.unwrap().scan;
    let planned = Planned { deaths, scan, roots: vec![PathBuf::from("/home")], purpose: Purpose::Selection };
    rehearse_and_keep(shared, planned, selected)
}

fn has_pending(engine: &Engine) -> bool {
    engine.shared.pending.lock().unwrap().pending().is_some()
}

/// The fake cleaner, counting its rehearsals.
#[derive(Debug, Default)]
struct Counting {
    inner: FakeCleaner,
    rehearsals: AtomicUsize,
}

impl Cleaner for Counting {
    fn rehearse(&self, roots: &[PathBuf], items: &[PlanItem], cx: &CleanupContext<'_>, depth: Depth) -> Vec<Rehearsal> {
        self.rehearsals.fetch_add(1, Ordering::Relaxed);
        self.inner.rehearse(roots, items, cx, depth)
    }

    fn execute(
        &self,
        plan: ConfirmedPlan,
        cx: &CleanupContext<'_>,
        cancel: &CancelToken,
        out: &mut dyn FnMut(ItemDone),
    ) -> Ended {
        self.inner.execute(plan, cx, cancel, out)
    }

    fn protection(&self) -> fresher_core::Protection {
        self.inner.protection()
    }

    fn locate(&self, items: &[TrashedItem]) -> Vec<TrashState> {
        self.inner.locate(items)
    }

    fn put_back(&self, item: &TrashedItem, cx: &CleanupContext<'_>) -> PutBackOutcome {
        self.inner.put_back(item, cx)
    }
}

/// The danger of each part of each insight in the `ModuleFinished` of `events`.
fn insight_parts(events: &[fresher_core::Event]) -> Vec<DangerLevel> {
    let insights = events.iter().find_map(|event| match event {
        fresher_core::Event::ModuleFinished { insights, .. } => Some(insights.clone()),
        _ => None,
    });
    insights.unwrap().iter().flat_map(|insight| insight.parts().iter().map(|part| part.danger)).collect()
}

/// Module `alpha` describing one insight with a Safe part and a High one.
fn describing() -> TestModuleBuilder {
    let part = |danger: DangerLevel| fresher_core::InsightPart {
        label: crate::testing::text("alpha.insight.part"),
        size: ByteSize::new(10),
        detail: crate::testing::text("alpha.insight.detail"),
        effect: crate::testing::text("alpha.insight.effect"),
        danger,
        command: None,
    };
    TestModule::builder("alpha").on_finish(move |_, out| {
        let parts = vec![part(DangerLevel::Safe), part(DangerLevel::High)];
        let title = crate::testing::text("alpha.insight.title");
        let summary = crate::testing::text("alpha.insight.summary");
        out.describe(fresher_core::Insight::new("volumes", title, summary, vec![], parts, vec![]).unwrap());
        Ok(())
    })
}

/// The engine over one app, the editor, under `high`: its bundle, its caches and its files in Application Support.
fn editor(high: HighLevel) -> Engine {
    use crate::apps::fixture::{app, facts, fixture, idle, state};
    let editor = app("com.example.Editor", "Editor");
    let apps =
        fresher_testing::FakeInstalledApps::new(vec![editor.clone()]).with_bundle(editor.path, Ok(facts(100, None)));
    let home = state(
        1,
        &[
            ("/home/Library/Caches/com.example.Editor", NodeKind::Folder, 10),
            ("/home/Library/Application Support/com.example.Editor", NodeKind::Folder, 20),
        ],
    );
    scanned(fixture(vec![home], Arc::new(apps), idle()).high(high))
}

/// The kinds of the editor's page under `high`, each with its allocated bytes.
fn page_kinds(high: HighLevel) -> Vec<(fresher_core::FootprintKind, u64)> {
    let page = editor(high).app(fresher_core::AppId::new(StateId::new(1), 0)).unwrap();
    page.items.iter().map(|item| (item.kind, item.sizes.allocated.bytes())).collect()
}

/// The editor's row of the Apps list under `high`: its parts with their allocated bytes, and its total.
fn list_row(high: HighLevel) -> (Vec<(fresher_core::FootprintKind, u64)>, u64) {
    let view = editor(high).apps(fresher_core::AppsQuery::default()).unwrap();
    let row = view.apps.first().unwrap();
    (row.parts.iter().map(|part| (part.kind, part.sizes.allocated.bytes())).collect(), row.sizes.allocated.bytes())
}

mod negative {
    use std::path::Path;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;

    use fresher_core::{
        AppId, CommandError, DangerLevel, Event, HighLevel, LeftOutItem, StateId, TextKey, UninstallChoice,
    };
    use fresher_testing::FakeInstalledApps;

    use super::{
        CACHES, Counting, DATA, LOGS, RAISED, describing, explained, has_pending, high_finding, id, insight_parts,
        list_row, page, page_kinds, scanned, tail, totals, world, your_folders,
    };
    use crate::apps::fixture::{app, facts, fixture, idle, state};
    use crate::cleanup::testing::{items, select};
    use crate::testing::{Fixture, ScriptedWalk, batches, bytes, scan};

    #[test]
    fn a_high_finding_is_in_no_page_total_or_found_size() {
        let engine = world(HighLevel::Withheld).engine();

        let events = scan(&engine);

        assert_eq!(page(&engine, None), [CACHES, LOGS]);
        assert_eq!(page(&engine, Some("alpha.data")), Vec::<u32>::new());
        assert_eq!(
            totals(&engine),
            [("alpha.caches".into(), 1, 100, DangerLevel::Safe), ("alpha.logs".into(), 1, 30, DangerLevel::Low)]
        );
        assert_eq!(engine.overview().unwrap().found.allocated.bytes(), 130);
        let published: Vec<u32> = batches(&events).iter().flatten().map(|finding| finding.id().get()).collect();
        assert_eq!(published, [CACHES, LOGS]);
    }

    #[test]
    fn a_finding_raised_to_high_is_left_out() {
        let engine = scanned(world(HighLevel::Withheld));

        assert_eq!(page(&engine, Some("alpha.logs")), [LOGS]);
        assert_eq!(totals(&engine)[1], ("alpha.logs".into(), 1, 30, DangerLevel::Low));
    }

    #[test]
    fn a_withheld_finding_is_unknown_by_id() {
        let engine = scanned(world(HighLevel::Withheld));

        for withheld in [DATA, RAISED] {
            let id = fresher_core::FindingId::new(withheld);
            assert_eq!(engine.finding(id), Err(CommandError::UnknownFinding { id }));
        }
        assert!(engine.finding(fresher_core::FindingId::new(CACHES)).is_ok());
    }

    #[test]
    fn the_map_counts_no_high_owner_as_can_be_freed() {
        let engine = scanned(world(HighLevel::Withheld));

        let map = engine.storage_map().unwrap();

        // Caches and Logs only: Data and Raised are the user's folders, with Movies.
        assert_eq!(explained(&map), 130);
        assert_eq!(your_folders(&map), 270);
        let explained_under = engine.shared.results.lock().unwrap().explained_under(&[Path::new("/home").into()]);
        assert_eq!(explained_under[0].allocated.bytes(), 130);
    }

    #[test]
    fn a_module_lists_no_high_category_and_no_empty_choice() {
        let engine = world(HighLevel::Withheld).engine();

        let modules = engine.modules();

        let categories: Vec<_> = modules[0].categories.iter().map(|info| info.descriptor.id()).collect();
        assert_eq!(categories, [id("alpha.caches"), id("alpha.logs")]);
        let choices: Vec<_> =
            modules[0].choices.iter().map(|choice| (choice.name.as_str(), &choice.categories)).collect();
        assert_eq!(choices, [("all", &vec![id("alpha.caches")])]);
    }

    #[test]
    fn a_left_out_of_a_high_category_is_not_published() {
        let left_out = |category: &'static str, size: u64| LeftOutItem {
            category: id(category),
            reason: TextKey::new("alpha.left-out.open").unwrap(),
            sizes: bytes(size),
        };
        let items = [left_out("alpha.data", 5), left_out("alpha.caches", 3)];
        let module = crate::testing::TestModule::builder("alpha")
            .category("alpha.caches", DangerLevel::Safe)
            .category("alpha.data", DangerLevel::High)
            .problems(&["alpha.left-out.open"])
            .on_finish(move |_, out| {
                items.iter().cloned().for_each(|item| out.left_out(item));
                Ok(())
            });
        let engine = Fixture::new(ScriptedWalk::new()).module(module).high(HighLevel::Withheld).engine();

        let events = scan(&engine);

        let published: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                Event::ModuleFinished { left_out, .. } => Some(left_out.clone()),
                _ => None,
            })
            .flatten()
            .map(|count| (count.category.as_str().to_owned(), count.items))
            .collect();
        assert_eq!(published, [("alpha.caches".to_owned(), 1)]);
    }

    #[test]
    fn a_selection_of_a_high_category_finds_nothing_to_prepare() {
        let engine = scanned(world(HighLevel::Withheld));

        assert_eq!(engine.prepare_cleanup(&select(&[], &[("alpha.data", &[])])), Err(CommandError::NothingToClean));
        let id = fresher_core::FindingId::new(DATA);
        assert_eq!(engine.prepare_cleanup(&select(&[DATA], &[])), Err(CommandError::UnknownFinding { id }));
        let logs = engine.prepare_cleanup(&select(&[], &[("alpha.logs", &[])])).unwrap();
        assert_eq!(items(&logs), [LOGS]);
        assert_eq!(logs.summary.highest, DangerLevel::Low);
    }

    #[test]
    fn a_prepare_with_a_high_item_is_refused_and_keeps_no_plan() {
        let cleaner = Arc::new(Counting::default());
        let engine = scanned(world(HighLevel::Withheld).cleaner(Arc::clone(&cleaner)));
        let safe = engine.finding(fresher_core::FindingId::new(CACHES)).unwrap();
        let high = high_finding();

        assert_eq!(tail(&engine, &[&high]), Err(CommandError::HighWithheld));
        assert_eq!(tail(&engine, &[&safe, &high]), Err(CommandError::HighWithheld));

        assert!(!has_pending(&engine));
        assert_eq!(cleaner.rehearsals.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn a_largest_pick_is_refused() {
        let engine = scanned(world(HighLevel::Withheld));
        let state = engine.disk_state().unwrap().unwrap();
        let movies = state.find(Path::new("/home/Movies")).unwrap();

        assert_eq!(engine.pick_own_item(movies), Err(CommandError::HighWithheld));
        assert_eq!(engine.shared.results.lock().unwrap().own(), []);
    }

    #[test]
    fn an_uninstall_of_everything_is_refused_and_the_page_offers_keep_data_only() {
        const EDITOR: AppId = AppId::new(StateId::new(1), 0);
        let editor = app("com.example.Editor", "Editor");
        let apps = FakeInstalledApps::new(vec![editor.clone()]).with_bundle(editor.path, Ok(facts(100, None)));
        let home = state(
            1,
            &[
                ("/home/Library/Caches/com.example.Editor", fresher_core::NodeKind::Folder, 10),
                ("/home/Library/Application Support/com.example.Editor", fresher_core::NodeKind::Folder, 20),
            ],
        );
        let engine = scanned(fixture(vec![home], Arc::new(apps), idle()).high(HighLevel::Withheld));

        assert_eq!(engine.prepare_uninstall(EDITOR, UninstallChoice::Everything), Err(CommandError::HighWithheld));
        assert!(!has_pending(&engine));
        let choices: Vec<_> = engine.app(EDITOR).unwrap().choices.iter().map(|view| view.choice).collect();
        assert_eq!(choices, [UninstallChoice::KeepData]);
        let keep = engine.prepare_uninstall(EDITOR, UninstallChoice::KeepData).unwrap();
        assert_eq!(keep.summary.highest, DangerLevel::Medium);
    }

    #[test]
    fn the_refusal_comes_before_the_agreement_check() {
        let engine = scanned(world(HighLevel::Withheld).unaccepted());
        let safe = engine.finding(fresher_core::FindingId::new(CACHES)).unwrap();

        assert_eq!(tail(&engine, &[&high_finding()]), Err(CommandError::HighWithheld));
        assert!(matches!(tail(&engine, &[&safe]), Err(CommandError::CleaningBlocked { .. })));
    }
    #[test]
    fn an_insight_part_at_high_is_not_published() {
        let engine = Fixture::new(ScriptedWalk::new()).module(describing()).high(HighLevel::Withheld).engine();

        let events = scan(&engine);

        assert_eq!(insight_parts(&events), [DangerLevel::Safe]);
    }

    #[test]
    fn the_app_page_lists_no_footprint_item_of_a_high_kind() {
        let kinds: Vec<_> = page_kinds(HighLevel::Withheld).into_iter().map(|(kind, _)| kind).collect();

        assert_eq!(kinds, [fresher_core::FootprintKind::Bundle, fresher_core::FootprintKind::Caches]);
    }

    #[test]
    fn the_apps_list_shows_no_part_of_a_high_kind_and_leaves_its_bytes_out() {
        let (parts, total) = list_row(HighLevel::Withheld);

        assert!(parts.iter().all(|(kind, _)| kind.danger() != DangerLevel::High), "{parts:?}");
        assert_eq!(total, parts.iter().map(|(_, bytes)| bytes).sum::<u64>());
    }
}

mod positive {
    use fresher_core::{DangerLevel, HighLevel};

    use super::{
        CACHES, DATA, LOGS, RAISED, describing, explained, id, insight_parts, list_row, page, page_kinds, scanned,
        totals, world,
    };
    use crate::cleanup::testing::{items, select};
    use crate::testing::{Fixture, ScriptedWalk, TestModule, bytes, category, finding};

    #[test]
    fn offered_high_is_as_before() {
        let engine = scanned(world(HighLevel::Offered));

        assert_eq!(page(&engine, None), [CACHES, DATA, LOGS, RAISED]);
        assert_eq!(
            totals(&engine),
            [
                ("alpha.caches".into(), 1, 100, DangerLevel::Safe),
                ("alpha.data".into(), 1, 200, DangerLevel::High),
                ("alpha.logs".into(), 2, 70, DangerLevel::High),
            ]
        );
        assert_eq!(explained(&engine.storage_map().unwrap()), 370);
        let modules = engine.modules();
        assert_eq!(modules[0].categories.len(), 3);
        let choices: Vec<_> = modules[0].choices.iter().map(|choice| choice.name.as_str()).collect();
        assert_eq!(choices, ["all", "only-data"]);
        assert_eq!(modules[0].choices[0].categories, [id("alpha.caches"), id("alpha.data")]);
        let plan = engine.prepare_cleanup(&select(&[], &[("alpha.data", &[])])).unwrap();
        assert_eq!((items(&plan), plan.summary.highest), (vec![DATA], DangerLevel::High));
    }

    #[test]
    fn withheld_keeps_every_lower_level() {
        let lower = || {
            let module = TestModule::builder("beta")
                .category("beta.caches", DangerLevel::Safe)
                .category("beta.logs", DangerLevel::Low)
                .category("beta.files", DangerLevel::Medium)
                .choice("all", &["beta.caches", "beta.logs", "beta.files"])
                .folder("Caches")
                .folder("Logs")
                .folder("Files")
                .quiet_on_match()
                .on_finish(|_, out| {
                    out.push(finding(&category("beta.caches", DangerLevel::Safe), "/home/Caches", bytes(1)));
                    out.push(finding(&category("beta.logs", DangerLevel::Low), "/home/Logs", bytes(2)));
                    out.push(finding(&category("beta.files", DangerLevel::Medium), "/home/Files", bytes(4)));
                    Ok(())
                });
            let walk = ScriptedWalk::new()
                .folder("Caches", "/home/Caches", bytes(1))
                .folder("Logs", "/home/Logs", bytes(2))
                .folder("Files", "/home/Files", bytes(4));
            Fixture::new(walk).module(module)
        };
        let offered = scanned(lower().high(HighLevel::Offered));
        let withheld = scanned(lower().high(HighLevel::Withheld));

        assert_eq!(page(&withheld, None), [1, 2, 3]);
        assert_eq!(totals(&withheld), totals(&offered));
        assert_eq!(withheld.overview().unwrap().found, offered.overview().unwrap().found);
        assert_eq!(withheld.modules(), offered.modules());
        let selection = select(&[], &[("beta.caches", &[]), ("beta.logs", &[]), ("beta.files", &[])]);
        let (kept, as_before) =
            (withheld.prepare_cleanup(&selection).unwrap(), offered.prepare_cleanup(&selection).unwrap());
        assert_eq!((items(&kept), kept.summary), (items(&as_before), as_before.summary));
        assert_eq!(kept.summary.highest, DangerLevel::Medium);
    }

    #[test]
    fn the_engine_reports_its_switch() {
        for high in [HighLevel::Offered, HighLevel::Withheld] {
            let engine = Fixture::new(ScriptedWalk::new()).high(high).engine();

            assert_eq!(engine.high_level(), high);
        }
    }

    #[test]
    fn offered_publishes_every_insight_part_and_every_footprint_item() {
        let engine = Fixture::new(ScriptedWalk::new()).module(describing()).high(HighLevel::Offered).engine();

        let events = crate::testing::scan(&engine);

        assert_eq!(insight_parts(&events), [DangerLevel::Safe, DangerLevel::High]);
        let kinds = page_kinds(HighLevel::Offered);
        assert!(kinds.contains(&(fresher_core::FootprintKind::Support, 20)), "{kinds:?}");
        let withheld = page_kinds(HighLevel::Withheld);
        assert_eq!(
            kinds.into_iter().filter(|(kind, _)| kind.danger() != DangerLevel::High).collect::<Vec<_>>(),
            withheld
        );
    }

    #[test]
    fn offered_keeps_every_part_in_the_apps_list() {
        let (parts, total) = list_row(HighLevel::Offered);
        let (withheld, withheld_total) = list_row(HighLevel::Withheld);

        assert!(parts.contains(&(fresher_core::FootprintKind::Support, 20)), "{parts:?}");
        assert_eq!(total, withheld_total + 20);
        assert_eq!(
            parts.into_iter().filter(|(kind, _)| kind.danger() != DangerLevel::High).collect::<Vec<_>>(),
            withheld
        );
    }
}
