//! The five parts the disk card draws: the map's parts merged by what they mean to the user.

use serde::Serialize;
use ts_rs::TS;

use crate::{ByteSize, MapPart, PartKind, Sizes, TextKey};

/// One of the five parts of a volume's summary, in the order they are drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum SummaryKind {
    /// The findings: what can be removed.
    CanBeFreed,
    /// Everything else of the user's: scanned, not scanned yet, not measured, other volumes.
    FilesAndApps,
    /// The system's volumes.
    MacOs,
    /// Swap, the sleep image and the disk's own space.
    SystemSpace,
    /// The space the user can still write.
    Free,
}

impl SummaryKind {
    pub const ALL: [Self; 5] = [Self::CanBeFreed, Self::FilesAndApps, Self::MacOs, Self::SystemSpace, Self::Free];

    /// The summary part a map part belongs to.
    pub fn of(kind: &PartKind) -> Self {
        match kind {
            PartKind::Explained => Self::CanBeFreed,
            PartKind::YourFolders
            | PartKind::NotScannedYet
            | PartKind::Remainder { .. }
            | PartKind::OtherVolume { .. } => Self::FilesAndApps,
            PartKind::MacOs => Self::MacOs,
            PartKind::SwapAndSleep | PartKind::DiskOwn => Self::SystemSpace,
            PartKind::Free => Self::Free,
        }
    }

    /// What the part is called: `core.storage.summary.<kind>.name`.
    pub fn name_key(self) -> TextKey {
        self.key("name")
    }

    /// What the part holds, in one sentence: `core.storage.summary.<kind>.description`.
    pub fn description_key(self) -> TextKey {
        self.key("description")
    }

    fn key(self, text: &str) -> TextKey {
        let kind = match self {
            Self::CanBeFreed => "can-be-freed",
            Self::FilesAndApps => "files-and-apps",
            Self::MacOs => "mac-os",
            Self::SystemSpace => "system-space",
            Self::Free => "free",
        };
        TextKey::from_valid(format!("core.storage.summary.{kind}.{text}"))
    }
}

/// One part of a volume's summary. The headline is `sizes.allocated`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, TS)]
pub struct SummaryPart {
    pub kind: SummaryKind,
    pub sizes: Sizes,
}

/// The five parts in the order of [`SummaryKind::ALL`], each the sum of its map parts (a kind with none at zero).
pub(crate) fn summarize(parts: &[MapPart]) -> Vec<SummaryPart> {
    SummaryKind::ALL
        .into_iter()
        .map(|kind| {
            let sizes = parts
                .iter()
                .filter(|part| SummaryKind::of(&part.kind) == kind)
                .fold(Sizes::ZERO, |sum, part| saturating_add(sum, part.sizes));
            SummaryPart { kind, sizes }
        })
        .collect()
}

fn saturating_add(a: Sizes, b: Sizes) -> Sizes {
    let sum = |a: ByteSize, b: ByteSize| ByteSize::new(a.bytes().saturating_add(b.bytes()));
    Sizes {
        logical: sum(a.logical, b.logical),
        allocated: sum(a.allocated, b.allocated),
        reclaimable: sum(a.reclaimable, b.reclaimable),
        completeness: a.completeness.max(b.completeness),
    }
}

#[cfg(test)]
mod tests {
    use super::{SummaryKind, SummaryPart};
    use crate::{ByteSize, Sizes};

    fn sizes(bytes: u64) -> Sizes {
        Sizes::complete(ByteSize::new(bytes), ByteSize::new(bytes))
    }

    fn of(summary: &[SummaryPart]) -> Vec<(SummaryKind, Sizes)> {
        summary.iter().map(|part| (part.kind, part.sizes)).collect()
    }

    mod negative {
        use super::{of, sizes};
        use crate::storage::summary::summarize;
        use crate::{Completeness, MapPart, PartKind, Sizes, SummaryKind};

        #[test]
        fn an_empty_map_summarizes_to_five_zero_parts() {
            let summary = summarize(&[]);

            assert_eq!(of(&summary), SummaryKind::ALL.map(|kind| (kind, Sizes::ZERO)).to_vec());
        }

        #[test]
        fn a_sum_past_the_maximum_stays_at_it() {
            let parts = [
                MapPart { kind: PartKind::YourFolders, sizes: sizes(u64::MAX) },
                MapPart { kind: PartKind::NotScannedYet, sizes: sizes(7) },
            ];

            let summary = summarize(&parts);

            assert_eq!(summary[1].sizes, sizes(u64::MAX));
        }

        #[test]
        fn the_weakest_completeness_wins() {
            let partial = Sizes { completeness: Completeness::Partial, ..sizes(3) };
            let unknown = Sizes { completeness: Completeness::Unknown, ..sizes(4) };
            let parts = [
                MapPart { kind: PartKind::SwapAndSleep, sizes: partial },
                MapPart { kind: PartKind::DiskOwn, sizes: unknown },
                MapPart { kind: PartKind::Free, sizes: sizes(5) },
            ];

            let summary = summarize(&parts);

            assert_eq!(summary[3].sizes, Sizes { completeness: Completeness::Unknown, ..sizes(7) });
            assert_eq!((summary[4].sizes.completeness, summary[0].sizes), (Completeness::Complete, Sizes::ZERO));
        }
    }

    mod positive {
        use serde_json::json;

        use super::{of, sizes};
        use crate::storage::summary::summarize;
        use crate::{ByteSize, MapPart, MountKind, PartKind, SummaryKind, VolumeMap};

        #[test]
        fn every_part_kind_goes_to_its_summary_kind() {
            let parts = [
                (PartKind::Explained, 1),
                (PartKind::YourFolders, 2),
                (PartKind::NotScannedYet, 4),
                (PartKind::Remainder { outside_scan: true, reasons: vec![] }, 8),
                (PartKind::Remainder { outside_scan: false, reasons: vec![] }, 16),
                (PartKind::MacOs, 32),
                (PartKind::SwapAndSleep, 64),
                (PartKind::OtherVolume { name: "Backup".into() }, 128),
                (PartKind::OtherVolume { name: "Media".into() }, 256),
                (PartKind::DiskOwn, 512),
                (PartKind::Free, 1024),
            ]
            .map(|(kind, bytes)| MapPart { kind, sizes: sizes(bytes) });

            let summary = summarize(&parts);

            assert_eq!(
                of(&summary),
                vec![
                    (SummaryKind::CanBeFreed, sizes(1)),
                    (SummaryKind::FilesAndApps, sizes(2 + 4 + 8 + 16 + 128 + 256)),
                    (SummaryKind::MacOs, sizes(32)),
                    (SummaryKind::SystemSpace, sizes(64 + 512)),
                    (SummaryKind::Free, sizes(1024)),
                ]
            );
        }

        #[test]
        fn the_summary_adds_up_to_the_capacity() {
            let parts = crate::storage::every_kind()
                .into_iter()
                .map(|kind| MapPart { kind, sizes: sizes(10) })
                .collect::<Vec<_>>();
            let volume = VolumeMap::new(
                "Macintosh HD".into(),
                MountKind::Internal,
                ByteSize::new(90),
                parts,
                None,
                vec![],
                false,
            )
            .unwrap();

            let sum = volume.summary().iter().map(|part| part.sizes.allocated.bytes()).sum::<u64>();

            assert_eq!((volume.summary().len(), sum), (5, 90));
        }

        #[test]
        fn serializes_as_kebab_case() {
            let kinds = SummaryKind::ALL.map(|kind| serde_json::to_value(kind).unwrap());

            assert_eq!(
                kinds.to_vec(),
                vec![
                    json!("can-be-freed"),
                    json!("files-and-apps"),
                    json!("mac-os"),
                    json!("system-space"),
                    json!("free")
                ]
            );
        }

        #[test]
        fn the_keys_name_each_kind() {
            assert_eq!(SummaryKind::CanBeFreed.name_key().as_str(), "core.storage.summary.can-be-freed.name");
            assert_eq!(
                SummaryKind::SystemSpace.description_key().as_str(),
                "core.storage.summary.system-space.description"
            );
            assert_eq!(SummaryKind::MacOs.name_key().as_str(), "core.storage.summary.mac-os.name");
        }
    }
}
