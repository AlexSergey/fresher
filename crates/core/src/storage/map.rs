//! The map of the scanned volumes: each volume's capacity split into parts that add up to it.

use serde::Serialize;
use ts_rs::TS;

use super::summary::{SummaryPart, summarize};
use crate::{ByteSize, Capability, MountKind, NodeId, Sizes, StateId, Text, TextArg, TextKey};

/// The answer of `get_storage_map`: one map per volume that holds a scan root, the home's volume first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct StorageMap {
    pub phase: MapPhase,
    pub volumes: Vec<VolumeMap>,
}

/// Where the map's numbers come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum MapPhase {
    /// No scan yet: the volumes' own facts only.
    NoScan,
    /// A scan is running: "Explained" is the counted totals so far, the rest is "Not scanned yet".
    Scanning,
    /// Every size comes from the committed tree `state`.
    Complete { state: StateId },
}

/// The map of one volume, drawn over the space it shares. Built only by [`VolumeMap::new`], so its parts always add up
/// to its capacity and come in the fixed order of [`PartKind`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VolumeMap {
    name: String,
    kind: MountKind,
    capacity: ByteSize,
    parts: Vec<MapPart>,
    /// What the disk card draws; the parts stay for the by-meaning view and the place view.
    summary: Vec<SummaryPart>,
    purgeable: Option<ByteSize>,
    roots: Vec<NodeId>,
    /// The roots that are walked places, each with its name: shown by that name, not by its folder's.
    places: Vec<MapPlace>,
    disk_changed: bool,
}

/// A root of a volume's map that is a place walked outside the scan roots (a temporary folder), with its name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct MapPlace {
    pub root: NodeId,
    pub name: Text,
}

/// Why a volume's map was not built.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum StorageMapError {
    /// The parts' allocated sizes do not add up to the capacity.
    #[error("the parts add up to {sum}, not to the capacity {capacity}")]
    PartsDoNotAddUp { capacity: ByteSize, sum: ByteSize },
    /// The parts' allocated sizes add up to more than a size can hold.
    #[error("the parts add up to more than a size can hold")]
    Overflow,
    /// The part at `position` comes before one it must follow, or repeats a kind that comes once.
    #[error("the part at {position} is out of order")]
    OutOfOrder { position: usize },
}

impl VolumeMap {
    /// The map of the volume `name`. `parts` are in the order of [`PartKind`] (only `OtherVolume` may repeat) and their
    /// allocated sizes add up to `capacity`; `purgeable` is shown with Free; `roots` are where the place view starts;
    /// `disk_changed` says the scan counted more than the volume reports.
    pub fn new(
        name: String,
        kind: MountKind,
        capacity: ByteSize,
        parts: Vec<MapPart>,
        purgeable: Option<ByteSize>,
        roots: Vec<NodeId>,
        disk_changed: bool,
    ) -> Result<Self, StorageMapError> {
        check_order(&parts)?;
        let sum = parts
            .iter()
            .try_fold(ByteSize::ZERO, |sum, part| sum.checked_add(part.sizes.allocated))
            .ok_or(StorageMapError::Overflow)?;
        if sum != capacity {
            return Err(StorageMapError::PartsDoNotAddUp { capacity, sum });
        }
        let summary = summarize(&parts);
        Ok(Self { name, kind, capacity, parts, summary, purgeable, roots, places: Vec::new(), disk_changed })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> MountKind {
        self.kind
    }

    /// The whole the parts are drawn over.
    pub fn capacity(&self) -> ByteSize {
        self.capacity
    }

    /// The parts, in the order of [`PartKind`].
    pub fn parts(&self) -> &[MapPart] {
        &self.parts
    }

    /// What the disk card draws; the parts stay for the by-meaning view and the place view. Five parts, in the order
    /// of [`SummaryKind::ALL`](crate::SummaryKind::ALL).
    pub fn summary(&self) -> &[SummaryPart] {
        &self.summary
    }

    /// What macOS can free on its own when space runs low: shown with Free, not a part.
    pub fn purgeable(&self) -> Option<ByteSize> {
        self.purgeable
    }

    /// Where the place view starts; empty before a commit.
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }

    /// The map with `places` named among its roots.
    #[must_use]
    pub fn with_places(mut self, places: Vec<MapPlace>) -> Self {
        self.places = places;
        self
    }

    /// The roots that are walked places, with their names.
    pub fn places(&self) -> &[MapPlace] {
        &self.places
    }

    /// The scan counted more than the volume reports: the remainder is shown as zero.
    pub fn disk_changed(&self) -> bool {
        self.disk_changed
    }

    /// The note on Free: `core.storage.purgeable` with the purgeable `size`; `None` when it is not known.
    pub fn purgeable_note(&self) -> Option<Text> {
        self.purgeable.map(|size| {
            Text::new(TextKey::from_valid("core.storage.purgeable".into())).with("size", TextArg::Size(size))
        })
    }

    /// The note of a negative remainder: `core.storage.changed`; `None` when the disk did not change during the scan.
    pub fn changed_note(&self) -> Option<TextKey> {
        self.disk_changed.then(|| TextKey::from_valid("core.storage.changed".into()))
    }
}

fn check_order(parts: &[MapPart]) -> Result<(), StorageMapError> {
    for (position, pair) in parts.windows(2).enumerate() {
        let (before, after) = (pair[0].kind.rank(), pair[1].kind.rank());
        let repeats_once_only = before == after && !matches!(pair[1].kind, PartKind::OtherVolume { .. });
        if after < before || repeats_once_only {
            return Err(StorageMapError::OutOfOrder { position: position + 1 });
        }
    }
    Ok(())
}

/// One part of a volume's map. The headline is `sizes.allocated`; `sizes.reclaimable` is "freed if removed".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct MapPart {
    pub kind: PartKind,
    pub sizes: Sizes,
}

/// What a part of the map is, in the order the parts are drawn. Only `Explained` is ever offered for cleaning.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum PartKind {
    /// The findings of the modules.
    Explained,
    /// Everything scanned that no module claimed.
    YourFolders,
    /// During a scan only: the volume's used space the scan has not reached yet.
    NotScannedYet,
    /// The volume's used space the scan did not measure, with its reasons. `outside_scan`: the roots do not cover the
    /// volume's mount point ("Outside the scanned folders"); otherwise it is "Not measured".
    Remainder { outside_scan: bool, reasons: Vec<RemainderReason> },
    /// The system's volumes that share the disk.
    MacOs,
    /// The virtual memory volume: swap files and the sleep image.
    SwapAndSleep,
    /// Another volume of the user's on the same disk, by its own name.
    OtherVolume { name: String },
    /// Capacity minus free minus every mounted volume's used space.
    DiskOwn,
    /// The space the user can still write.
    Free,
}

impl PartKind {
    /// What the part is called: `core.storage.part.<kind>.name`; the remainder's depends on the roots.
    pub fn name_key(&self) -> TextKey {
        self.key("name")
    }

    /// What it is and what the user can do: `core.storage.part.<kind>.description`, or the remainder's own.
    pub fn description_key(&self) -> TextKey {
        self.key("description")
    }

    fn key(&self, text: &str) -> TextKey {
        let prefix = match self {
            Self::Remainder { outside_scan: true, .. } => "core.storage.remainder.outside",
            Self::Remainder { outside_scan: false, .. } => "core.storage.remainder.not-measured",
            Self::Explained => "core.storage.part.explained",
            Self::YourFolders => "core.storage.part.your-folders",
            Self::NotScannedYet => "core.storage.part.not-scanned-yet",
            Self::MacOs => "core.storage.part.mac-os",
            Self::SwapAndSleep => "core.storage.part.swap-and-sleep",
            Self::OtherVolume { .. } => "core.storage.part.other-volume",
            Self::DiskOwn => "core.storage.part.disk-own",
            Self::Free => "core.storage.part.free",
        };
        TextKey::from_valid(format!("{prefix}.{text}"))
    }

    const fn rank(&self) -> u8 {
        match self {
            Self::Explained => 0,
            Self::YourFolders => 1,
            Self::NotScannedYet => 2,
            Self::Remainder { .. } => 3,
            Self::MacOs => 4,
            Self::SwapAndSleep => 5,
            Self::OtherVolume { .. } => 6,
            Self::DiskOwn => 7,
            Self::Free => 8,
        }
    }
}

/// Why some of a volume's used space was not measured: a plain line under the remainder, never a category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", rename_all_fields = "camelCase")]
pub enum RemainderReason {
    /// Folders the app could not read, and the permission that would show them.
    Unreadable {
        #[ts(type = "number")]
        folders: u64,
        needs: Option<Capability>,
    },
    /// Local snapshots and the file system's own bookkeeping.
    SnapshotsAndBookkeeping,
    /// Files changed on the disk while the scan ran.
    ChangedDuringScan,
}

impl RemainderReason {
    /// The reason's line: `core.storage.reason.<reason>`.
    pub fn text_key(self) -> TextKey {
        let reason = match self {
            Self::Unreadable { .. } => "unreadable",
            Self::SnapshotsAndBookkeeping => "snapshots-and-bookkeeping",
            Self::ChangedDuringScan => "changed-during-scan",
        };
        TextKey::from_valid(format!("core.storage.reason.{reason}"))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{ByteSize, Capability, GuardedFolder, MapPart, PartKind, RemainderReason, Sizes};

    /// One part of each kind, in order, the remainder outside the scan with every reason.
    pub(crate) fn every_kind() -> Vec<PartKind> {
        vec![
            PartKind::Explained,
            PartKind::YourFolders,
            PartKind::NotScannedYet,
            PartKind::Remainder { outside_scan: true, reasons: every_reason() },
            PartKind::MacOs,
            PartKind::SwapAndSleep,
            PartKind::OtherVolume { name: "Backup".into() },
            PartKind::DiskOwn,
            PartKind::Free,
        ]
    }

    pub(crate) fn every_reason() -> Vec<RemainderReason> {
        vec![
            RemainderReason::Unreadable { folders: 3, needs: Some(Capability::Folder(GuardedFolder::Documents)) },
            RemainderReason::SnapshotsAndBookkeeping,
            RemainderReason::ChangedDuringScan,
        ]
    }

    pub(crate) fn part(kind: PartKind, allocated: u64) -> MapPart {
        MapPart { kind, sizes: Sizes::complete(ByteSize::new(allocated), ByteSize::new(allocated)) }
    }

    mod negative {
        use super::{every_kind, part};
        use crate::{ByteSize, MountKind, PartKind, StorageMapError, VolumeMap};

        fn map(capacity: u64, parts: Vec<crate::MapPart>) -> Result<VolumeMap, StorageMapError> {
            VolumeMap::new(
                "Macintosh HD".into(),
                MountKind::Internal,
                ByteSize::new(capacity),
                parts,
                None,
                vec![],
                false,
            )
        }

        #[test]
        fn parts_that_do_not_add_up_to_the_capacity_are_refused() {
            let parts = vec![part(PartKind::Explained, 10), part(PartKind::Free, 20)];

            assert_eq!(
                map(31, parts),
                Err(StorageMapError::PartsDoNotAddUp { capacity: ByteSize::new(31), sum: ByteSize::new(30) })
            );
        }

        #[test]
        fn parts_out_of_order_are_refused() {
            let reversed = vec![part(PartKind::Free, 10), part(PartKind::Explained, 10)];
            let repeated = vec![part(PartKind::MacOs, 10), part(PartKind::MacOs, 10)];
            let mut late = every_kind().into_iter().map(|kind| part(kind, 1)).collect::<Vec<_>>();
            late.swap(4, 5);

            assert_eq!(map(20, reversed), Err(StorageMapError::OutOfOrder { position: 1 }));
            assert_eq!(map(20, repeated), Err(StorageMapError::OutOfOrder { position: 1 }));
            assert_eq!(map(9, late), Err(StorageMapError::OutOfOrder { position: 5 }));
        }

        #[test]
        fn an_overflowing_sum_is_refused() {
            let parts = vec![part(PartKind::Explained, u64::MAX), part(PartKind::Free, 1)];

            assert_eq!(map(0, parts), Err(StorageMapError::Overflow));
        }
    }

    mod positive {
        use serde_json::json;

        use super::{every_kind, part};
        use crate::{
            ByteSize, MapPhase, MapPlace, MountKind, NodeId, PartKind, RemainderReason, StateId, StorageMap, Text,
            TextArg, TextKey, VolumeMap,
        };

        #[test]
        fn a_volume_map_serializes_in_camel_case() {
            let parts = every_kind()
                .into_iter()
                .map(|kind| match kind {
                    PartKind::Remainder { outside_scan, .. } => PartKind::Remainder { outside_scan, reasons: vec![] },
                    kind => kind,
                })
                .map(|kind| part(kind, 10))
                .collect();
            let roots = vec![NodeId::new(StateId::new(2), 0)];
            let volume = VolumeMap::new(
                "Macintosh HD".into(),
                MountKind::Internal,
                ByteSize::new(90),
                parts,
                Some(ByteSize::new(5)),
                roots,
                false,
            )
            .unwrap()
            .with_places(vec![MapPlace {
                root: NodeId::new(StateId::new(2), 0),
                name: Text::new(TextKey::from_valid("core.place.shared-temp.name".into())),
            }]);
            let map = StorageMap { phase: MapPhase::Complete { state: StateId::new(2) }, volumes: vec![volume] };
            let sizes_of = |bytes: u64| json!({ "logical": bytes, "allocated": bytes, "reclaimable": bytes, "completeness": "complete" });
            let sizes = sizes_of(10);

            assert_eq!(
                serde_json::to_value(map).unwrap(),
                json!({
                    "phase": { "kind": "complete", "state": 2 },
                    "volumes": [{
                        "name": "Macintosh HD",
                        "kind": "internal",
                        "capacity": 90,
                        "parts": [
                            { "kind": { "kind": "explained" }, "sizes": sizes },
                            { "kind": { "kind": "your-folders" }, "sizes": sizes },
                            { "kind": { "kind": "not-scanned-yet" }, "sizes": sizes },
                            { "kind": { "kind": "remainder", "outsideScan": true, "reasons": [] }, "sizes": sizes },
                            { "kind": { "kind": "mac-os" }, "sizes": sizes },
                            { "kind": { "kind": "swap-and-sleep" }, "sizes": sizes },
                            { "kind": { "kind": "other-volume", "name": "Backup" }, "sizes": sizes },
                            { "kind": { "kind": "disk-own" }, "sizes": sizes },
                            { "kind": { "kind": "free" }, "sizes": sizes },
                        ],
                        "summary": [
                            { "kind": "can-be-freed", "sizes": sizes },
                            { "kind": "files-and-apps", "sizes": sizes_of(40) },
                            { "kind": "mac-os", "sizes": sizes },
                            { "kind": "system-space", "sizes": sizes_of(20) },
                            { "kind": "free", "sizes": sizes },
                        ],
                        "purgeable": 5,
                        "roots": [{ "state": 2, "index": 0 }],
                        "places": [{
                            "root": { "state": 2, "index": 0 },
                            "name": { "key": "core.place.shared-temp.name", "args": {} },
                        }],
                        "diskChanged": false,
                    }],
                })
            );
        }

        #[test]
        fn a_remainder_serializes_its_reasons() {
            let remainder = PartKind::Remainder { outside_scan: false, reasons: super::every_reason() };

            assert_eq!(
                serde_json::to_value(remainder).unwrap(),
                json!({
                    "kind": "remainder",
                    "outsideScan": false,
                    "reasons": [
                        { "kind": "unreadable", "folders": 3, "needs": { "kind": "folder", "value": "documents" } },
                        { "kind": "snapshots-and-bookkeeping" },
                        { "kind": "changed-during-scan" },
                    ],
                })
            );
        }

        #[test]
        fn the_phases_serialize_by_kind() {
            assert_eq!(serde_json::to_value(MapPhase::NoScan).unwrap(), json!({ "kind": "no-scan" }));
            assert_eq!(serde_json::to_value(MapPhase::Scanning).unwrap(), json!({ "kind": "scanning" }));
        }

        #[test]
        fn other_volumes_may_repeat_and_the_notes_follow_the_facts() {
            let parts = vec![
                part(PartKind::OtherVolume { name: "Backup".into() }, 10),
                part(PartKind::OtherVolume { name: "Media".into() }, 10),
                part(PartKind::Free, 0),
            ];
            let volume = VolumeMap::new(
                "Macintosh HD".into(),
                MountKind::External,
                ByteSize::new(20),
                parts,
                Some(ByteSize::new(7)),
                vec![],
                true,
            )
            .unwrap();

            assert_eq!(
                (volume.name(), volume.kind(), volume.capacity()),
                ("Macintosh HD", MountKind::External, ByteSize::new(20))
            );
            assert_eq!((volume.parts().len(), volume.roots(), volume.disk_changed()), (3, &[][..], true));
            assert_eq!(volume.purgeable(), Some(ByteSize::new(7)));
            let note = volume.purgeable_note().unwrap();
            assert_eq!(note.key().as_str(), "core.storage.purgeable");
            assert_eq!(note.args().get("size"), Some(&TextArg::Size(ByteSize::new(7))));
            assert_eq!(volume.changed_note().unwrap().as_str(), "core.storage.changed");
        }

        #[test]
        fn a_remainder_is_named_by_the_roots() {
            let outside = PartKind::Remainder { outside_scan: true, reasons: vec![] };
            let inside = PartKind::Remainder { outside_scan: false, reasons: vec![RemainderReason::ChangedDuringScan] };

            assert_eq!(outside.name_key().as_str(), "core.storage.remainder.outside.name");
            assert_eq!(inside.description_key().as_str(), "core.storage.remainder.not-measured.description");
        }
    }
}
