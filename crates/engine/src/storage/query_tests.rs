use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;
use std::time::Duration;

use fresher_core::{
    ByteSize, CategoryId, DangerLevel, DiskSpace, Event, MapPart, MeaningGroup, ModuleId, NodeKind, PartKind, StateId,
    StorageMap, Timestamp, VolumeFacts, VolumeId, VolumeMap,
};
use fresher_testing::{FakeVolumes, FixedDiskState};

use crate::batch::BATCH_LIMIT;
use crate::testing::{ScriptedWalk, TestModule, TestModuleBuilder, bytes};

/// How long a test waits for an event before it fails: a guard against a hang, never a timing assumption.
const PATIENCE: Duration = Duration::from_secs(10);

fn facts(id: u128) -> VolumeFacts {
    VolumeFacts { id: VolumeId::new(id), device: 1, case_sensitive: false, used: ByteSize::new(1) }
}

/// A volume `name` mounted at `mount`, of `capacity` bytes with `used` taken and `free` left.
fn disk(name: &str, mount: &str, capacity: u64, used: u64, free: u64) -> DiskSpace {
    let space = FakeVolumes::disk(name, ByteSize::new(capacity), ByteSize::new(used), ByteSize::new(free));
    DiskSpace { mount_point: PathBuf::from(mount), ..space }
}

/// The start-up disk: 1000 bytes, 400 used by the data volume, 500 free, so 100 are the disk's own.
fn startup() -> FakeVolumes {
    FakeVolumes::default().with("/", facts(1)).with_space("/", disk("Macintosh HD", "/", 1000, 400, 500))
}

/// The start-up disk and a second volume `Data` mounted at `/Volumes/Data`: 600 bytes, 250 used, 350 free.
fn two_volumes() -> FakeVolumes {
    startup().with("/Volumes/Data", facts(2)).with_space("/Volumes/Data", disk("Data", "/Volumes/Data", 600, 250, 350))
}

fn caches() -> CategoryId {
    CategoryId::new(ModuleId::new("alpha").unwrap(), "alpha.caches").unwrap()
}

/// A module whose every folder named `Caches` is one finding of `alpha.caches`.
fn alpha() -> TestModuleBuilder {
    TestModule::builder("alpha").category("alpha.caches", DangerLevel::Safe).folder("Caches")
}

/// The home `/home` (300 bytes) as state `id`: `Caches` (100, owned by `alpha.caches`) and `Movies` (150).
fn home(id: u32) -> FixedDiskState {
    state(id, &[("/home", 300), ("/home/Caches", 100), ("/home/Movies", 150)])
        .with_owner(Path::new("/home/Caches"), caches())
}

/// A state of folders of the sizes given, as state `id`.
fn state(id: u32, folders: &[(&str, u64)]) -> FixedDiskState {
    let nodes = folders.iter().map(|&(path, size)| (PathBuf::from(path), NodeKind::Folder, bytes(size)));
    FixedDiskState::new(StateId::new(id), Timestamp::from_secs(0), nodes)
}

/// A walk that finds `/home/Caches` (100 bytes) and commits `states` in turn.
fn walk(states: Vec<FixedDiskState>) -> ScriptedWalk {
    let states = states.into_iter().map(|state| std::sync::Arc::new(state) as _).collect();
    ScriptedWalk::new().folder("Caches", "/home/Caches", bytes(100)).committing(states)
}

/// `count` folders named `Caches` of one byte each, under `/home`, numbered from `from`.
fn many_caches(walk: ScriptedWalk, from: usize, count: usize) -> ScriptedWalk {
    (from..from + count).fold(walk, |walk, n| walk.folder("Caches", &format!("/home/{n}/Caches"), bytes(1)))
}

/// The parts of `map` as short names with their allocated bytes.
fn parts(map: &VolumeMap) -> Vec<(String, u64)> {
    map.parts().iter().map(|part| (name(&part.kind), part.sizes.allocated.bytes())).collect()
}

fn name(kind: &PartKind) -> String {
    match kind {
        PartKind::Explained => "explained".into(),
        PartKind::YourFolders => "your-folders".into(),
        PartKind::NotScannedYet => "not-scanned-yet".into(),
        PartKind::Remainder { outside_scan: true, .. } => "outside".into(),
        PartKind::Remainder { outside_scan: false, .. } => "not-measured".into(),
        PartKind::MacOs => "mac-os".into(),
        PartKind::SwapAndSleep => "swap".into(),
        PartKind::OtherVolume { name } => format!("volume {name}"),
        PartKind::DiskOwn => "disk-own".into(),
        PartKind::Free => "free".into(),
    }
}

fn owned(parts: &[(&str, u64)]) -> Vec<(String, u64)> {
    parts.iter().map(|&(name, bytes)| (name.to_owned(), bytes)).collect()
}

/// The allocated bytes of the part `kind` names in the only volume of `map`.
fn part(map: &StorageMap, kind: &str) -> u64 {
    parts(&map.volumes[0]).into_iter().find(|(name, _)| name == kind).map_or(0, |(_, bytes)| bytes)
}

/// Every leaf of the groups: an app's categories and data, a category, a place, a part.
fn leaves(groups: &[MeaningGroup]) -> u64 {
    let mut total = 0;
    for group in groups {
        total += match group {
            MeaningGroup::App { categories, data, .. } => {
                categories.iter().map(|share| share.sizes.allocated.bytes()).sum::<u64>()
                    + data.iter().map(|place| place.sizes.allocated.bytes()).sum::<u64>()
            }
            MeaningGroup::Category(share) => share.sizes.allocated.bytes(),
            MeaningGroup::Place(place) => place.sizes.allocated.bytes(),
            MeaningGroup::Part(MapPart { sizes, .. }) => sizes.allocated.bytes(),
        };
    }
    total
}

/// Waits for the next `FindingsAdded`: its findings are in the results by then.
fn next_batch(events: &Receiver<Event>) {
    loop {
        if matches!(events.recv_timeout(PATIENCE).unwrap(), Event::FindingsAdded { .. }) {
            return;
        }
    }
}

/// Waits for `ScanStarted`: the volumes are read by then.
fn started(events: &Receiver<Event>) {
    loop {
        if matches!(events.recv_timeout(PATIENCE).unwrap(), Event::ScanStarted { .. }) {
            return;
        }
    }
}

mod negative {
    use std::sync::Arc;

    use fresher_core::{CommandError, Descent, MapPhase, NodeId, Operation, SettingUpdate, SettingValue, StateId};
    use fresher_testing::FakeVolumes;
    use serde_json::json;

    use super::{
        BATCH_LIMIT, alpha, facts, home, many_caches, next_batch, owned, part, parts, started, startup, state,
        two_volumes, walk,
    };
    use crate::testing::{Fixture, MemorySettings, ScriptedWalk, scan, wait_for_finish};

    #[test]
    fn a_cancelled_scan_of_other_roots_changes_neither_the_map_nor_its_meaning() {
        let (walk, gate) = walk(vec![home(1)]).hold();
        gate.release();
        let engine = Fixture::new(walk).module(alpha()).volumes(two_volumes()).engine();
        scan(&engine);
        let (map, meaning) = (engine.storage_map().unwrap(), engine.storage_meaning(0).unwrap());
        let roots = SettingValue::Folders(vec!["/Volumes/Data".into()]);
        engine.update_setting(SettingUpdate { key: "core.scan.roots".into(), value: roots }).unwrap();
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        started(&events);

        engine.cancel_scan().unwrap();
        wait_for_finish(&events);

        assert_eq!(engine.storage_map().unwrap(), map);
        assert_eq!(engine.storage_meaning(0).unwrap(), meaning);
        assert_eq!(engine.storage_meaning(1), Err(CommandError::UnknownVolume));
    }

    #[test]
    fn no_place_is_opened_while_a_scan_runs() {
        let (walk, gate) = walk(vec![home(1)]).hold();
        gate.release();
        let engine = Fixture::new(walk).module(alpha()).volumes(startup()).engine();
        scan(&engine);
        let root = engine.storage_map().unwrap().volumes[0].roots()[0];
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        started(&events);

        assert_eq!(
            engine.storage_place(root, Descent::Through, 0, 10),
            Err(CommandError::Busy { running: Operation::Scan })
        );

        gate.release();
        wait_for_finish(&events);
        assert!(engine.storage_place(root, Descent::Through, 0, 10).is_err(), "the next state replaced it");
    }

    #[test]
    fn a_place_page_holds_at_most_500_rows() {
        let mut folders = vec![("/home".to_owned(), 501)];
        folders.extend((0..501).map(|n| (format!("/home/{n:03}"), 1)));
        let folders: Vec<(&str, u64)> = folders.iter().map(|(path, size)| (path.as_str(), *size)).collect();
        let wide = state(1, &folders);
        let engine = Fixture::new(walk(vec![wide])).volumes(startup()).engine();
        scan(&engine);
        let root = engine.storage_map().unwrap().volumes[0].roots()[0];

        let view = engine.storage_place(root, Descent::Through, 0, 10_000).unwrap();

        assert_eq!((view.rows.len(), view.total), (500, 501));
    }

    #[test]
    fn the_map_before_any_scan_is_no_scan_with_the_volume() {
        let engine = Fixture::new(ScriptedWalk::new()).volumes(startup()).engine();

        let map = engine.storage_map().unwrap();

        assert_eq!(map.phase, MapPhase::NoScan);
        assert_eq!(map.volumes.len(), 1);
        assert_eq!(map.volumes[0].name(), "Macintosh HD");
        assert_eq!(map.volumes[0].roots(), []);
        assert_eq!(
            parts(&map.volumes[0]),
            owned(&[
                ("explained", 0),
                ("not-scanned-yet", 400),
                ("mac-os", 0),
                ("swap", 0),
                ("disk-own", 100),
                ("free", 500)
            ])
        );
        assert_eq!(engine.storage_meaning(0), Err(CommandError::NoScan));
        assert_eq!(
            engine.storage_place(NodeId::new(StateId::new(1), 0), Descent::Through, 0, 10),
            Err(CommandError::NoScan)
        );
    }

    #[test]
    fn a_place_of_an_older_state_is_stale() {
        let engine = Fixture::new(walk(vec![home(1), home(2)])).module(alpha()).volumes(startup()).engine();
        scan(&engine);
        let old = engine.storage_map().unwrap().volumes[0].roots()[0];
        scan(&engine);

        assert_eq!(engine.storage_place(old, Descent::Through, 0, 10), Err(CommandError::StaleNode));
    }

    #[test]
    fn an_unknown_volume_is_refused() {
        let engine = Fixture::new(walk(vec![home(1)])).module(alpha()).volumes(startup()).engine();
        scan(&engine);

        assert_eq!(engine.storage_meaning(1), Err(CommandError::UnknownVolume));
        assert!(engine.storage_meaning(0).is_ok());
    }

    #[test]
    fn a_failed_volume_read_gives_the_scanned_parts_only() {
        let volumes = FakeVolumes::default().with("/", facts(1));
        let engine = Fixture::new(walk(vec![home(1)])).module(alpha()).volumes(volumes).engine();
        scan(&engine);

        let map = engine.storage_map().unwrap();

        assert_eq!(map.phase, MapPhase::Complete { state: StateId::new(1) });
        assert_eq!(parts(&map.volumes[0]), owned(&[("explained", 100), ("your-folders", 200)]));
        assert_eq!(map.volumes[0].capacity().bytes(), 300);
        assert_eq!(map.volumes[0].name(), "home");
    }

    #[test]
    fn ignored_findings_are_not_explained_while_scanning() {
        let settings = MemorySettings::with(&[("core.ignore.categories", json!(["alpha.caches"]))]);
        let (walk, gate) = many_caches(ScriptedWalk::new(), 0, BATCH_LIMIT).hold();
        let engine = Fixture::new(walk).module(alpha()).volumes(startup()).settings(settings).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        next_batch(&events);

        let map = engine.storage_map().unwrap();

        assert_eq!(map.phase, MapPhase::Scanning);
        assert_eq!(part(&map, "explained"), 0);
        assert_eq!(part(&map, "not-scanned-yet"), 400);
        gate.release();
        wait_for_finish(&events);
    }

    #[test]
    fn a_cancelled_scan_keeps_the_last_map() {
        let walk = walk(vec![home(1)]);
        let (walk, gate) = walk.hold();
        gate.release();
        let engine = Fixture::new(walk).module(alpha()).volumes(startup()).engine();
        scan(&engine);
        let before = engine.storage_map().unwrap();
        gate.close();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();

        engine.cancel_scan().unwrap();
        wait_for_finish(&events);

        assert_eq!(engine.storage_map().unwrap(), before);
        assert_eq!(before.phase, MapPhase::Complete { state: StateId::new(1) });
    }

    #[test]
    fn the_volume_is_not_read_per_query() {
        let (walk, gate) = ScriptedWalk::held();
        let fixture = Fixture::new(walk).volumes(startup());
        let volumes = Arc::clone(&fixture.volumes);
        let engine = fixture.engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        started(&events);

        for _ in 0..4 {
            assert_eq!(engine.storage_map().unwrap().phase, MapPhase::Scanning);
        }

        assert_eq!(volumes.space_calls(), 1);
        gate.release();
        wait_for_finish(&events);
    }
}

mod positive {
    use std::path::Path;

    use fresher_core::{
        Access, Capability, Descent, DiskState, IgnoreTarget, IgnoredEntry, MapPhase, MeaningGroup, NotScanned,
        PartKind, RemainderReason, StateId, Unreadable,
    };
    use fresher_testing::{FakeAccess, FakeVolumes};
    use serde_json::json;

    use super::{
        BATCH_LIMIT, alpha, caches, disk, facts, home, leaves, many_caches, next_batch, owned, part, parts, started,
        startup, state, two_volumes, walk,
    };
    use crate::testing::{Fixture, MemorySettings, ScriptedWalk, bytes, scan, wait_for_finish};

    #[test]
    fn explained_grows_while_the_scan_streams() {
        let (walk, first) = many_caches(ScriptedWalk::new(), 0, BATCH_LIMIT).pause();
        let (walk, last) = many_caches(walk, BATCH_LIMIT, BATCH_LIMIT).hold();
        let volumes = FakeVolumes::default().with("/", facts(1)).with_space("/", disk("HD", "/", 2000, 1000, 900));
        let engine = Fixture::new(walk).module(alpha()).volumes(volumes).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();

        next_batch(&events);
        let early = engine.storage_map().unwrap();
        first.release();
        next_batch(&events);
        let later = engine.storage_map().unwrap();

        assert_eq!(early.phase, MapPhase::Scanning);
        assert_eq!((part(&early, "explained"), part(&early, "not-scanned-yet")), (256, 744));
        assert_eq!((part(&later, "explained"), part(&later, "not-scanned-yet")), (512, 488));
        assert_eq!(later.volumes[0].capacity().bytes(), 2000);
        last.release();
        wait_for_finish(&events);
    }

    #[test]
    fn the_meaning_of_a_volume_during_a_scan_is_that_volume_of_the_shown_map() {
        let settings = MemorySettings::with(&[("core.scan.roots", json!(["/Volumes/Data", "~"]))]);
        let (walk, gate) = walk(vec![]).hold();
        let engine = Fixture::new(walk).module(alpha()).volumes(two_volumes()).settings(settings).engine();
        let events = engine.subscribe();
        engine.start_scan(crate::ScanMode::Auto).unwrap();
        started(&events);

        let map = engine.storage_map().unwrap();

        assert_eq!(map.phase, MapPhase::Scanning);
        assert_eq!(map.volumes.len(), 2);
        for (index, volume) in map.volumes.iter().enumerate() {
            let parts: Vec<MeaningGroup> = volume.parts().iter().cloned().map(MeaningGroup::Part).collect();
            assert_eq!(engine.storage_meaning(index).unwrap(), parts, "volume {index}");
        }
        gate.release();
        wait_for_finish(&events);
    }

    #[test]
    fn the_commit_gives_the_complete_map() {
        let engine = Fixture::new(walk(vec![home(1)])).module(alpha()).volumes(startup()).engine();
        scan(&engine);

        let map = engine.storage_map().unwrap();

        assert_eq!(map.phase, MapPhase::Complete { state: StateId::new(1) });
        let volume = &map.volumes[0];
        assert_eq!(
            parts(volume),
            owned(&[
                ("explained", 100),
                ("your-folders", 200),
                ("outside", 100),
                ("mac-os", 0),
                ("swap", 0),
                ("disk-own", 100),
                ("free", 500)
            ])
        );
        assert_eq!(volume.parts()[0].sizes, bytes(100));
        let reasons = vec![RemainderReason::SnapshotsAndBookkeeping, RemainderReason::ChangedDuringScan];
        assert_eq!(volume.parts()[2].kind, PartKind::Remainder { outside_scan: true, reasons });
        assert_eq!(volume.roots(), [home(1).roots()[0]]);
        assert!(!volume.disk_changed());
    }

    #[test]
    fn roots_on_two_volumes_give_two_maps_home_first() {
        let settings = MemorySettings::with(&[("core.scan.roots", json!(["/Volumes/Data", "~"]))]);
        let volumes = FakeVolumes::default()
            .with("/", facts(1))
            .with("/Volumes/Data", facts(2))
            .with_space("/", disk("Macintosh HD", "/", 1000, 400, 500))
            .with_space("/Volumes/Data", disk("Data", "/Volumes/Data", 600, 250, 350));
        let both = state(1, &[("/Volumes/Data", 250), ("/home", 300), ("/home/Caches", 100), ("/home/Movies", 150)])
            .with_owner(Path::new("/home/Caches"), caches());
        let engine = Fixture::new(walk(vec![both])).module(alpha()).volumes(volumes).settings(settings).engine();
        let names = |map: &fresher_core::StorageMap| {
            map.volumes.iter().map(|volume| volume.name().to_owned()).collect::<Vec<_>>()
        };
        assert_eq!(names(&engine.storage_map().unwrap()), ["Macintosh HD", "Data"]);
        scan(&engine);

        let map = engine.storage_map().unwrap();

        assert_eq!(names(&map), ["Macintosh HD", "Data"]);
        assert_eq!(parts(&map.volumes[0])[..3], owned(&[("explained", 100), ("your-folders", 200), ("outside", 100)]));
        assert_eq!(parts(&map.volumes[1])[..3], owned(&[("explained", 0), ("your-folders", 250), ("not-measured", 0)]));
        for (index, volume) in map.volumes.iter().enumerate() {
            assert_eq!(leaves(&engine.storage_meaning(index).unwrap()), volume.capacity().bytes(), "volume {index}");
        }
    }

    #[test]
    fn ignoring_a_category_moves_its_bytes_to_your_folders_without_a_rescan() {
        let walk = walk(vec![home(1)]);
        let commits = walk.commits();
        let engine = Fixture::new(walk).module(alpha()).volumes(startup()).engine();
        scan(&engine);
        assert_eq!(part(&engine.storage_map().unwrap(), "explained"), 100);

        engine.ignore(&IgnoreTarget::Category { category: "alpha.caches".into() }).unwrap();

        let map = engine.storage_map().unwrap();
        assert_eq!((part(&map, "explained"), part(&map, "your-folders")), (0, 300));
        assert_eq!(commits.lock().unwrap().len(), 1);
        engine.restore_ignored(&IgnoredEntry::Category { category: "alpha.caches".into() }).unwrap();
        assert_eq!(part(&engine.storage_map().unwrap(), "explained"), 100);
    }

    #[test]
    fn the_folders_the_scan_could_not_read_are_a_reason_of_the_remainder() {
        let unread = state(
            1,
            &[("/home", 300), ("/home/Caches", 100), ("/home/Locked", 0), ("/home/Locked/inner", 0), ("/home/Disk", 0)],
        )
        .with_owner(Path::new("/home/Caches"), caches())
        .with_not_scanned(Path::new("/home/Locked"), NotScanned::Unreadable(Unreadable::Guarded))
        .with_not_scanned(Path::new("/home/Disk"), NotScanned::OtherVolume);
        let access = FakeAccess::with(&[(Capability::AllFiles, Access::Denied)]);
        let engine = Fixture::new(walk(vec![unread])).module(alpha()).volumes(startup()).access(access).engine();
        scan(&engine);

        let map = engine.storage_map().unwrap();

        let reasons = vec![
            RemainderReason::Unreadable { folders: 2, needs: Some(Capability::AllFiles) },
            RemainderReason::SnapshotsAndBookkeeping,
            RemainderReason::ChangedDuringScan,
        ];
        assert_eq!(map.volumes[0].parts()[2].kind, PartKind::Remainder { outside_scan: true, reasons });
    }

    #[test]
    fn the_two_views_have_one_total() {
        let engine = Fixture::new(walk(vec![home(1)])).module(alpha()).volumes(startup()).engine();
        scan(&engine);

        let map = engine.storage_map().unwrap();
        let groups = engine.storage_meaning(0).unwrap();

        let parts: u64 = map.volumes[0].parts().iter().map(|part| part.sizes.allocated.bytes()).sum();
        assert_eq!(leaves(&groups), parts);
        assert_eq!(parts, 1000);
    }

    #[test]
    fn storage_place_descends_from_the_home() {
        let chain = state(
            1,
            &[
                ("/home", 300),
                ("/home/Library", 290),
                ("/home/Library/Caches", 290),
                ("/home/Library/Caches/a", 200),
                ("/home/Library/Caches/b", 90),
            ],
        );
        let root = chain.roots()[0];
        let engine = Fixture::new(walk(vec![chain])).volumes(startup()).engine();
        scan(&engine);

        let view = engine.storage_place(root, Descent::Through, 0, 10).unwrap();

        let path: Vec<&str> = view.path.iter().map(|node| node.name.as_str()).collect();
        assert_eq!(path, ["home", "Library", "Caches"]);
        let rows: Vec<&str> = view.rows.iter().map(|row| row.node.name.as_str()).collect();
        assert_eq!(rows, ["a", "b"]);
        assert_eq!(engine.storage_place(root, Descent::Through, 0, 1).unwrap().rows.len(), 1);
    }
}
