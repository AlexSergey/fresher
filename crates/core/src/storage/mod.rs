//! The storage map: where a volume's space went, every byte in one part, in two views (by meaning and by place).
//! The types only; the engine fills them.

mod map;
mod meaning;
mod place;
mod summary;

pub use map::{MapPart, MapPhase, MapPlace, PartKind, RemainderReason, StorageMap, StorageMapError, VolumeMap};
pub use meaning::{AppRef, CategoryShare, MeaningGroup, PlaceName, PlaceShare};
pub use place::{Descent, PlaceRow, PlaceView};
pub use summary::{SummaryKind, SummaryPart};

#[cfg(test)]
pub(crate) use map::tests::{every_kind, every_reason};
