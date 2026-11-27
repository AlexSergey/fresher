use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fresher_core::Protection;
use fresher_core::{
    ByteSize, CategoryId, ChildOrder, DangerLevel, DiskState, ItemRule, LargestPage, LargestQuery, LeftAlone, ModuleId,
    NodeId, NodeInfo, NodeKind, NodePage, OwnPlace, Sizes, StateId, Timestamp, Unchanged, UnreadablePlace,
};
use fresher_testing::FixedDiskState;
use serde_json::json;

use crate::Engine;
use crate::testing::{Fixture, MemorySettings, Protecting, ScriptedWalk, TestModule, scan};

const MB: u64 = 1_000_000;
const DAY: u64 = 86_400;
/// When the fixture's state was taken.
const TAKEN: u64 = 1_000 * DAY;

fn caches() -> CategoryId {
    CategoryId::new(ModuleId::new("alpha").unwrap(), "alpha.caches").unwrap()
}

fn mb(size: u64) -> Sizes {
    Sizes::complete(ByteSize::new(size * MB), ByteSize::new(size * MB))
}

/// A state `id` of the folders and files given, each of its size in MB.
fn state(id: u32, nodes: &[(&str, NodeKind, u64)]) -> FixedDiskState {
    let nodes = nodes.iter().map(|&(path, kind, size)| (PathBuf::from(path), kind, mb(size)));
    FixedDiskState::new(StateId::new(id), Timestamp::from_secs(TAKEN), nodes)
}

/// The home `/home` as state `id`: its `Library`, Trash, a hidden folder and an app, each holding a large item; a
/// claimed `Caches` and a claimed file; Downloads with an old movie, a recent archive beside its folder and a small
/// file; a photo library; a chain of project folders; a folder holding a claimed one; and an image directly in the
/// home.
fn home(id: u32) -> FixedDiskState {
    state(
        id,
        &[
            ("/home", NodeKind::Folder, 3_000),
            ("/home/Library", NodeKind::Folder, 500),
            ("/home/Library/big.bin", NodeKind::File, 400),
            ("/home/.Trash", NodeKind::Folder, 300),
            ("/home/.Trash/old.dmg", NodeKind::File, 300),
            ("/home/.cache", NodeKind::Folder, 200),
            ("/home/.cache/blob.bin", NodeKind::File, 200),
            ("/home/Applications", NodeKind::Folder, 150),
            ("/home/Applications/Tool.app", NodeKind::Folder, 150),
            ("/home/Caches", NodeKind::Folder, 250),
            ("/home/Caches/blob.bin", NodeKind::File, 250),
            ("/home/Caches/inner", NodeKind::Folder, 100),
            ("/home/Downloads", NodeKind::Folder, 700),
            ("/home/Downloads/movie.mov", NodeKind::File, 300),
            ("/home/Downloads/claimed.dmg", NodeKind::File, 200),
            ("/home/Downloads/Foo.zip", NodeKind::File, 120),
            ("/home/Downloads/Foo", NodeKind::Folder, 50),
            ("/home/Downloads/small.bin", NodeKind::File, 20),
            ("/home/Pictures", NodeKind::Folder, 400),
            ("/home/Pictures/Trip.photoslibrary", NodeKind::Folder, 400),
            ("/home/Pictures/Trip.photoslibrary/master.heic", NodeKind::File, 390),
            ("/home/Pictures/Trip.photoslibrary/sub", NodeKind::Folder, 5),
            ("/home/Projects", NodeKind::Folder, 160),
            ("/home/Projects/old", NodeKind::Folder, 150),
            ("/home/Projects/old/build.tar", NodeKind::File, 150),
            ("/home/Work", NodeKind::Folder, 130),
            ("/home/Work/node_modules", NodeKind::Folder, 130),
            ("/home/top.iso", NodeKind::File, 110),
        ],
    )
    .with_owner(Path::new("/home/Caches"), caches())
    .with_owner(Path::new("/home/Downloads/claimed.dmg"), caches())
    .with_owner(Path::new("/home/Work/node_modules"), caches())
    .with_modified(Path::new("/home/Downloads/movie.mov"), Timestamp::from_secs(TAKEN - 400 * DAY))
    .with_modified(Path::new("/home/Downloads/Foo.zip"), Timestamp::from_secs(TAKEN - 10 * DAY))
}

/// A state of `/home` with `count` files of 11 MB directly in it.
fn many(count: usize) -> FixedDiskState {
    let total = mb(11 * u64::try_from(count).unwrap());
    let files = (0..count).map(|position| (PathBuf::from(format!("/home/{position:05}.bin")), NodeKind::File, mb(11)));
    let nodes = std::iter::once((PathBuf::from("/home"), NodeKind::Folder, total)).chain(files);
    FixedDiskState::new(StateId::new(1), Timestamp::from_secs(TAKEN), nodes)
}

/// An engine with the module `alpha` (its category `alpha.caches`, finding nothing) that has scanned once and holds
/// `state`, with `settings` stored.
fn scanned_with(state: Arc<dyn DiskState>, settings: MemorySettings) -> Engine {
    let alpha = TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe);
    let walk = ScriptedWalk::new().committing(vec![state]);
    let engine = Fixture::new(walk).module(alpha).settings(settings).engine();
    scan(&engine);
    engine
}

fn scanned(state: FixedDiskState) -> Engine {
    scanned_with(Arc::new(state), MemorySettings::default())
}

/// `/home/Projects` holding `old/build.tar` and `other/big.iso`, 150 MB each.
fn projects() -> FixedDiskState {
    state(
        1,
        &[
            ("/home", NodeKind::Folder, 300),
            ("/home/Projects", NodeKind::Folder, 300),
            ("/home/Projects/old", NodeKind::Folder, 150),
            ("/home/Projects/old/build.tar", NodeKind::File, 150),
            ("/home/Projects/other", NodeKind::Folder, 150),
            ("/home/Projects/other/big.iso", NodeKind::File, 150),
        ],
    )
}

/// `/home/Projects/old/build.tar` ignored.
fn ignoring_build() -> MemorySettings {
    MemorySettings::with(&[("core.ignore.paths", json!(["/home/Projects/old/build.tar"]))])
}

/// `/home/Vault` holding a large file beside a small folder `keep`.
fn vault() -> FixedDiskState {
    state(
        1,
        &[
            ("/home", NodeKind::Folder, 300),
            ("/home/Vault", NodeKind::Folder, 300),
            ("/home/Vault/big.bin", NodeKind::File, 290),
            ("/home/Vault/keep", NodeKind::Folder, 10),
        ],
    )
}

/// An engine that has scanned `state` once and whose cleaner protects `never`, its contents and its ancestors.
fn scanned_protecting(state: FixedDiskState, never: &str) -> Engine {
    let protection = Protection::new(vec![PathBuf::from(never)], Vec::new(), Vec::new()).unwrap();
    let cleaner = Arc::new(Protecting::new(protection));
    let walk = ScriptedWalk::new().committing(vec![Arc::new(state)]);
    let engine = Fixture::new(walk).cleaner(cleaner).engine();
    scan(&engine);
    engine
}

/// The category `alpha.caches` ignored.
fn ignoring_caches() -> MemorySettings {
    MemorySettings::with(&[("core.ignore.categories", json!(["alpha.caches"]))])
}

/// The smallest size listed set to `choice`.
fn min_size(choice: &str) -> MemorySettings {
    MemorySettings::with(&[("core.largest.min-size", json!(choice))])
}

/// Every row of the home, any kind, place and age.
const fn query() -> LargestQuery {
    LargestQuery { kind: None, place: None, unchanged: Unchanged::Any, offset: 0, limit: 100 }
}

fn names(page: &LargestPage) -> Vec<&str> {
    page.rows.iter().map(|row| row.node.name.as_str()).collect()
}

fn row_names(place: &OwnPlace) -> Vec<&str> {
    place.rows.iter().map(|row| row.node.name.as_str()).collect()
}

/// The node at `path`, below `/home`, in the engine's last state.
fn node(engine: &Engine, path: &str) -> NodeId {
    let mut at = engine.disk_roots().unwrap()[0].id;
    for name in Path::new(path).strip_prefix("/home").unwrap() {
        let page = engine.disk_children(at, ChildOrder::ByName, 0, 500).unwrap();
        at = page.nodes.into_iter().find(|child| *child.name == *name.to_string_lossy()).unwrap().id;
    }
    at
}

/// A state that counts its items queries.
struct Counting {
    inner: FixedDiskState,
    items: AtomicUsize,
}

impl Counting {
    fn new(inner: FixedDiskState) -> Arc<Self> {
        Arc::new(Self { inner, items: AtomicUsize::new(0) })
    }

    fn asked(&self) -> usize {
        self.items.load(Ordering::SeqCst)
    }
}

impl DiskState for Counting {
    fn id(&self) -> StateId {
        self.inner.id()
    }

    fn taken(&self) -> Timestamp {
        self.inner.taken()
    }

    fn roots(&self) -> Vec<NodeId> {
        self.inner.roots()
    }

    fn node(&self, id: NodeId) -> Option<NodeInfo> {
        self.inner.node(id)
    }

    fn children(&self, id: NodeId, order: ChildOrder, offset: u32, limit: u32) -> Option<NodePage> {
        self.inner.children(id, order, offset, limit)
    }

    fn largest_files(&self, limit: u32) -> Vec<NodeInfo> {
        self.inner.largest_files(limit)
    }

    fn search(&self, query: &str, limit: u32) -> Vec<NodeInfo> {
        self.inner.search(query, limit)
    }

    fn unreadable(&self, limit: u32) -> Vec<UnreadablePlace> {
        self.inner.unreadable(limit)
    }

    fn path(&self, id: NodeId) -> Option<PathBuf> {
        self.inner.path(id)
    }

    fn owned(&self) -> Vec<(NodeId, CategoryId)> {
        self.inner.owned()
    }

    fn find(&self, path: &Path) -> Option<NodeId> {
        self.inner.find(path)
    }

    fn items(&self, root: NodeId, rule: &dyn ItemRule, min: ByteSize, limit: u32) -> Option<Vec<NodeId>> {
        self.items.fetch_add(1, Ordering::SeqCst);
        self.inner.items(root, rule, min, limit)
    }

    fn left_alone(
        &self,
        root: NodeId,
        rule: &dyn ItemRule,
        min: ByteSize,
        before: Timestamp,
    ) -> Option<Vec<LeftAlone>> {
        self.inner.left_alone(root, rule, min, before)
    }
}

mod files {
    mod negative {
        use std::path::Path;
        use std::sync::Arc;
        use std::time::Duration;

        use fresher_core::{
            CommandError, Event, LargestQuery, NodeId, NodeKind, Offer, Operation, StateId, Unchanged, Withheld,
        };
        use serde_json::json;

        use super::super::{MB, caches, home, ignoring_caches, names, query, scanned, scanned_with, state};
        use crate::testing::{Fixture, MemorySettings, ScriptedWalk, engine, scan};

        #[test]
        fn no_scan_is_refused() {
            let engine = engine(ScriptedWalk::new());

            assert_eq!(engine.largest(&query()), Err(CommandError::NoScan));
            assert_eq!(engine.largest_place(None, 0, 10), Err(CommandError::NoScan));
        }

        #[test]
        fn a_running_scan_is_busy() {
            let (walk, gate) = ScriptedWalk::new().committing(vec![Arc::new(home(1))]).hold();
            gate.release();
            let engine = Fixture::new(walk).engine();
            scan(&engine);
            gate.close();
            let events = engine.subscribe();
            engine.start_scan(crate::ScanMode::Auto).unwrap();
            while !matches!(events.recv_timeout(Duration::from_secs(10)).unwrap(), Event::ScanStarted { .. }) {}

            let busy = CommandError::Busy { running: Operation::Scan };
            assert_eq!(engine.largest(&query()), Err(busy.clone()));
            assert_eq!(engine.largest_place(None, 0, 10), Err(busy));
            engine.cancel_scan().unwrap();
        }

        #[test]
        fn a_place_of_another_state_is_stale() {
            let engine = scanned(home(1));
            let other = NodeId::new(StateId::new(9), 1);

            assert_eq!(engine.largest(&LargestQuery { place: Some(other), ..query() }), Err(CommandError::StaleNode));
            assert_eq!(engine.largest_place(Some(other), 0, 10), Err(CommandError::StaleNode));
        }

        #[test]
        fn a_place_outside_the_home_is_refused() {
            let engine = scanned(state(1, &[("/home", NodeKind::Folder, 10), ("/elsewhere", NodeKind::Folder, 10)]));
            let roots = engine.disk_roots().unwrap();
            let (home, elsewhere) = (roots[0].id, roots[1].id);

            for place in [home, elsewhere] {
                let asked = LargestQuery { place: Some(place), ..query() };
                assert_eq!(engine.largest(&asked), Err(CommandError::NotOwnItem), "{place:?}");
            }
            assert_eq!(engine.largest_place(Some(elsewhere), 0, 10), Err(CommandError::NotOwnItem));
        }

        #[test]
        fn nothing_below_the_minimum_is_listed() {
            let page = scanned(home(1)).largest(&query()).unwrap();

            assert!(!names(&page).contains(&"small.bin"));
            assert_eq!(page.min_size.bytes(), 100 * MB);
        }

        #[test]
        fn a_claimed_file_is_not_listed() {
            let page = scanned(home(1)).largest(&query()).unwrap();

            for name in ["claimed.dmg", "Caches", "blob.bin", "node_modules"] {
                assert!(!names(&page).contains(&name), "{name}");
            }
        }

        #[test]
        fn nothing_under_the_library_or_the_trash_is_listed() {
            let page = scanned(home(1)).largest(&query()).unwrap();

            for name in ["big.bin", "old.dmg", "blob.bin", "Tool.app"] {
                assert!(!names(&page).contains(&name), "{name}");
            }
        }

        #[test]
        fn an_ignored_path_is_not_listed() {
            let settings = MemorySettings::with(&[("core.ignore.paths", json!(["/home/Downloads"]))]);

            let page = scanned_with(Arc::new(home(1)), settings).largest(&query()).unwrap();

            assert_eq!(names(&page), ["Trip.photoslibrary", "build.tar", "top.iso"]);
        }

        #[test]
        fn a_package_holding_a_file_of_an_ignored_category_is_withheld() {
            let state = home(1).with_owner(Path::new("/home/Pictures/Trip.photoslibrary/master.heic"), caches());

            let page = scanned_with(Arc::new(state), ignoring_caches()).largest(&query()).unwrap();

            assert_eq!(page.rows[0].node.name, "Trip.photoslibrary");
            assert_eq!(page.rows[0].offer, Offer::Withheld(Withheld::HoldsExplained));
        }

        #[test]
        fn a_file_changed_after_the_cutoff_is_filtered_out() {
            let engine = scanned(home(1));
            let unchanged_for = |unchanged| {
                names(&engine.largest(&LargestQuery { unchanged, ..query() }).unwrap())
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            };

            assert_eq!(unchanged_for(Unchanged::Month), ["movie.mov"]);
            assert_eq!(unchanged_for(Unchanged::Year), ["movie.mov"]);
            assert_eq!(unchanged_for(Unchanged::TwoYears), Vec::<String>::new());
        }

        #[test]
        fn a_scan_of_another_root_answers_an_empty_page() {
            let other = state(1, &[("/elsewhere", NodeKind::Folder, 500), ("/elsewhere/big.iso", NodeKind::File, 500)]);
            let engine = scanned(other);

            let page = engine.largest(&query()).unwrap();
            let place = engine.largest_place(None, 0, 10).unwrap();

            assert_eq!((page.rows.len(), page.total, page.truncated), (0, 0, false));
            assert_eq!((place.path.len(), place.rows.len(), place.total), (0, 0, 0));
        }
    }

    mod positive {
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{
            ItemKind, KindCount, LargestQuery, MAX_ITEMS, Offer, SettingUpdate, SettingValue, Withheld,
        };

        use super::super::{Counting, MB, caches, home, many, min_size, names, node, query, scanned, scanned_with};
        use crate::testing::MemorySettings;

        #[test]
        fn the_largest_come_first() {
            let page = scanned(home(1)).largest(&query()).unwrap();

            assert_eq!(names(&page), ["Trip.photoslibrary", "movie.mov", "build.tar", "Foo.zip", "top.iso"]);
            assert_eq!(page.total, 5);
            assert!(page.rows.iter().all(|row| row.offer == Offer::Offered && row.picked.is_none()));
        }

        #[test]
        fn a_package_is_one_row() {
            let page = scanned(home(1)).largest(&query()).unwrap();

            assert_eq!(page.rows[0].kind, ItemKind::MediaLibrary);
            assert!(!names(&page).contains(&"master.heic"));
        }

        #[test]
        fn the_place_is_written_from_the_home() {
            let page = scanned(home(1)).largest(&query()).unwrap();
            let places: Vec<&str> = page.rows.iter().map(|row| row.place.as_str()).collect();

            assert_eq!(places, ["~/Pictures", "~/Downloads", "~/Projects/old", "~/Downloads", "~"]);
        }

        #[test]
        fn the_kinds_and_places_count_the_other_filters_only() {
            let engine = scanned(home(1));
            let downloads = node(&engine, "/home/Downloads");

            let videos = engine.largest(&LargestQuery { kind: Some(ItemKind::Video), ..query() }).unwrap();
            let in_downloads = engine.largest(&LargestQuery { place: Some(downloads), ..query() }).unwrap();

            assert_eq!(names(&videos), ["movie.mov"]);
            let kinds: Vec<(ItemKind, u32)> = videos.kinds.iter().map(|count| (count.kind, count.count)).collect();
            assert_eq!(
                kinds,
                [(ItemKind::DiskImage, 1), (ItemKind::Archive, 2), (ItemKind::Video, 1), (ItemKind::MediaLibrary, 1)]
            );
            let places: Vec<(&str, u32)> =
                videos.places.iter().map(|count| (count.folder.name.as_str(), count.count)).collect();
            assert_eq!(places, [("Downloads", 1)]);
            assert_eq!(names(&in_downloads), ["movie.mov", "Foo.zip"]);
            assert_eq!(
                in_downloads.kinds,
                [KindCount { kind: ItemKind::Archive, count: 1 }, KindCount { kind: ItemKind::Video, count: 1 }]
            );
            let all: Vec<(&str, u32)> =
                in_downloads.places.iter().map(|count| (count.folder.name.as_str(), count.count)).collect();
            assert_eq!(all, [("Downloads", 2), ("Pictures", 1), ("Projects", 1)]);
        }

        #[test]
        fn the_sizes_sum_the_filtered_rows() {
            let engine = scanned(home(1));
            let downloads = node(&engine, "/home/Downloads");

            let page = engine.largest(&LargestQuery { place: Some(downloads), ..query() }).unwrap();

            assert_eq!(page.sizes.allocated.bytes(), 420 * MB);
            assert_eq!(page.total, 2);
        }

        #[test]
        fn an_archive_beside_its_folder_names_it() {
            let page = scanned(home(1)).largest(&query()).unwrap();
            let unpacked: Vec<(&str, Option<&str>)> =
                page.rows.iter().map(|row| (row.node.name.as_str(), row.unpacked.as_deref())).collect();

            assert!(unpacked.contains(&("Foo.zip", Some("Foo"))));
            assert!(unpacked.contains(&("build.tar", None)));
        }

        #[test]
        fn the_items_are_computed_once_per_state_and_minimum() {
            let state = Counting::new(home(1));
            let engine = scanned_with(Arc::clone(&state) as _, MemorySettings::default());

            engine.largest(&query()).unwrap();
            engine.largest(&LargestQuery { kind: Some(ItemKind::Video), ..query() }).unwrap();

            assert_eq!(state.asked(), 1);
        }

        #[test]
        fn a_changed_minimum_recomputes_them() {
            let state = Counting::new(home(1));
            let engine = scanned_with(Arc::clone(&state) as _, min_size("100-mb"));
            engine.largest(&query()).unwrap();

            let value = SettingValue::Choice("10-mb".into());
            engine.update_setting(SettingUpdate { key: "core.largest.min-size".into(), value }).unwrap();
            let page = engine.largest(&query()).unwrap();

            assert_eq!(state.asked(), 2);
            assert!(names(&page).contains(&"small.bin"));
            assert_eq!(page.min_size.bytes(), 10 * MB);
        }

        #[test]
        fn more_than_the_cap_says_truncated() {
            let cap = MAX_ITEMS as usize;

            let over = scanned_with(Arc::new(many(cap + 1)), min_size("10-mb")).largest(&query()).unwrap();
            let exact = scanned_with(Arc::new(many(cap)), min_size("10-mb")).largest(&query()).unwrap();

            assert_eq!((over.truncated, over.total as usize), (true, cap));
            assert_eq!((exact.truncated, exact.total as usize), (false, cap));
        }

        #[test]
        fn the_page_is_capped() {
            let engine = scanned_with(Arc::new(many(150)), min_size("10-mb"));

            let first = engine.largest(&LargestQuery { limit: 500, ..query() }).unwrap();
            let next = engine.largest(&LargestQuery { offset: 100, limit: 500, ..query() }).unwrap();

            assert_eq!((first.rows.len(), first.total), (100, 150));
            assert_eq!((next.rows.len(), next.offset), (50, 100));
        }

        #[test]
        fn a_package_holding_a_claimed_file_is_listed_withheld() {
            let state = home(1).with_owner(Path::new("/home/Pictures/Trip.photoslibrary/master.heic"), caches());

            let page = scanned(state).largest(&query()).unwrap();

            assert_eq!(page.rows[0].node.name, "Trip.photoslibrary");
            assert_eq!(page.rows[0].offer, Offer::Withheld(Withheld::HoldsExplained));
        }
    }
}

mod folders {
    mod negative {
        use std::sync::Arc;

        use fresher_core::{CommandError, Offer, Withheld};

        use super::super::{
            home, ignoring_build, ignoring_caches, node, projects, query, scanned, scanned_with, vault,
        };

        #[test]
        fn the_library_does_not_open() {
            let engine = scanned(home(1));

            for path in ["/home/Library", "/home/.Trash", "/home/.cache", "/home/Applications/Tool.app"] {
                let opened = engine.largest_place(Some(node(&engine, path)), 0, 10);

                assert_eq!(opened, Err(CommandError::NotOwnItem), "{path}");
            }
        }

        #[test]
        fn a_package_does_not_open() {
            let engine = scanned(home(1));
            let package = node(&engine, "/home/Pictures/Trip.photoslibrary");

            assert_eq!(engine.largest_place(Some(package), 0, 10), Err(CommandError::NotOwnItem));
        }

        #[test]
        fn a_claimed_folder_does_not_open() {
            let engine = scanned(home(1));
            let claimed = node(&engine, "/home/Caches");

            assert_eq!(engine.largest_place(Some(claimed), 0, 10), Err(CommandError::NotOwnItem));
        }

        #[test]
        fn a_folder_inside_a_package_does_not_open() {
            let engine = scanned(home(1));
            let inside = node(&engine, "/home/Pictures/Trip.photoslibrary/sub");

            assert_eq!(engine.largest_place(Some(inside), 0, 10), Err(CommandError::NotOwnItem));
        }

        #[test]
        fn a_folder_inside_a_claimed_folder_does_not_open_even_when_its_category_is_ignored() {
            for engine in [scanned(home(1)), scanned_with(Arc::new(home(1)), ignoring_caches())] {
                let inside = node(&engine, "/home/Caches/inner");

                assert_eq!(engine.largest_place(Some(inside), 0, 10), Err(CommandError::NotOwnItem));
            }
        }

        #[test]
        fn a_folder_holding_an_ignored_path_is_withheld_but_opens() {
            let engine = scanned_with(Arc::new(projects()), ignoring_build());

            let place = engine.largest_place(None, 0, 100).unwrap();
            let projects = place.rows.iter().find(|row| row.node.name == "Projects").unwrap();

            assert_eq!((projects.offer, projects.opens), (Offer::Withheld(Withheld::Ignored), true));
            let old = node(&engine, "/home/Projects/old");
            assert_eq!(engine.largest_place(Some(old), 0, 10).map(|place| place.total), Ok(1));
            let build = engine.largest_place(Some(old), 0, 10).unwrap().rows[0].clone();
            assert_eq!((build.offer, build.opens), (Offer::Withheld(Withheld::Ignored), false));
        }

        #[test]
        fn a_file_under_a_folder_that_does_not_open_is_not_offered_in_files() {
            let engine = super::super::scanned_protecting(vault(), "/home/Vault/keep");

            let page = engine.largest(&query()).unwrap();
            let vault = node(&engine, "/home/Vault");

            assert_eq!(page.rows.len(), 1);
            assert_eq!(page.rows[0].node.name, "big.bin");
            assert_eq!(page.rows[0].offer, Offer::Withheld(Withheld::Protected));
            assert_eq!(engine.largest_place(Some(vault), 0, 10), Err(CommandError::NotOwnItem));
        }

        #[test]
        fn a_file_does_not_open() {
            let engine = scanned(home(1));
            let file = node(&engine, "/home/top.iso");

            assert_eq!(engine.largest_place(Some(file), 0, 10), Err(CommandError::NotOwnItem));
        }

        #[test]
        fn a_browse_row_carries_no_pick() {
            let engine = scanned(home(1));
            let movie = engine.largest(&query()).unwrap().rows.into_iter().find(|row| row.node.name == "movie.mov");
            engine.pick_own_item(movie.unwrap().node.id).unwrap();

            let place = engine.largest_place(Some(node(&engine, "/home/Downloads")), 0, 10).unwrap();

            assert!(place.rows.iter().any(|row| row.node.name == "movie.mov"));
            for row in &place.rows {
                assert!(serde_json::to_value(row).unwrap().get("picked").is_none(), "{}", row.node.name);
            }
        }
    }

    mod positive {
        use std::sync::Arc;

        use fresher_core::{ItemKind, Offer, Withheld};

        use super::super::{MB, home, ignoring_build, node, projects, query, row_names, scanned, scanned_with};

        #[test]
        fn a_sibling_of_an_ignored_path_stays_offered_and_reachable() {
            let engine = scanned_with(Arc::new(projects()), ignoring_build());
            let projects = node(&engine, "/home/Projects");
            let other = node(&engine, "/home/Projects/other");

            let in_projects = engine.largest_place(Some(projects), 0, 10).unwrap();
            let in_other = engine.largest_place(Some(other), 0, 10).unwrap();
            let files = engine.largest(&query()).unwrap();

            let other_row = in_projects.rows.iter().find(|row| row.node.name == "other").unwrap();
            assert_eq!((other_row.offer, other_row.opens), (Offer::Offered, true));
            assert_eq!((in_other.rows[0].node.name.as_str(), in_other.rows[0].offer), ("big.iso", Offer::Offered));
            let listed: Vec<(&str, Offer)> = files.rows.iter().map(|row| (row.node.name.as_str(), row.offer)).collect();
            assert_eq!(listed, [("big.iso", Offer::Offered)]);
        }

        #[test]
        fn the_home_opens_with_no_folder_given() {
            let engine = scanned(home(1));

            let place = engine.largest_place(None, 0, 100).unwrap();

            let way: Vec<&str> = place.path.iter().map(|node| node.name.as_str()).collect();
            assert_eq!(way, ["home"]);
            assert_eq!(
                row_names(&place),
                [
                    "Downloads",
                    "Library",
                    "Pictures",
                    ".Trash",
                    "Caches",
                    ".cache",
                    "Projects",
                    "Applications",
                    "Work",
                    "top.iso"
                ]
            );
            assert_eq!(place.total, 10);
        }

        #[test]
        fn the_way_back_starts_at_the_home() {
            let engine = scanned(home(1));
            let old = node(&engine, "/home/Projects/old");

            let place = engine.largest_place(Some(old), 0, 10).unwrap();

            let way: Vec<&str> = place.path.iter().map(|node| node.name.as_str()).collect();
            assert_eq!(way, ["home", "Projects", "old"]);
            assert_eq!(row_names(&place), ["build.tar"]);
            assert_eq!(place.rows[0].kind, ItemKind::Archive);
        }

        #[test]
        fn rows_carry_their_explained_part() {
            let engine = scanned(home(1));

            let place = engine.largest_place(None, 0, 100).unwrap();
            let row = |name: &str| place.rows.iter().find(|row| row.node.name == name).unwrap();

            assert_eq!(row("Caches").explained.allocated.bytes(), 250 * MB);
            assert_eq!((row("Caches").offer, row("Caches").opens), (Offer::Withheld(Withheld::Explained), false));
            assert_eq!(row("Work").explained.allocated.bytes(), 130 * MB);
            assert_eq!((row("Work").offer, row("Work").opens), (Offer::Withheld(Withheld::HoldsExplained), true));
            assert_eq!((row("Library").offer, row("Library").opens), (Offer::Withheld(Withheld::Library), false));
            assert_eq!(
                (row("Pictures").offer, row("Pictures").opens, row("Pictures").kind),
                (Offer::Offered, true, ItemKind::Folder)
            );
            assert_eq!(row("Pictures").explained.allocated.bytes(), 0);
        }

        #[test]
        fn a_folder_holding_explained_bytes_opens_to_its_children() {
            let engine = scanned(home(1));
            let work = node(&engine, "/home/Work");

            let place = engine.largest_place(Some(work), 0, 10).unwrap();

            assert_eq!(row_names(&place), ["node_modules"]);
            assert_eq!(place.rows[0].offer, Offer::Withheld(Withheld::Explained));
        }
    }
}
