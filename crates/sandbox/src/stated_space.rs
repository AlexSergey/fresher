use std::fmt;
use std::path::Path;
use std::sync::Arc;

use fresher_core::{ByteSize, DiskSpace, ReadError, VolumeFacts, Volumes};
use fresher_testing::FakeVolumes;

use crate::DemoVolume;

/// The volumes of sandbox mode in a demo world that states its volume: the stated space answered for every path, so
/// the app never shows the developer's disk; which volume a path is on and the bytes still available are the inner
/// volumes'.
pub struct StatedSpace {
    inner: Arc<dyn Volumes>,
    space: DiskSpace,
}

impl StatedSpace {
    /// `inner` with the space of `volume`: a plain internal volume mounted at `/`, no purgeable space, no neighbours.
    pub fn new(inner: Arc<dyn Volumes>, volume: &DemoVolume) -> Self {
        let bytes = ByteSize::new;
        let space =
            FakeVolumes::disk(volume.name(), bytes(volume.capacity()), bytes(volume.used()), bytes(volume.free()));
        Self { inner, space }
    }
}

impl fmt::Debug for StatedSpace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("StatedSpace").field("space", &self.space).finish_non_exhaustive()
    }
}

impl Volumes for StatedSpace {
    fn identify(&self, path: &Path) -> Result<VolumeFacts, ReadError> {
        self.inner.identify(path)
    }

    fn available(&self, path: &Path) -> Result<ByteSize, ReadError> {
        self.inner.available(path)
    }

    fn space(&self, _path: &Path) -> Result<DiskSpace, ReadError> {
        Ok(self.space.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fresher_core::{ByteSize, VolumeFacts, VolumeId};
    use fresher_testing::FakeVolumes;

    use super::StatedSpace;
    use crate::{CaseFile, DemoVolume};

    /// The volume a demo world states: 1000 bytes, 600 used, 400 free.
    fn volume() -> DemoVolume {
        let text = "feature = \"developer-mac\"\n[demo]\nnow = 1795000000\nshows = [\"trial.caches\"]\n\
                    [demo.volume]\nname = \"Macintosh HD\"\ncapacity = 1000\nused = 600\nfree = 400\n\
                    [[case]]\nname = \"a cache\"\nkind = \"scenery\"\n[[case.file]]\npath = \"home/a.bin\"\n";
        CaseFile::parse(text).unwrap().demo().unwrap().volume().unwrap().clone()
    }

    fn facts() -> VolumeFacts {
        VolumeFacts { id: VolumeId::new(7), device: 16, case_sensitive: false, used: ByteSize::new(1 << 30) }
    }

    fn inner() -> Arc<FakeVolumes> {
        Arc::new(FakeVolumes::default().with("/box", facts()).answering_available(Ok(ByteSize::new(123))))
    }

    mod negative {
        use std::path::Path;
        use std::sync::Arc;

        use fresher_core::{ReadError, Volumes as _};
        use fresher_testing::FakeVolumes;

        use super::{StatedSpace, inner, volume};

        #[test]
        fn the_inner_volumes_space_is_never_asked() {
            let inner = inner();
            let stated = StatedSpace::new(Arc::<FakeVolumes>::clone(&inner), &volume());

            assert!(stated.space(Path::new("/elsewhere")).is_ok());
            assert_eq!(inner.space_calls(), 0);
        }

        #[test]
        fn a_path_the_inner_volumes_do_not_know_is_not_identified() {
            let stated = StatedSpace::new(inner(), &volume());

            assert_eq!(stated.identify(Path::new("/elsewhere")), Err(ReadError::Vanished));
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::{ByteSize, MountKind, Volumes as _};

        use super::{StatedSpace, facts, inner, volume};

        #[test]
        fn the_stated_space_is_answered_for_any_path() {
            let stated = StatedSpace::new(inner(), &volume());

            for path in ["/", "/box/home", "/Volumes/External"] {
                let space = stated.space(Path::new(path)).unwrap();

                assert_eq!(space.name, "Macintosh HD", "{path}");
                assert_eq!(
                    (space.capacity, space.used, space.free),
                    (ByteSize::new(1000), ByteSize::new(600), ByteSize::new(400))
                );
                assert_eq!((space.mount_point.as_path(), space.kind), (Path::new("/"), MountKind::Internal));
                assert_eq!((space.purgeable, space.neighbours.len()), (None, 0));
            }
        }

        #[test]
        fn identify_and_available_are_the_inner_volumes() {
            let stated = StatedSpace::new(inner(), &volume());

            assert_eq!(stated.identify(Path::new("/box/home")), Ok(facts()));
            assert_eq!(stated.available(Path::new("/box/home")), Ok(ByteSize::new(123)));
        }
    }
}
