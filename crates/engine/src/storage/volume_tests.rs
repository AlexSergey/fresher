use std::path::PathBuf;

use fresher_core::{
    ByteSize, DiskSpace, MapPart, MapPhase, Neighbour, NeighbourRole, NodeId, PartKind, Sizes, StateId, VolumeMap,
};
use fresher_testing::FakeVolumes;

use crate::storage::volume::{RootOnVolume, VolumeInput, volume_map};

const KIB: u64 = 1024;
const GIB: u64 = 1024 * 1024 * 1024;

fn complete() -> MapPhase {
    MapPhase::Complete { state: StateId::new(1) }
}

fn root(path: &str, sizes: u64, explained: u64) -> RootOnVolume {
    RootOnVolume {
        path: PathBuf::from(path),
        node: Some(NodeId::new(StateId::new(1), 0)),
        sizes: Sizes::complete(ByteSize::new(sizes), ByteSize::new(sizes)),
        explained: Sizes::complete(ByteSize::new(explained), ByteSize::new(explained)),
    }
}

/// A volume mounted at `/System/Volumes/Data` with no neighbours.
fn data(capacity: u64, used: u64, free: u64) -> DiskSpace {
    DiskSpace {
        mount_point: PathBuf::from("/System/Volumes/Data"),
        ..FakeVolumes::disk("Data", ByteSize::new(capacity), ByteSize::new(used), ByteSize::new(free))
    }
}

fn neighbour(name: &str, role: NeighbourRole, used: u64) -> Neighbour {
    Neighbour { name: name.to_owned(), role, used: ByteSize::new(used) }
}

fn input(space: Option<DiskSpace>, roots: Vec<RootOnVolume>, phase: MapPhase) -> VolumeInput {
    VolumeInput { space, roots, unreadable_folders: 0, needs: None, phase }
}

fn size_of(map: &VolumeMap, wanted: fn(&PartKind) -> bool) -> u64 {
    map.parts().iter().filter(|part| wanted(&part.kind)).map(|part| part.sizes.allocated.bytes()).sum()
}

fn remainder(map: &VolumeMap) -> &MapPart {
    map.parts().iter().find(|part| matches!(part.kind, PartKind::Remainder { .. })).unwrap()
}

fn total(map: &VolumeMap) -> u64 {
    map.parts().iter().map(|part| part.sizes.allocated.bytes()).sum()
}

mod negative {
    use std::collections::{HashMap, HashSet};

    use fresher_core::{
        Capability, GuardedFolder, MapPhase, MountKind, NeighbourRole, PartKind, RemainderReason, StorageMapError,
    };

    use super::{complete, data, input, neighbour, remainder, root, size_of, total, volume_map};

    #[test]
    fn findings_past_the_used_space_during_a_scan_do_not_mark_the_disk_changed() {
        let mut found = root("/Users/me", 0, 500);
        found.node = None;

        let map = volume_map(input(Some(data(1000, 400, 500)), vec![found], MapPhase::Scanning)).unwrap();

        assert!(!map.disk_changed());
        assert_eq!(map.changed_note(), None);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::Explained), 500);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::NotScannedYet), 0);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::DiskOwn), 0);
        assert_eq!((total(&map), map.capacity().bytes()), (1000, 1000));
    }

    #[test]
    fn neighbours_past_the_largest_size_are_an_overflow() {
        let mut space = data(1000, 400, 500);
        space.neighbours = vec![
            neighbour("Macintosh HD", NeighbourRole::System, u64::MAX - 1),
            neighbour("Preboot", NeighbourRole::System, 10),
        ];

        let map = volume_map(input(Some(space), vec![root("/Users/me", 300, 100)], complete()));

        assert_eq!(map, Err(StorageMapError::Overflow));
    }

    #[test]
    fn neighbours_that_fit_alone_but_not_with_the_used_space_are_an_overflow() {
        let mut space = data(1000, 400, 500);
        space.neighbours = vec![neighbour("Backup", NeighbourRole::Other, u64::MAX - 100)];

        let map = volume_map(input(Some(space), vec![root("/Users/me", 300, 100)], complete()));

        assert_eq!(map, Err(StorageMapError::Overflow));
    }

    #[test]
    fn a_negative_remainder_is_zero_with_the_note() {
        let map = volume_map(input(Some(data(1000, 100, 500)), vec![root("/Users/me", 150, 40)], complete())).unwrap();

        assert_eq!(remainder(&map).sizes.allocated.bytes(), 0);
        assert!(map.disk_changed());
        assert_eq!(map.changed_note().unwrap().as_str(), "core.storage.changed");
        assert_eq!(size_of(&map, |kind| *kind == PartKind::DiskOwn), 1000 - 500 - 150);
        assert_eq!((total(&map), map.capacity().bytes()), (1000, 1000));
    }

    #[test]
    fn a_negative_disk_own_space_is_zero() {
        let mut space = data(1000, 600, 300);
        space.neighbours = vec![neighbour("Macintosh HD", NeighbourRole::System, 200)];

        let map = volume_map(input(Some(space), vec![root("/Users/me", 500, 0)], complete())).unwrap();

        assert_eq!(size_of(&map, |kind| *kind == PartKind::DiskOwn), 0);
        assert_eq!(map.capacity().bytes(), 600 + 200 + 300);
        assert_eq!(total(&map), 1100);
        assert!(!map.disk_changed());
    }

    #[test]
    fn without_volume_facts_only_the_scanned_parts_are_drawn() {
        let map = volume_map(input(None, vec![root("/Users/me", 300, 120)], complete())).unwrap();

        let kinds: Vec<&PartKind> = map.parts().iter().map(|part| &part.kind).collect();
        assert_eq!(kinds, [&PartKind::Explained, &PartKind::YourFolders]);
        assert_eq!(map.parts()[0].sizes.allocated.bytes(), 120);
        assert_eq!(map.parts()[1].sizes.allocated.bytes(), 180);
        assert_eq!((map.capacity().bytes(), map.kind(), map.name()), (300, MountKind::Internal, "me"));
        assert_eq!(map.purgeable(), None);
    }

    #[test]
    fn without_volume_facts_a_scan_draws_what_it_found_so_far() {
        let mut found = root("/", 0, 70);
        found.node = None;

        let map = volume_map(input(None, vec![found], MapPhase::Scanning)).unwrap();

        let kinds: Vec<&PartKind> = map.parts().iter().map(|part| &part.kind).collect();
        assert_eq!(kinds, [&PartKind::Explained, &PartKind::NotScannedYet]);
        assert_eq!((map.capacity().bytes(), map.name(), map.roots()), (70, "/", &[][..]));
    }

    #[test]
    fn a_home_root_is_outside_scan() {
        let map = volume_map(input(Some(data(1000, 400, 500)), vec![root("/Users/me", 300, 100)], complete())).unwrap();

        assert!(matches!(remainder(&map).kind, PartKind::Remainder { outside_scan: true, .. }));
        assert_eq!(remainder(&map).sizes.allocated.bytes(), 100);
        assert_eq!(remainder(&map).sizes.reclaimable.bytes(), 0);
    }

    #[test]
    fn a_root_of_slash_on_a_data_volume_mounted_elsewhere_is_outside_scan() {
        let map = volume_map(input(Some(data(1000, 400, 500)), vec![root("/", 300, 100)], complete())).unwrap();

        assert!(matches!(remainder(&map).kind, PartKind::Remainder { outside_scan: true, .. }));
    }

    #[test]
    fn unreadable_folders_are_a_reason_not_a_part() {
        let needs = Some(Capability::Folder(GuardedFolder::Documents));
        let mut asked = input(Some(data(1000, 400, 500)), vec![root("/Users/me", 300, 100)], complete());
        asked.unreadable_folders = 4;
        asked.needs = needs;

        let map = volume_map(asked).unwrap();

        let PartKind::Remainder { reasons, .. } = &remainder(&map).kind else { unreachable!() };
        assert_eq!(
            reasons,
            &[
                RemainderReason::Unreadable { folders: 4, needs },
                RemainderReason::SnapshotsAndBookkeeping,
                RemainderReason::ChangedDuringScan,
            ]
        );
        assert_eq!(map.parts().len(), 7);
        assert_eq!(total(&map), 1000);
    }

    #[test]
    fn without_unreadable_folders_the_reasons_are_the_snapshots_and_the_changes() {
        let map = volume_map(input(Some(data(1000, 400, 500)), vec![root("/Users/me", 300, 100)], complete())).unwrap();

        let PartKind::Remainder { reasons, .. } = &remainder(&map).kind else { unreachable!() };
        assert_eq!(reasons, &[RemainderReason::SnapshotsAndBookkeeping, RemainderReason::ChangedDuringScan]);
    }

    #[test]
    fn no_part_is_named_other() {
        let texts: HashMap<String, String> =
            serde_json::from_str(include_str!("../../../core/locales/en.json")).unwrap();
        let kinds = [
            PartKind::Explained,
            PartKind::YourFolders,
            PartKind::NotScannedYet,
            PartKind::Remainder { outside_scan: true, reasons: vec![] },
            PartKind::Remainder { outside_scan: false, reasons: vec![] },
            PartKind::MacOs,
            PartKind::SwapAndSleep,
            PartKind::OtherVolume { name: "Backup".into() },
            PartKind::DiskOwn,
            PartKind::Free,
        ];

        let names: HashSet<String> = kinds.iter().map(|kind| kind.name_key().as_str().to_owned()).collect();
        let descriptions: HashSet<String> =
            kinds.iter().map(|kind| kind.description_key().as_str().to_owned()).collect();

        assert_eq!((names.len(), descriptions.len()), (kinds.len(), kinds.len()));
        for key in names.iter().chain(&descriptions) {
            let text = texts.get(key).map(|text| text.trim().to_lowercase());
            assert!(text.is_some(), "{key} has no text");
            assert_ne!(text.as_deref(), Some("other"), "{key} is named Other");
        }
    }
}

mod positive {
    use fresher_core::{ByteSize, MapPhase, MeaningGroup, MountKind, NeighbourRole, PartKind, SummaryKind};

    use super::{GIB, KIB, complete, data, input, neighbour, remainder, root, size_of, total, volume_map};
    use crate::storage::meaning::append_volume_parts;

    /// The kinds of the parts `append_volume_parts` adds to no groups.
    fn appended(map: &fresher_core::VolumeMap) -> Vec<PartKind> {
        let mut groups = Vec::new();
        append_volume_parts(&mut groups, map);
        let kinds: Vec<PartKind> = groups
            .iter()
            .filter_map(|group| match group {
                MeaningGroup::Part(part) => Some(part.kind.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(kinds.len(), groups.len(), "only parts are appended");
        kinds
    }

    #[test]
    fn a_scanning_map_ends_the_meaning_view_with_its_volume_parts_only() {
        let mut space = data(1000, 400, 300);
        space.neighbours = vec![neighbour("Backup", NeighbourRole::Other, 100)];
        let mut found = root("/Users/me", 0, 150);
        found.node = None;

        let map = volume_map(input(Some(space), vec![found], MapPhase::Scanning)).unwrap();

        assert_eq!(
            appended(&map),
            [
                PartKind::MacOs,
                PartKind::SwapAndSleep,
                PartKind::OtherVolume { name: "Backup".into() },
                PartKind::DiskOwn,
                PartKind::Free,
            ]
        );
    }

    #[test]
    fn a_complete_map_ends_the_meaning_view_with_its_remainder_first() {
        let map = volume_map(input(Some(data(1000, 400, 500)), vec![root("/Users/me", 300, 100)], complete())).unwrap();

        let kinds = appended(&map);

        assert!(matches!(kinds.first(), Some(PartKind::Remainder { outside_scan: true, .. })));
        assert_eq!(kinds[1..], [PartKind::MacOs, PartKind::SwapAndSleep, PartKind::DiskOwn, PartKind::Free]);
    }

    #[test]
    fn a_complete_map_carries_its_summary() {
        let map = volume_map(input(Some(data(1000, 400, 500)), vec![root("/Users/me", 300, 100)], complete())).unwrap();

        let summary: Vec<(SummaryKind, u64)> =
            map.summary().iter().map(|part| (part.kind, part.sizes.allocated.bytes())).collect();

        assert_eq!(
            summary,
            [
                (SummaryKind::CanBeFreed, 100),
                (SummaryKind::FilesAndApps, 300),
                (SummaryKind::MacOs, 0),
                (SummaryKind::SystemSpace, 100),
                (SummaryKind::Free, 500),
            ]
        );
    }

    #[test]
    fn the_developers_mac_adds_up() {
        let mut space = data(482_797_652 * KIB, 380_727_388 * KIB, 77_429_216 * KIB);
        space.name = "Macintosh HD - Data".into();
        space.purgeable = Some(ByteSize::new(30 * GIB));
        space.neighbours = vec![
            neighbour("Macintosh HD", NeighbourRole::System, 13_434_928 * KIB),
            neighbour("Preboot", NeighbourRole::System, 9_541_224 * KIB),
            neighbour("VM", NeighbourRole::VirtualMemory, 24 * KIB),
            neighbour("Update", NeighbourRole::System, 23_756 * KIB),
        ];

        let map = volume_map(input(Some(space), vec![root("/Users/me", 200 * GIB, 50 * GIB)], complete())).unwrap();

        let kinds: Vec<&PartKind> = map.parts().iter().map(|part| &part.kind).collect();
        assert!(matches!(
            kinds[..],
            [
                PartKind::Explained,
                PartKind::YourFolders,
                PartKind::Remainder { outside_scan: true, .. },
                PartKind::MacOs,
                PartKind::SwapAndSleep,
                PartKind::DiskOwn,
                PartKind::Free,
            ]
        ));
        assert_eq!(total(&map), 482_797_652 * KIB);
        assert_eq!(map.capacity().bytes(), 482_797_652 * KIB);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::MacOs), (13_434_928 + 9_541_224 + 23_756) * KIB);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::SwapAndSleep), 24 * KIB);
        let disk_own = size_of(&map, |kind| *kind == PartKind::DiskOwn);
        assert_eq!(disk_own, 1_641_116 * KIB);
        assert!((GIB * 3 / 2..GIB * 17 / 10).contains(&disk_own));
        assert_eq!(remainder(&map).sizes.allocated.bytes(), 380_727_388 * KIB - 200 * GIB);
        assert_eq!(map.purgeable(), Some(ByteSize::new(30 * GIB)));
        assert_eq!((map.name(), map.kind(), map.disk_changed()), ("Macintosh HD - Data", MountKind::Internal, false));
    }

    #[test]
    fn a_root_at_the_mount_point_is_not_measured() {
        let map = volume_map(input(
            Some(data(1000, 400, 500)),
            vec![root("/Users/me", 100, 0), root("/System/Volumes/Data/", 300, 100)],
            complete(),
        ))
        .unwrap();

        assert!(matches!(remainder(&map).kind, PartKind::Remainder { outside_scan: false, .. }));
        assert_eq!(map.roots().len(), 2);
    }

    #[test]
    fn scanning_splits_used_into_explained_and_not_scanned_yet() {
        let mut found = root("/Users/me", 0, 150);
        found.node = None;

        let map = volume_map(input(Some(data(1000, 400, 500)), vec![found], MapPhase::Scanning)).unwrap();

        let kinds: Vec<&PartKind> = map.parts().iter().map(|part| &part.kind).collect();
        assert_eq!(
            kinds,
            [
                &PartKind::Explained,
                &PartKind::NotScannedYet,
                &PartKind::MacOs,
                &PartKind::SwapAndSleep,
                &PartKind::DiskOwn,
                &PartKind::Free
            ]
        );
        assert_eq!(size_of(&map, |kind| *kind == PartKind::Explained), 150);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::NotScannedYet), 250);
        assert_eq!(size_of(&map, |kind| *kind == PartKind::DiskOwn), 100);
        assert_eq!((total(&map), map.roots()), (1000, &[][..]));
    }

    #[test]
    fn before_a_scan_the_used_space_is_not_scanned_yet() {
        let mut configured = root("/Users/me", 0, 0);
        configured.node = None;

        let map = volume_map(input(Some(data(1000, 400, 500)), vec![configured], MapPhase::NoScan)).unwrap();

        assert_eq!(size_of(&map, |kind| *kind == PartKind::NotScannedYet), 400);
        assert_eq!(total(&map), 1000);
    }

    #[test]
    fn each_other_volume_is_its_own_part() {
        let mut space = data(1000, 400, 300);
        space.neighbours =
            vec![neighbour("Backup", NeighbourRole::Other, 100), neighbour("Media", NeighbourRole::Other, 50)];

        let map = volume_map(input(Some(space), vec![root("/Users/me", 300, 100)], complete())).unwrap();

        let others: Vec<(&PartKind, u64)> = map
            .parts()
            .iter()
            .filter(|part| matches!(part.kind, PartKind::OtherVolume { .. }))
            .map(|part| (&part.kind, part.sizes.allocated.bytes()))
            .collect();
        assert_eq!(
            others,
            [
                (&PartKind::OtherVolume { name: "Backup".into() }, 100),
                (&PartKind::OtherVolume { name: "Media".into() }, 50),
            ]
        );
        assert_eq!(size_of(&map, |kind| *kind == PartKind::DiskOwn), 150);
    }
}

mod property {
    use fresher_core::{MapPhase, NeighbourRole, PartKind};
    use proptest::prelude::*;

    use super::{complete, data, input, neighbour, root, size_of, total, volume_map};

    const MOST: u64 = 1 << 50;

    fn role(index: u8) -> NeighbourRole {
        match index % 3 {
            0 => NeighbourRole::System,
            1 => NeighbourRole::VirtualMemory,
            _ => NeighbourRole::Other,
        }
    }

    proptest! {
        #[test]
        fn the_parts_add_up_to_the_capacity(
            capacity in 0..MOST,
            used in 0..MOST,
            free in 0..MOST,
            roots in prop::collection::vec((0..MOST, 0..MOST), 0..4),
            neighbours in prop::collection::vec((0_u8..3, 0..MOST), 0..5),
            facts in any::<bool>(),
            phase in 0_u8..3,
        ) {
            let mut space = data(capacity, used, free);
            space.neighbours = neighbours.iter().map(|&(index, used)| neighbour("N", role(index), used)).collect();
            // Explained may pass the root's own size: a finding counted while the disk changed.
            let roots = roots.iter().map(|&(sizes, explained)| root("/Users/me", sizes, explained)).collect();
            let phase = match phase {
                0 => MapPhase::NoScan,
                1 => MapPhase::Scanning,
                _ => complete(),
            };

            let map = volume_map(input(facts.then_some(space), roots, phase)).unwrap();

            prop_assert_eq!(total(&map), map.capacity().bytes());
            if facts {
                prop_assert!(map.capacity().bytes() >= capacity);
                let disk_own = size_of(&map, |kind| *kind == PartKind::DiskOwn);
                prop_assert!(disk_own == 0 || map.capacity().bytes() == capacity);
            }
        }
    }
}
